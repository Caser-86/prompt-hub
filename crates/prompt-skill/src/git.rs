use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;

use crate::{ScanLimits, SkillCandidate, scan_skill};

const MAX_TREE_LISTING_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TreeEntry {
    object: String,
    relative_path: PathBuf,
    bytes: u64,
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
    #[error("Git Skill snapshot exceeds the local review limits")]
    SnapshotLimit,
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
        run_git(&["init", "--bare", "--quiet"], &object_store)?;
        run_git(
            &[
                "-C",
                path(&object_store)?,
                "fetch",
                "--quiet",
                "--depth=1",
                "--no-tags",
                "--filter=blob:none",
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
            "-l",
            "-z",
            source.commit(),
        ];
        if !source.subdirectory().as_os_str().is_empty() {
            tree_arguments.extend(["--", path(source.subdirectory())?]);
        }
        let tree = git_output_limited(&tree_arguments, &object_store, MAX_TREE_LISTING_BYTES)?;
        let entries = parse_tree_entries(&tree, source, ScanLimits::default())?;
        fs::create_dir(snapshot_root)?;
        for entry in entries {
            let output = snapshot_root.join(&entry.relative_path);
            fs::create_dir_all(output.parent().ok_or(GitSkillError::UnsafeTree)?)?;
            let contents = git_output_limited(
                &[
                    "-C",
                    path(&object_store)?,
                    "cat-file",
                    "blob",
                    &entry.object,
                ],
                &object_store,
                entry.bytes,
            )?;
            if u64::try_from(contents.len()).map_err(|_| GitSkillError::SnapshotLimit)?
                != entry.bytes
            {
                return Err(GitSkillError::UnsafeTree);
            }
            fs::write(output, contents)?;
        }
        scan_skill(snapshot_root).map_err(GitSkillError::from)
    })();
    let _ = fs::remove_dir_all(&object_store);
    if outcome.is_err() && snapshot_root.exists() {
        let _ = fs::remove_dir_all(snapshot_root);
    }
    outcome
}

fn parse_tree_entries(
    tree: &[u8],
    source: &GitSkillSource,
    limits: ScanLimits,
) -> Result<Vec<TreeEntry>, GitSkillError> {
    let mut entries = Vec::new();
    let mut total_bytes = 0_u64;
    let prefix = format_prefix(source.subdirectory());
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
        if fields.len() != 4
            || !matches!(fields[0], "100644" | "100755")
            || fields[1] != "blob"
            || fields[2].len() != 40
            || !fields[2].bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(GitSkillError::UnsafeTree);
        }
        let bytes = fields[3]
            .parse::<u64>()
            .map_err(|_| GitSkillError::UnsafeTree)?;
        let remote_path =
            std::str::from_utf8(remote_path).map_err(|_| GitSkillError::UnsafeTree)?;
        let relative = remote_path
            .strip_prefix(&prefix)
            .ok_or(GitSkillError::UnsafeTree)?;
        let relative_path = PathBuf::from(relative);
        let depth = relative_path.components().count();
        if relative.is_empty()
            || depth > limits.max_depth
            || relative_path
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(GitSkillError::UnsafeTree);
        }
        total_bytes = total_bytes
            .checked_add(bytes)
            .ok_or(GitSkillError::SnapshotLimit)?;
        if entries.len() >= limits.max_files
            || bytes > limits.max_file_bytes
            || total_bytes > limits.max_total_bytes
        {
            return Err(GitSkillError::SnapshotLimit);
        }
        entries.push(TreeEntry {
            object: fields[2].to_ascii_lowercase(),
            relative_path,
            bytes,
        });
    }
    Ok(entries)
}

fn run_git(arguments: &[&str], directory: &Path) -> Result<(), GitSkillError> {
    if safe_git_command()
        .args(arguments)
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(GitSkillError::Io)?
        .success()
    {
        Ok(())
    } else {
        Err(GitSkillError::Git)
    }
}
fn git_output_limited(
    arguments: &[&str],
    directory: &Path,
    max_bytes: u64,
) -> Result<Vec<u8>, GitSkillError> {
    let mut child = safe_git_command()
        .args(arguments)
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(GitSkillError::Io)?;
    let mut output = Vec::new();
    child
        .stdout
        .take()
        .ok_or(GitSkillError::Git)?
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut output)?;
    if u64::try_from(output.len()).map_err(|_| GitSkillError::SnapshotLimit)? > max_bytes {
        let _ = child.kill();
        let _ = child.wait();
        return Err(GitSkillError::SnapshotLimit);
    }
    if !child.wait().map_err(GitSkillError::Io)?.success() {
        Err(GitSkillError::Git)
    } else {
        Ok(output)
    }
}

fn safe_git_command() -> Command {
    let mut command = Command::new("git");
    command
        .arg("-c")
        .arg("credential.helper=")
        .arg("-c")
        .arg("protocol.file.allow=never")
        .arg("-c")
        .arg("protocol.ext.allow=never")
        .arg("-c")
        .arg("submodule.recurse=false")
        .arg("-c")
        .arg(format!("core.hooksPath={}", null_device()))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", null_device())
        .env("GIT_CONFIG_SYSTEM", null_device())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "Never");
    for variable in [
        "GIT_CONFIG_COUNT",
        "GIT_CONFIG_PARAMETERS",
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
        "GIT_NAMESPACE",
        "GIT_SSH",
        "GIT_SSH_COMMAND",
        "GIT_PROXY_COMMAND",
        "GIT_EXTERNAL_DIFF",
        "GIT_EXEC_PATH",
        "GIT_TEMPLATE_DIR",
    ] {
        command.env_remove(variable);
    }
    command
}

#[cfg(windows)]
const fn null_device() -> &'static str {
    "NUL"
}

#[cfg(not(windows))]
const fn null_device() -> &'static str {
    "/dev/null"
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
    let path = path.strip_suffix('/').unwrap_or(path);
    let segments = path.split('/').collect::<Vec<_>>();
    if segments.len() != 2
        || !segments
            .iter()
            .all(|segment| valid_repository_segment(segment))
        || value.contains('@')
        || value.contains('?')
        || value.contains('#')
        || value.contains('\\')
    {
        return Err(GitSkillError::RepositoryUrl);
    }
    Ok(())
}

fn valid_repository_segment(value: &str) -> bool {
    let value = value.strip_suffix(".git").unwrap_or(value);
    !value.is_empty()
        && !matches!(value, "." | "..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_listing_is_rejected_before_materialization_when_a_blob_exceeds_limits() {
        let source = GitSkillSource::new(
            "https://github.com/example/skills.git",
            "0123456789abcdef0123456789abcdef01234567",
            PathBuf::new(),
        )
        .unwrap();
        let object = "1111111111111111111111111111111111111111";
        let listing = format!("100644 blob {object} 2097153\tSKILL.md\0");

        assert!(matches!(
            parse_tree_entries(listing.as_bytes(), &source, ScanLimits::default()),
            Err(GitSkillError::SnapshotLimit)
        ));
    }

    #[test]
    fn git_commands_disable_ambient_configuration_and_unsafe_protocols() {
        let command = safe_git_command();
        let args = command
            .get_args()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let environment = command
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().into_owned(),
                    value.map(|item| item.to_string_lossy().into_owned()),
                )
            })
            .collect::<Vec<_>>();

        assert!(
            args.windows(2)
                .any(|pair| pair == ["-c", "credential.helper="])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["-c", "protocol.file.allow=never"])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["-c", "protocol.ext.allow=never"])
        );
        assert!(
            environment.iter().any(|(key, value)| {
                key == "GIT_CONFIG_NOSYSTEM" && value.as_deref() == Some("1")
            })
        );
        assert!(
            environment
                .iter()
                .any(|(key, value)| key == "GIT_CONFIG_COUNT" && value.is_none())
        );
    }
}
