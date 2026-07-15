use std::fs;

use prompt_import::{ImportFormat, normalized_body_fingerprint, parse_file, scan_folder};
use tempfile::tempdir;

#[test]
fn parses_supported_files_into_unpublished_candidates() {
    let directory = tempdir().unwrap();
    let markdown = directory.path().join("代码审查.md");
    fs::write(
        &markdown,
        include_str!("../../../tests/fixtures/import/valid.md"),
    )
    .unwrap();
    let text = directory.path().join("requirements.txt");
    fs::write(
        &text,
        include_str!("../../../tests/fixtures/import/valid.txt"),
    )
    .unwrap();
    let json = directory.path().join("prompts.json");
    fs::write(
        &json,
        include_str!("../../../tests/fixtures/import/valid.json"),
    )
    .unwrap();
    let csv = directory.path().join("prompts.csv");
    fs::write(
        &csv,
        include_str!("../../../tests/fixtures/import/valid.csv"),
    )
    .unwrap();

    let markdown_candidates = parse_file(&markdown).unwrap();
    let text_candidates = parse_file(&text).unwrap();
    let json_candidates = parse_file(&json).unwrap();
    let csv_candidates = parse_file(&csv).unwrap();

    assert_eq!(markdown_candidates[0].title, "代码审查");
    assert_eq!(markdown_candidates[0].format, ImportFormat::Markdown);
    assert_eq!(text_candidates[0].format, ImportFormat::Text);
    assert_eq!(json_candidates[0].body, "这是一个待审核提示词。");
    assert_eq!(csv_candidates[0].title, "CSV 导入");
    assert!(
        markdown_candidates
            .iter()
            .all(|candidate| !candidate.publishable)
    );
}

#[test]
fn rejects_invalid_import_fixtures_for_every_supported_format() {
    let directory = tempdir().unwrap();
    for (name, fixture) in [
        (
            "invalid.md",
            include_str!("../../../tests/fixtures/import/invalid.md"),
        ),
        (
            "invalid.txt",
            include_str!("../../../tests/fixtures/import/invalid.txt"),
        ),
        (
            "invalid.json",
            include_str!("../../../tests/fixtures/import/invalid.json"),
        ),
        (
            "invalid.csv",
            include_str!("../../../tests/fixtures/import/invalid.csv"),
        ),
    ] {
        let path = directory.path().join(name);
        fs::write(&path, fixture).unwrap();
        assert!(parse_file(&path).is_err(), "{name} should be rejected");
    }
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
