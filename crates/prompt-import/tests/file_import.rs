use std::fs;

use prompt_import::{ImportFormat, normalized_body_fingerprint, parse_file, scan_folder};
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

#[test]
fn fingerprints_normalized_bodies_for_exact_duplicate_detection() {
    assert_eq!(
        normalized_body_fingerprint("审查  当前\r\n变更"),
        normalized_body_fingerprint("审查 当前\n变更")
    );
    assert_ne!(
        normalized_body_fingerprint("审查当前变更"),
        normalized_body_fingerprint("生成测试")
    );
}

#[test]
fn recursively_scans_supported_files_and_skips_unrelated_files() {
    let directory = tempdir().unwrap();
    let nested = directory.path().join("子目录");
    fs::create_dir(&nested).unwrap();
    fs::write(directory.path().join("one.txt"), "第一条").unwrap();
    fs::write(nested.join("two.md"), "# 第二条\n\n正文").unwrap();
    fs::write(nested.join("ignored.png"), "not an image").unwrap();

    let candidates = scan_folder(directory.path()).unwrap();

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].title, "one");
    assert_eq!(candidates[1].title, "第二条");
}
