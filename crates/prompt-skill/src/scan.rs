use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};
use thiserror::Error;

const DEFAULT_MAX_FILES: usize = 512;
const DEFAULT_MAX_TOTAL_BYTES: u64 = 16 * 1024 * 1024;
const DEFAULT_MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
const DEFAULT_MAX_DEPTH: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanLimits {
    pub max_files: usize,
    pub max_total_bytes: u64,
    pub max_file_bytes: u64,
    pub max_depth: usize,
}

impl Default for ScanLimits {
    fn default() -> Self {
        Self {
            max_files: DEFAULT_MAX_FILES,
            max_total_bytes: DEFAULT_MAX_TOTAL_BYTES,
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillFileKind {
    SkillMarkdown,
    Script,
    Binary,
    Hidden,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillRisk {
    ContainsScript,
    ContainsBinary,
    ContainsHiddenFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillFile {
    relative_path: String,
    bytes: u64,
    sha256: String,
    kind: SkillFileKind,
}

impl SkillFile {
    #[must_use]
    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    #[must_use]
    pub const fn kind(&self) -> SkillFileKind {
        self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillCandidate {
    name: String,
    description: String,
    skill_markdown: String,
    files: Vec<SkillFile>,
    risks: BTreeSet<SkillRisk>,
    content_hash: String,
    total_bytes: u64,
}

impl SkillCandidate {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub fn skill_markdown(&self) -> &str {
        &self.skill_markdown
    }

    #[must_use]
    pub fn files(&self) -> &[SkillFile] {
        &self.files
    }

    #[must_use]
    pub fn risks(&self) -> &BTreeSet<SkillRisk> {
        &self.risks
    }

    #[must_use]
    pub fn content_hash(&self) -> &str {
        &self.content_hash
    }

    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

#[derive(Debug, Error)]
pub enum SkillScanError {
    #[error("Skill folder is not a directory")]
    NotDirectory,
    #[error("Skill folder is missing SKILL.md")]
    MissingSkillMarkdown,
    #[error("Skill folder contains a symbolic link: {0}")]
    SymbolicLink(String),
    #[error("Skill folder exceeds the file limit")]
    FileLimit,
    #[error("Skill folder exceeds the total size limit")]
    TotalSizeLimit,
    #[error("Skill file exceeds the per-file size limit: {0}")]
    FileSizeLimit(String),
    #[error("Skill folder exceeds the directory depth limit")]
    DepthLimit,
    #[error("Skill file path is invalid")]
    InvalidPath,
    #[error("SKILL.md must be valid UTF-8 text")]
    InvalidSkillMarkdown,
    #[error("unable to read Skill folder")]
    Read(#[from] std::io::Error),
}

pub fn scan_skill(root: &Path) -> Result<SkillCandidate, SkillScanError> {
    scan_skill_with_limits(root, ScanLimits::default())
}

pub fn scan_skill_with_limits(
    root: &Path,
    limits: ScanLimits,
) -> Result<SkillCandidate, SkillScanError> {
    if !root.is_dir() {
        return Err(SkillScanError::NotDirectory);
    }
    if fs::symlink_metadata(root)?.file_type().is_symlink() {
        return Err(SkillScanError::SymbolicLink(root.display().to_string()));
    }

    let mut files = Vec::new();
    let mut risks = BTreeSet::new();
    let mut total_bytes = 0;
    collect_files(
        root,
        root,
        0,
        limits,
        &mut files,
        &mut risks,
        &mut total_bytes,
    )?;
    let markdown_path = root.join("SKILL.md");
    if !markdown_path.is_file() {
        return Err(SkillScanError::MissingSkillMarkdown);
    }
    let skill_markdown = fs::read_to_string(&markdown_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::InvalidData {
            SkillScanError::InvalidSkillMarkdown
        } else {
            SkillScanError::Read(error)
        }
    })?;
    if !files.iter().any(|file| file.relative_path == "SKILL.md") {
        return Err(SkillScanError::MissingSkillMarkdown);
    }

    let (name, description) = parse_metadata(&skill_markdown, root);
    let mut fingerprint = Sha256::new();
    for file in &files {
        fingerprint.update(file.relative_path.as_bytes());
        fingerprint.update([0]);
        fingerprint.update(file.sha256.as_bytes());
        fingerprint.update([0]);
    }
    Ok(SkillCandidate {
        name,
        description,
        skill_markdown,
        files,
        risks,
        content_hash: format!("{:x}", fingerprint.finalize()),
        total_bytes,
    })
}

#[allow(clippy::too_many_arguments)]
fn collect_files(
    root: &Path,
    directory: &Path,
    depth: usize,
    limits: ScanLimits,
    files: &mut Vec<SkillFile>,
    risks: &mut BTreeSet<SkillRisk>,
    total_bytes: &mut u64,
) -> Result<(), SkillScanError> {
    if depth > limits.max_depth {
        return Err(SkillScanError::DepthLimit);
    }
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(SkillScanError::SymbolicLink(relative_path(root, &path)?));
        }
        if metadata.is_dir() {
            collect_files(root, &path, depth + 1, limits, files, risks, total_bytes)?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }
        if files.len() >= limits.max_files {
            return Err(SkillScanError::FileLimit);
        }
        let relative_path = relative_path(root, &path)?;
        let bytes = metadata.len();
        if bytes > limits.max_file_bytes {
            return Err(SkillScanError::FileSizeLimit(relative_path));
        }
        *total_bytes = total_bytes
            .checked_add(bytes)
            .ok_or(SkillScanError::TotalSizeLimit)?;
        if *total_bytes > limits.max_total_bytes {
            return Err(SkillScanError::TotalSizeLimit);
        }
        let content = fs::read(&path)?;
        let kind = classify_file(&relative_path, &content);
        match kind {
            SkillFileKind::Script => {
                risks.insert(SkillRisk::ContainsScript);
            }
            SkillFileKind::Binary => {
                risks.insert(SkillRisk::ContainsBinary);
            }
            SkillFileKind::Hidden => {
                risks.insert(SkillRisk::ContainsHiddenFile);
            }
            SkillFileKind::SkillMarkdown | SkillFileKind::Text => {}
        }
        let digest = Sha256::digest(content);
        files.push(SkillFile {
            relative_path,
            bytes,
            sha256: format!("{digest:x}"),
            kind,
        });
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> Result<String, SkillScanError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| SkillScanError::InvalidPath)?;
    if relative.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(SkillScanError::InvalidPath);
    }
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn classify_file(relative_path: &str, content: &[u8]) -> SkillFileKind {
    if relative_path == "SKILL.md" {
        return SkillFileKind::SkillMarkdown;
    }
    if relative_path
        .split('/')
        .any(|segment| segment.starts_with('.'))
    {
        return SkillFileKind::Hidden;
    }
    let extension = PathBuf::from(relative_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if matches!(
        extension.as_str(),
        "ps1" | "bat" | "cmd" | "sh" | "py" | "js" | "mjs" | "cjs"
    ) {
        return SkillFileKind::Script;
    }
    if content.contains(&0) {
        return SkillFileKind::Binary;
    }
    SkillFileKind::Text
}

fn parse_metadata(markdown: &str, root: &Path) -> (String, String) {
    let fallback_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("未命名 Skill")
        .to_owned();
    let mut name = None;
    let mut description = None;
    let mut lines = markdown.lines();
    if lines.next().is_some_and(|line| line.trim() == "---") {
        for line in lines.by_ref() {
            if line.trim() == "---" {
                break;
            }
            if let Some(value) = line.strip_prefix("name:") {
                name = Some(unquote(value));
            }
            if let Some(value) = line.strip_prefix("description:") {
                description = Some(unquote(value));
            }
        }
    }
    let name = name
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback_name);
    let description = description
        .filter(|value| !value.is_empty())
        .or_else(|| {
            markdown
                .lines()
                .find_map(|line| line.strip_prefix("# ").map(str::to_owned))
        })
        .unwrap_or_else(|| "未提供说明".to_owned());
    (name, description)
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches(['\'', '"']).to_owned()
}
