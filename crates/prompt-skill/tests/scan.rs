use std::fs;

use prompt_skill::{ScanLimits, SkillFileKind, SkillRisk, scan_skill};
use tempfile::tempdir;

fn write_skill(root: &std::path::Path, markdown: &str) {
    fs::write(root.join("SKILL.md"), markdown).unwrap();
}

#[test]
fn scans_skill_metadata_hashes_and_script_risk_without_execution() {
    let directory = tempdir().unwrap();
    write_skill(
        directory.path(),
        "---\nname: safe-skill\ndescription: Safe local workflow\n---\n# Safe skill\n",
    );
    fs::create_dir(directory.path().join("scripts")).unwrap();
    fs::write(
        directory.path().join("scripts/run.ps1"),
        "Write-Output should-not-run",
    )
    .unwrap();

    let candidate = scan_skill(directory.path()).unwrap();

    assert_eq!(candidate.name(), "safe-skill");
    assert_eq!(candidate.description(), "Safe local workflow");
    assert!(candidate.content_hash().len() == 64);
    assert!(
        candidate
            .files()
            .iter()
            .any(|file| file.kind() == SkillFileKind::Script)
    );
    assert!(candidate.risks().contains(&SkillRisk::ContainsScript));
}

#[test]
fn rejects_a_directory_without_skill_markdown() {
    let directory = tempdir().unwrap();
    let error = scan_skill(directory.path()).unwrap_err();
    assert!(error.to_string().contains("SKILL.md"));
}

#[test]
fn rejects_a_skill_that_exceeds_the_file_limit() {
    let directory = tempdir().unwrap();
    write_skill(directory.path(), "# Limited skill\nRead only.\n");
    fs::write(directory.path().join("reference.md"), "reference").unwrap();

    let error = prompt_skill::scan_skill_with_limits(
        directory.path(),
        ScanLimits {
            max_files: 1,
            ..ScanLimits::default()
        },
    )
    .unwrap_err();

    assert!(error.to_string().contains("file limit"));
}
