use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;

use crate::scan_skill;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMode {
    FailIfExists,
    ReplaceAfterBackup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallRequest<'a> {
    pub source: &'a Path,
    pub target_root: &'a Path,
    pub backup_root: &'a Path,
    pub destination_name: &'a str,
    pub expected_content_hash: &'a str,
    pub mode: InstallMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationReceipt {
    install_path: PathBuf,
    backup_path: Option<PathBuf>,
    installed_hash: String,
}

impl InstallationReceipt {
    #[must_use]
    pub fn install_path(&self) -> &Path {
        &self.install_path
    }
    #[must_use]
    pub fn backup_path(&self) -> Option<&Path> {
        self.backup_path.as_deref()
    }
    #[must_use]
    pub fn installed_hash(&self) -> &str {
        &self.installed_hash
    }
}

#[derive(Debug, Error)]
pub enum SkillInstallError {
    #[error("Skill destination name is invalid")]
    InvalidDestinationName,
    #[error("Skill source content changed after review")]
    SourceChanged,
    #[error("Skill destination already exists")]
    DestinationExists,
    #[error("Skill backup path already exists")]
    BackupExists,
    #[error("Skill installation changed after copying; automatic rollback was refused")]
    RollbackUnsafe,
    #[error("unable to install Skill")]
    Io(#[from] std::io::Error),
    #[error("unable to verify installed Skill")]
    Scan(#[from] crate::SkillScanError),
}

/// Copies a reviewed Skill without executing any of its files.
pub fn install_skill(
    request: InstallRequest<'_>,
) -> Result<InstallationReceipt, SkillInstallError> {
    validate_destination_name(request.destination_name)?;
    let source = scan_skill(request.source)?;
    if source.content_hash() != request.expected_content_hash {
        return Err(SkillInstallError::SourceChanged);
    }
    fs::create_dir_all(request.target_root)?;
    fs::create_dir_all(request.backup_root)?;
    let destination = request.target_root.join(request.destination_name);
    let stage = request.target_root.join(format!(
        ".{}-prompt-hub-stage-{}",
        request.destination_name,
        unique_suffix()
    ));
    fs::create_dir(&stage)?;
    let install = (|| {
        for file in source.files() {
            let relative = Path::new(file.relative_path());
            let output = stage.join(relative);
            let parent = output
                .parent()
                .ok_or(SkillInstallError::InvalidDestinationName)?;
            fs::create_dir_all(parent)?;
            fs::copy(request.source.join(relative), output)?;
        }
        let staged = scan_skill(&stage)?;
        if staged.content_hash() != request.expected_content_hash {
            return Err(SkillInstallError::SourceChanged);
        }
        let backup_path = if destination.exists() {
            if request.mode == InstallMode::FailIfExists {
                return Err(SkillInstallError::DestinationExists);
            }
            let backup = request.backup_root.join(format!(
                "{}-{}",
                request.destination_name,
                unique_suffix()
            ));
            if backup.exists() {
                return Err(SkillInstallError::BackupExists);
            }
            fs::rename(&destination, &backup)?;
            Some(backup)
        } else {
            None
        };
        if let Err(error) = fs::rename(&stage, &destination) {
            if let Some(backup) = &backup_path {
                let _ = fs::rename(backup, &destination);
            }
            return Err(SkillInstallError::Io(error));
        }
        Ok(InstallationReceipt {
            install_path: destination,
            backup_path,
            installed_hash: staged.content_hash().to_owned(),
        })
    })();
    if stage.exists() {
        let _ = fs::remove_dir_all(&stage);
    }
    install
}

/// Reverts a completed installation when a later operation, such as persistence
/// of its receipt, fails. The installed tree is removed only while it still
/// matches the hash that was produced by [`install_skill`].
pub fn rollback_installation(receipt: &InstallationReceipt) -> Result<(), SkillInstallError> {
    let installed =
        scan_skill(receipt.install_path()).map_err(|_| SkillInstallError::RollbackUnsafe)?;
    if installed.content_hash() != receipt.installed_hash() {
        return Err(SkillInstallError::RollbackUnsafe);
    }

    fs::remove_dir_all(receipt.install_path())?;
    if let Some(backup_path) = receipt.backup_path() {
        fs::rename(backup_path, receipt.install_path())?;
    }
    Ok(())
}

fn validate_destination_name(value: &str) -> Result<(), SkillInstallError> {
    let path = Path::new(value);
    if value.is_empty()
        || path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
    {
        return Err(SkillInstallError::InvalidDestinationName);
    }
    Ok(())
}
fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}
