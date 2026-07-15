use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, backup::Backup};

use crate::StoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupDestination {
    Manual,
    Migration,
    PermanentDelete,
    PreRestore,
}

impl BackupDestination {
    const fn label(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Migration => "migration",
            Self::PermanentDelete => "permanent-delete",
            Self::PreRestore => "pre-restore",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackupMetadata {
    path: PathBuf,
    destination: BackupDestination,
    byte_len: u64,
    schema_version: u32,
}

impl BackupMetadata {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
    #[must_use]
    pub const fn destination(&self) -> BackupDestination {
        self.destination
    }
    #[must_use]
    pub const fn byte_len(&self) -> u64 {
        self.byte_len
    }
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }
}

#[derive(Debug, Clone)]
pub struct RestorePreview {
    target_exists: bool,
    backup_schema_version: u32,
    backup_byte_len: u64,
}

impl RestorePreview {
    #[must_use]
    pub const fn target_exists(&self) -> bool {
        self.target_exists
    }
    #[must_use]
    pub const fn backup_schema_version(&self) -> u32 {
        self.backup_schema_version
    }
    #[must_use]
    pub const fn backup_byte_len(&self) -> u64 {
        self.backup_byte_len
    }
}

#[derive(Debug, Clone)]
pub struct RestoreReport {
    pre_replacement_backup: Option<PathBuf>,
}

impl RestoreReport {
    #[must_use]
    pub fn pre_replacement_backup(&self) -> Option<&Path> {
        self.pre_replacement_backup.as_deref()
    }
}

pub fn create_backup(
    source: &Path,
    destination: BackupDestination,
) -> Result<BackupMetadata, StoreError> {
    let backup_directory = source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("backups");
    fs::create_dir_all(&backup_directory)?;
    let path = backup_directory.join(format!(
        "{}-{}-{}.db",
        source
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("prompt-hub"),
        destination.label(),
        timestamp()?
    ));
    copy_database(source, &path)?;
    backup_metadata(path, destination)
}

pub fn preview_restore(backup: &Path, target: &Path) -> Result<RestorePreview, StoreError> {
    let metadata = verified_database(backup)?;
    Ok(RestorePreview {
        target_exists: target.exists(),
        backup_schema_version: metadata.0,
        backup_byte_len: metadata.1,
    })
}

pub fn restore_backup(backup: &Path, target: &Path) -> Result<RestoreReport, StoreError> {
    verified_database(backup)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let pre_replacement_backup = if target.exists() {
        Some(create_backup(target, BackupDestination::PreRestore)?.path)
    } else {
        None
    };
    let temporary = target.with_extension(format!("restore-{}.tmp", timestamp()?));
    copy_database(backup, &temporary)?;
    if let Err(error) = replace_database(&temporary, target) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(RestoreReport {
        pre_replacement_backup,
    })
}

fn backup_metadata(
    path: PathBuf,
    destination: BackupDestination,
) -> Result<BackupMetadata, StoreError> {
    let (schema_version, byte_len) = verified_database(&path)?;
    Ok(BackupMetadata {
        path,
        destination,
        byte_len,
        schema_version,
    })
}

fn copy_database(source: &Path, target: &Path) -> Result<(), StoreError> {
    let source_connection = Connection::open(source)?;
    let mut target_connection = Connection::open(target)?;
    Backup::new(&source_connection, &mut target_connection)?.run_to_completion(
        64,
        std::time::Duration::from_millis(5),
        None,
    )?;
    drop(target_connection);
    verified_database(target)?;
    Ok(())
}

fn replace_database(temporary: &Path, target: &Path) -> Result<(), StoreError> {
    if !target.exists() {
        fs::rename(temporary, target)?;
        return Ok(());
    }
    let previous = target.with_extension(format!("pre-restore-{}.tmp", timestamp()?));
    fs::rename(target, &previous)?;
    if let Err(error) = fs::rename(temporary, target) {
        let _ = fs::rename(&previous, target);
        return Err(StoreError::Io(error));
    }
    fs::remove_file(previous)?;
    Ok(())
}

fn verified_database(path: &Path) -> Result<(u32, u64), StoreError> {
    let connection = Connection::open(path)?;
    let integrity: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(StoreError::BackupIntegrity(integrity));
    }
    let schema_version = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    Ok((schema_version, fs::metadata(path)?.len()))
}

fn timestamp() -> Result<u128, StoreError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| StoreError::Clock(error.to_string()))?
        .as_millis())
}
