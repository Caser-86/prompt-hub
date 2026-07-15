use std::fs;
use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportFormat {
    Markdown,
    Text,
    Json,
    Csv,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportCandidate {
    pub title: String,
    pub body: String,
    pub format: ImportFormat,
    pub source_path: String,
    pub publishable: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Error)]
pub enum FileImportError {
    #[error("unable to read import file")]
    Read(#[from] std::io::Error),
    #[error("unsupported import format: {0}")]
    UnsupportedFormat(String),
    #[error("invalid JSON import")]
    Json(#[from] serde_json::Error),
    #[error("invalid CSV import")]
    Csv(#[from] csv::Error),
    #[error("import candidate needs a title and body")]
    CandidateRequired,
}

pub fn parse_file(path: &Path) -> Result<Vec<ImportCandidate>, FileImportError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let content = fs::read_to_string(path)?;
    match extension.as_str() {
        "md" | "markdown" => parse_markdown(path, &content),
        "txt" => Ok(vec![candidate(
            path,
            stem(path),
            content,
            ImportFormat::Text,
        )?]),
        "json" => parse_json(path, &content),
        "csv" => parse_csv(path, &content),
        _ => Err(FileImportError::UnsupportedFormat(extension)),
    }
}

#[must_use]
pub fn normalized_body_fingerprint(body: &str) -> String {
    let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let digest = Sha256::digest(normalized.as_bytes());
    format!("{digest:x}")
}

fn parse_markdown(path: &Path, content: &str) -> Result<Vec<ImportCandidate>, FileImportError> {
    let mut lines = content.lines();
    let first = lines.next().unwrap_or_default();
    let (title, body) = if let Some(title) = first.strip_prefix("# ") {
        (
            title.to_owned(),
            lines.collect::<Vec<_>>().join("\n").trim().to_owned(),
        )
    } else {
        (stem(path), content.to_owned())
    };
    Ok(vec![candidate(path, title, body, ImportFormat::Markdown)?])
}

fn parse_json(path: &Path, content: &str) -> Result<Vec<ImportCandidate>, FileImportError> {
    let value: Value = serde_json::from_str(content)?;
    let values = match value {
        Value::Array(values) => values,
        value => vec![value],
    };
    values
        .iter()
        .map(|value| {
            let title = value
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let body = value
                .get("body")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            candidate(path, title, body, ImportFormat::Json)
        })
        .collect()
}

fn parse_csv(path: &Path, content: &str) -> Result<Vec<ImportCandidate>, FileImportError> {
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    let headers = reader.headers()?.clone();
    let title = headers.iter().position(|header| header == "title");
    let body = headers.iter().position(|header| header == "body");
    reader
        .records()
        .map(|record| {
            let record = record?;
            candidate(
                path,
                title
                    .and_then(|index| record.get(index))
                    .unwrap_or_default()
                    .to_owned(),
                body.and_then(|index| record.get(index))
                    .unwrap_or_default()
                    .to_owned(),
                ImportFormat::Csv,
            )
        })
        .collect()
}

fn candidate(
    path: &Path,
    title: String,
    body: String,
    format: ImportFormat,
) -> Result<ImportCandidate, FileImportError> {
    let title = title.trim().to_owned();
    let body = body.trim().to_owned();
    if title.is_empty() || body.is_empty() {
        return Err(FileImportError::CandidateRequired);
    }
    Ok(ImportCandidate {
        title,
        body,
        format,
        source_path: path.to_string_lossy().into_owned(),
        publishable: false,
        warnings: Vec::new(),
    })
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("未命名提示词")
        .to_owned()
}
