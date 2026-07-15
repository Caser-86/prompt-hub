use std::fs;

use prompt_import::{ImportFormat, parse_file};
use tempfile::tempdir;

#[test]
fn parses_supported_files_into_unpublished_candidates() {
    let directory = tempdir().unwrap();
    let markdown = directory.path().join("代码审查.md");
    fs::write(&markdown, "# 代码审查\n\n审查当前变更并给出证据。").unwrap();
    let json = directory.path().join("prompts.json");
    fs::write(&json, r#"[{"title":"翻译","body":"翻译为中文"}]"#).unwrap();
    let csv = directory.path().join("prompts.csv");
    fs::write(&csv, "title,body\n摘要,总结以下内容\n").unwrap();

    let markdown_candidates = parse_file(&markdown).unwrap();
    let json_candidates = parse_file(&json).unwrap();
    let csv_candidates = parse_file(&csv).unwrap();

    assert_eq!(markdown_candidates[0].title, "代码审查");
    assert_eq!(markdown_candidates[0].format, ImportFormat::Markdown);
    assert_eq!(json_candidates[0].body, "翻译为中文");
    assert_eq!(csv_candidates[0].title, "摘要");
    assert!(
        markdown_candidates
            .iter()
            .all(|candidate| !candidate.publishable)
    );
}
