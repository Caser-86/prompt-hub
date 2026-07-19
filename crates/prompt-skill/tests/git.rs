use std::path::PathBuf;

use prompt_skill::GitSkillSource;

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
}
