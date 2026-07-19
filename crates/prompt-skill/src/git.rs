use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;

use crate::{SkillCandidate, scan_skill};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitSkillSource {
    repository_url: String,
    commit: String,
    subdirectory: PathBuf,
}

impl GitSkillSource {
    pub fn new(
        repository_url: &str,
        commit: &str,
        subdirectory: PathBuf,
    ) -> Result<Self, GitSkillError> {
        validate_repository_url(repository_url)?;
        if commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(GitSkillError::Revision);
        }
        if !subdirectory.as_os_str().is_empty()
            && subdirectory
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(GitSkillError::Subdirectory);
        }
        Ok(Self {
            repository_url: repository_url.to_owned(),
            commit: commit.to_ascii_lowercase(),
            subdirectory,
        })
    }
    #[must_use]
    pub fn repository_url(&self) -> &str {
        &self.repository_url
    }
    #[must_use]
    pub fn commit(&self) -> &str {
        &self.commit
    }
    #[must_use]
    pub fn subdirectory(&self) -> &Path {
        &self.subdirectory
    }
}

#[derive(Debug, Error)]
pub enum GitSkillError {
    #[error("Git Skill source must be a public HTTPS GitHub repository URL")]
    RepositoryUrl,
    #[error("Git Skill source requires a fixed 40-character commit SHA")]
    Revision,
    #[error("Git Skill subdirectory is invalid")]
    Subdirectory,
    #[error("Skill snapshot path already exists")]
    SnapshotExists,
    #[error("Git did not return a safe regular file list")]
    UnsafeTree,
    #[error("Git command failed while reading the fixed revision")]
    Git,
    #[error("unable to create Skill snapshot")]
    Io(#[from] std::io::Error),
    #[error("unable to scan Skill snapshot")]
    Scan(#[from] crate::SkillScanError),
}

/// Fetches only objects addressed by a fixed commit and materializes regular files into a
/// controlled local snapshot. It never checks out the repository or runs its scripts.
pub fn snapshot_git_skill(
    source: &GitSkillSource,
    snapshot_root: &Path,
) -> Result<SkillCandidate, GitSkillError> {
    if snapshot_root.exists() {
        return Err(GitSkillError::SnapshotExists);
    }
    let object_store = snapshot_root.with_extension(format!("git-objects-{}", suffix()));
    fs::create_dir_all(&object_store)?;
    let outcome = (|| {
        git(&["init", "--bare", "--quiet"], &object_store)?;
        git(
            &[
                "-C",
                path(&object_store)?,
                "fetch",
                "--quiet",
                "--depth=1",
                source.repository_url(),
                source.commit(),
            ],
            &object_store,
        )?;
        let object_store_path = path(&object_store)?;
        let mut tree_arguments = vec![
            "-C",
            object_store_path,
            "ls-tree",
            "-r",
            "-z",
            source.commit(),
        ];
        if !source.subdirectory().as_os_str().is_empty() {
            tree_arguments.extend(["--", path(source.subdirectory())?]);
        }
        let tree = git_output(&tree_arguments, &object_store)?;
        fs::create_dir(snapshot_root)?;
        for entry in tree
            .split(|byte| *byte == 0)
            .filter(|entry| !entry.is_empty())
        {
            let tab = entry
                .iter()
                .position(|byte| *byte == b'\t')
                .ok_or(GitSkillError::UnsafeTree)?;
            let (header, remote_path) = (&entry[..tab], &entry[tab + 1..]);
            let fields = std::str::from_utf8(header)
                .map_err(|_| GitSkillError::UnsafeTree)?
                .split_whitespace()
                .collect::<Vec<_>>();
            if fields.len() != 3 || fields[0] == "120000" || fields[1] != "blob" {
                return Err(GitSkillError::UnsafeTree);
            }
            let remote_path =
                std::str::from_utf8(remote_path).map_err(|_| GitSkillError::UnsafeTree)?;
            let relative = remote_path
                .strip_prefix(&format_prefix(source.subdirectory()))
                .ok_or(GitSkillError::UnsafeTree)?;
            let relative_path = Path::new(relative);
            if relative.is_empty()
                || relative_path
                    .components()
                    .any(|component| !matches!(component, Component::Normal(_)))
            {
                return Err(GitSkillError::UnsafeTree);
            }
            let output = snapshot_root.join(relative_path);
            fs::create_dir_all(output.parent().ok_or(GitSkillError::UnsafeTree)?)?;
            let object = format!("{}:{}", source.commit(), remote_path);
            fs::write(
                output,
                git_output(
                    &["-C", path(&object_store)?, "show", &object],
                    &object_store,
                )?,
            )?;
        }
        scan_skill(snapshot_root).map_err(GitSkillError::from)
    })();
    let _ = fs::remove_dir_all(&object_store);
    if outcome.is_err() && snapshot_root.exists() {
        let _ = fs::remove_dir_all(snapshot_root);
    }
    outcome
}

fn git(arguments: &[&str], directory: &Path) -> Result<(), GitSkillError> {
    if Command::new("git")
        .args(arguments)
        .current_dir(directory)
        .status()
        .map_err(GitSkillError::Io)?
        .success()
    {
        Ok(())
    } else {
        Err(GitSkillError::Git)
    }
}
fn git_output(arguments: &[&str], directory: &Path) -> Result<Vec<u8>, GitSkillError> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(directory)
        .output()
        .map_err(GitSkillError::Io)?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(GitSkillError::Git)
    }
}
fn path(value: &Path) -> Result<&str, GitSkillError> {
    value.to_str().ok_or(GitSkillError::UnsafeTree)
}
fn format_prefix(value: &Path) -> String {
    if value.as_os_str().is_empty() {
        String::new()
    } else {
        format!("{}/", value.to_string_lossy().replace('\\', "/"))
    }
}
fn suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn validate_repository_url(value: &str) -> Result<(), GitSkillError> {
    let path = value
        .strip_prefix("https://github.com/")
        .ok_or(GitSkillError::RepositoryUrl)?;
    if path.is_empty()
        || value.contains('@')
        || value.contains('?')
        || value.contains('#')
        || value.contains('\\')
    {
        return Err(GitSkillError::RepositoryUrl);
    }
    Ok(())
}
