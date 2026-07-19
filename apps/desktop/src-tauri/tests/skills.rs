use std::fs;

use prompt_hub_desktop_lib::commands::{SkillReviewInput, SkillService};
use prompt_store::Database;
use tempfile::tempdir;

#[test]
fn collecting_a_local_skill_creates_a_pending_review_asset_without_executing_scripts() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("SKILL.md"),
        "---\nname: desktop-skill\ndescription: Desktop test\n---\n# Desktop skill\n",
    )
    .unwrap();
    fs::create_dir(directory.path().join("scripts")).unwrap();
    fs::write(directory.path().join("scripts/run.cmd"), "exit 99").unwrap();
    let service = SkillService::new(Database::open_in_memory().unwrap().into_skill_repository());

    let collected = service
        .collect_local_folder(directory.path().to_path_buf())
        .unwrap();

    assert_eq!(collected.review_status, "pending_review");
    assert!(collected.risks.contains(&"contains_script".to_owned()));
    let listed = service.list().unwrap();
    assert!(
        serde_json::to_value(&listed[0])
            .unwrap()
            .get("skillMarkdown")
            .is_none()
    );
}

#[test]
fn review_and_favorite_use_the_skill_service_boundary() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("SKILL.md"), "# Review skill\nText\n").unwrap();
    let service = SkillService::new(Database::open_in_memory().unwrap().into_skill_repository());
    let collected = service
        .collect_local_folder(directory.path().to_path_buf())
        .unwrap();

    service
        .review(
            &collected.id,
            SkillReviewInput {
                status: "approved".to_owned(),
                notes: Some("reviewed".to_owned()),
            },
        )
        .unwrap();
    service.set_favorite(&collected.id, true).unwrap();

    let detail = service.get(&collected.id).unwrap().unwrap();
    assert_eq!(detail.review_status, "approved");
    assert_eq!(detail.review_notes.as_deref(), Some("reviewed"));
    assert!(detail.favorite);
}
