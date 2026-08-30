use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;

use crate::{ScanLimits, SkillCandidate, scan_skill};

const MAX_TREE_OUTPUT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug)]
struct TreeEntry {
    object_id: String,
    relative_path: PathBuf,
}

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
    #[error("Git Skill snapshot exceeds resource limits")]
    ResourceLimit,
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
    let git_config = object_store.join("prompt-hub-empty.gitconfig");
    fs::write(&git_config, "")?;
    let outcome = (|| {
        git(&["init", "--bare", "--quiet"], &object_store, &git_config)?;
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
            &git_config,
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
        let tree = git_output_limited(
            &tree_arguments,
            &object_store,
            &git_config,
            MAX_TREE_OUTPUT_BYTES,
        )?;
        let mut entries = Vec::new();
        let mut total_bytes = 0_u64;
        let limits = ScanLimits::default();
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
            let bytes = git_object_size(fields[2], &object_store, &git_config)?;
            enforce_snapshot_limits(entries.len(), total_bytes, bytes, limits)?;
            total_bytes = total_bytes
                .checked_add(bytes)
                .ok_or(GitSkillError::ResourceLimit)?;
            entries.push(TreeEntry {
                object_id: fields[2].to_owned(),
                relative_path: relative_path.to_owned(),
            });
        }
        fs::create_dir(snapshot_root)?;
        for entry in entries {
            let output = snapshot_root.join(entry.relative_path);
            fs::create_dir_all(output.parent().ok_or(GitSkillError::UnsafeTree)?)?;
            fs::write(
                output,
                git_output(
                    &[
                        "-C",
                        path(&object_store)?,
                        "cat-file",
                        "blob",
                        &entry.object_id,
                    ],
                    &object_store,
                    &git_config,
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

fn git(arguments: &[&str], directory: &Path, git_config: &Path) -> Result<(), GitSkillError> {
    if git_command(arguments, directory, git_config)
        .status()
        .map_err(GitSkillError::Io)?
        .success()
    {
        Ok(())
    } else {
        Err(GitSkillError::Git)
    }
}
fn git_output(
    arguments: &[&str],
    directory: &Path,
    git_config: &Path,
) -> Result<Vec<u8>, GitSkillError> {
    let output = git_command(arguments, directory, git_config)
        .output()
        .map_err(GitSkillError::Io)?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(GitSkillError::Git)
    }
}

fn git_output_limited(
    arguments: &[&str],
    directory: &Path,
    git_config: &Path,
    maximum_bytes: usize,
) -> Result<Vec<u8>, GitSkillError> {
    let mut child = git_command(arguments, directory, git_config)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(GitSkillError::Io)?;
    let mut output = Vec::new();
    child
        .stdout
        .take()
        .ok_or(GitSkillError::Git)?
        .take((maximum_bytes + 1) as u64)
        .read_to_end(&mut output)
        .map_err(GitSkillError::Io)?;
    if let Err(error) = tree_output_is_within_limit(output.len(), maximum_bytes) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    if child.wait().map_err(GitSkillError::Io)?.success() {
        Ok(output)
    } else {
        Err(GitSkillError::Git)
    }
}

fn tree_output_is_within_limit(bytes: usize, maximum_bytes: usize) -> Result<(), GitSkillError> {
    if bytes > maximum_bytes {
        return Err(GitSkillError::ResourceLimit);
    }
    Ok(())
}

fn git_command(arguments: &[&str], directory: &Path, git_config: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .args(arguments)
        .current_dir(directory)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", git_config)
        .env("GIT_ALLOW_PROTOCOL", "https")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0");
    command
}

fn git_object_size(
    object_id: &str,
    object_store: &Path,
    git_config: &Path,
) -> Result<u64, GitSkillError> {
    let output = git_output(
        &["-C", path(object_store)?, "cat-file", "-s", object_id],
        object_store,
        git_config,
    )?;
    std::str::from_utf8(&output)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .ok_or(GitSkillError::Git)
}

fn enforce_snapshot_limits(
    existing_files: usize,
    existing_total_bytes: u64,
    next_file_bytes: u64,
    limits: ScanLimits,
) -> Result<(), GitSkillError> {
    if existing_files >= limits.max_files
        || next_file_bytes > limits.max_file_bytes
        || existing_total_bytes
            .checked_add(next_file_bytes)
            .is_none_or(|total| total > limits.max_total_bytes)
    {
        return Err(GitSkillError::ResourceLimit);
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScanLimits;

    #[test]
    fn snapshot_limits_reject_large_tree_before_blob_materialization() {
        let limits = ScanLimits {
            max_files: 2,
            max_total_bytes: 8,
            max_file_bytes: 5,
            max_depth: 12,
        };
        assert!(matches!(
            enforce_snapshot_limits(2, 8, 1, limits),
            Err(GitSkillError::ResourceLimit)
        ));
        assert!(matches!(
            enforce_snapshot_limits(1, 4, 6, limits),
            Err(GitSkillError::ResourceLimit)
        ));
        assert!(matches!(
            enforce_snapshot_limits(1, 5, 4, limits),
            Err(GitSkillError::ResourceLimit)
        ));
        assert!(enforce_snapshot_limits(1, 3, 5, limits).is_ok());
    }

    #[test]
    fn tree_output_limit_is_bounded_to_prevent_unbounded_metadata_buffers() {
        assert!(tree_output_is_within_limit(MAX_TREE_OUTPUT_BYTES, MAX_TREE_OUTPUT_BYTES).is_ok());
        assert!(matches!(
            tree_output_is_within_limit(MAX_TREE_OUTPUT_BYTES + 1, MAX_TREE_OUTPUT_BYTES),
            Err(GitSkillError::ResourceLimit)
        ));
    }
}
