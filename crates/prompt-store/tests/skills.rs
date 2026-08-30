use std::fs;

use prompt_skill::scan_skill;
use prompt_store::{Database, SkillReviewStatus, SkillSource};
use tempfile::tempdir;
use time::OffsetDateTime;

fn candidate() -> (tempfile::TempDir, prompt_skill::SkillCandidate) {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("SKILL.md"),
        "---\nname: stored-skill\ndescription: Stored safely\n---\n# Stored skill\n",
    )
    .unwrap();
    fs::write(directory.path().join("note.md"), "Reference").unwrap();
    let scanned = scan_skill(directory.path()).unwrap();
    (directory, scanned)
}

#[test]
fn stores_lists_reviews_and_favorites_skill_assets() {
    let (_directory, candidate) = candidate();
    let mut repository = Database::open_in_memory().unwrap().into_skill_repository();
    let now = OffsetDateTime::now_utc();
    let source = SkillSource::local_directory("C:/Skills/stored-skill");

    let stored = repository.save_candidate(&candidate, &source, now).unwrap();
    assert_eq!(stored.name(), "stored-skill");
    assert_eq!(stored.review_status(), SkillReviewStatus::PendingReview);
    assert!(stored.skill_markdown().contains("Stored skill"));

    repository
        .set_review(
            stored.id(),
            SkillReviewStatus::Approved,
            Some("checked locally"),
            now,
        )
        .unwrap();
    repository.set_favorite(stored.id(), true, now).unwrap();

    let listed = repository.list_skills().unwrap();
    assert_eq!(listed.len(), 1);
    assert!(listed[0].favorite());
    assert_eq!(listed[0].review_status(), SkillReviewStatus::Approved);
    assert!(listed[0].skill_markdown().is_none());

    let loaded = repository.get_skill(stored.id()).unwrap().unwrap();
    assert_eq!(loaded.files().len(), 2);
    assert_eq!(loaded.review_notes(), Some("checked locally"));
}

#[test]
fn deduplicates_the_same_source_and_content_without_creating_another_asset() {
    let (_directory, candidate) = candidate();
    let mut repository = Database::open_in_memory().unwrap().into_skill_repository();
    let now = OffsetDateTime::now_utc();
    let source = SkillSource::local_directory("C:/Skills/stored-skill");

    let first = repository.save_candidate(&candidate, &source, now).unwrap();
    let second = repository.save_candidate(&candidate, &source, now).unwrap();

    assert_eq!(first.id(), second.id());
    assert_eq!(repository.list_skills().unwrap().len(), 1);
}

#[test]
fn deduplication_keeps_the_latest_durable_snapshot_and_revision() {
    let (_directory, candidate) = candidate();
    let mut repository = Database::open_in_memory().unwrap().into_skill_repository();
    let first_source = SkillSource::git_repository(
        "https://github.com/example/skills.git",
        "1111111111111111111111111111111111111111",
    );
    let second_source = SkillSource::git_repository(
        "https://github.com/example/skills.git",
        "2222222222222222222222222222222222222222",
    );
    let first = repository
        .save_candidate_with_snapshot(
            &candidate,
            &first_source,
            Some("C:/snapshots/first"),
            OffsetDateTime::now_utc(),
        )
        .unwrap();
    let second = repository
        .save_candidate_with_snapshot(
            &candidate,
            &second_source,
            Some("C:/snapshots/second"),
            OffsetDateTime::now_utc(),
        )
        .unwrap();

    assert_eq!(first.id(), second.id());
    assert_eq!(second.snapshot_path(), Some("C:/snapshots/second"));
    assert_eq!(
        second.source().revision(),
        Some("2222222222222222222222222222222222222222")
    );
}
