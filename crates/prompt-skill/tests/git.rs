use std::path::PathBuf;

use prompt_skill::{GitSkillSource, snapshot_git_skill};
use tempfile::tempdir;

#[test]
fn git_skill_source_requires_a_public_https_github_url_and_fixed_commit() {
    let valid = GitSkillSource::new(
        "https://github.com/example/skills.git",
        "0123456789abcdef0123456789abcdef01234567",
        PathBuf::from("reviewer"),
    )
    .unwrap();
    assert_eq!(valid.commit(), "0123456789abcdef0123456789abcdef01234567");
    assert!(
        GitSkillSource::new(
            "https://github.com/example/skills.git",
            "main",
            PathBuf::new()
        )
        .is_err()
    );
    assert!(
        GitSkillSource::new(
            "https://token@github.com/example/skills.git",
            "0123456789abcdef0123456789abcdef01234567",
            PathBuf::new()
        )
        .is_err()
    );
    assert!(
        GitSkillSource::new(
            "https://git.example.test/skills.git",
            "0123456789abcdef0123456789abcdef01234567",
            PathBuf::new()
        )
        .is_err()
    );
    assert!(
        GitSkillSource::new(
            "https://github.com/only-owner",
            "0123456789abcdef0123456789abcdef01234567",
            PathBuf::new()
        )
        .is_err()
    );
    assert!(
        GitSkillSource::new(
            "https://github.com/example/skills/extra",
            "0123456789abcdef0123456789abcdef01234567",
            PathBuf::new()
        )
        .is_err()
    );
}

#[test]
#[ignore = "requires public GitHub network access"]
fn snapshots_a_real_public_skill_at_an_immutable_commit_without_checkout() {
    let source = GitSkillSource::new(
        "https://github.com/openai/skills.git",
        "49f948faa9258a0c61caceaf225e179651397431",
        PathBuf::from("skills/.curated/define-goal"),
    )
    .unwrap();
    let storage = tempdir().unwrap();
    let snapshot = storage.path().join("snapshot");

    let candidate = snapshot_git_skill(&source, &snapshot).unwrap();

    assert!(snapshot.join("SKILL.md").is_file());
    assert!(!candidate.content_hash().is_empty());
}
