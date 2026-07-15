use prompt_hub_desktop_lib::commands::{ManualPromptDraft, PromptService};
use prompt_store::Database;
use time::macros::datetime;

#[test]
fn user_can_create_publish_edit_and_restore_a_prompt_through_the_service_boundary() {
    let service = PromptService::new(Database::open_in_memory().unwrap().into_repository());
    let created = service
        .create_manual_draft(
            ManualPromptDraft {
                title: "代码审查".to_owned(),
                body: "审查当前变更".to_owned(),
                description: None,
                category: Some("开发".to_owned()),
                tags: vec!["审查".to_owned()],
            },
            datetime!(2026-07-15 00:00 UTC),
        )
        .unwrap();
    let original = created.current_version().clone();

    service
        .publish(created.id(), datetime!(2026-07-15 00:01 UTC))
        .unwrap();
    service
        .revise(
            created.id(),
            ManualPromptDraft {
                title: "代码审查".to_owned(),
                body: "审查当前变更并给出证据".to_owned(),
                description: None,
                category: Some("开发".to_owned()),
                tags: vec!["审查".to_owned()],
            },
            datetime!(2026-07-15 00:02 UTC),
        )
        .unwrap();
    let restored = service
        .restore_version(
            created.id(),
            original.number(),
            datetime!(2026-07-15 00:03 UTC),
        )
        .unwrap();

    assert_eq!(restored.current_version().number(), 3);
    assert_eq!(restored.current_version().content().body(), "审查当前变更");
}

#[test]
fn service_lists_prompts_for_the_library() {
    let service = PromptService::new(Database::open_in_memory().unwrap().into_repository());
    let created = service
        .create_manual_draft(
            ManualPromptDraft {
                title: "本地资产".to_owned(),
                body: "可查询的正文".to_owned(),
                description: None,
                category: Some("开发".to_owned()),
                tags: vec!["本地".to_owned()],
            },
            datetime!(2026-07-15 00:00 UTC),
        )
        .unwrap();

    assert_eq!(service.list().unwrap(), vec![created]);
}
