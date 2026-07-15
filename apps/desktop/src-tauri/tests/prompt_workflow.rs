use prompt_hub_desktop_lib::commands::{ManualPromptDraft, PromptService};
use prompt_store::{Database, SearchQuery};
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
                variables: vec![],
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
                variables: vec![],
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
fn service_exposes_ordered_version_history_for_comparison() {
    let service = PromptService::new(Database::open_in_memory().unwrap().into_repository());
    let created = service
        .create_manual_draft(
            ManualPromptDraft {
                title: "历史记录".to_owned(),
                body: "第一版".to_owned(),
                description: None,
                category: Some("开发".to_owned()),
                tags: vec![],
                variables: vec![],
            },
            datetime!(2026-07-15 00:00 UTC),
        )
        .unwrap();
    service
        .revise(
            created.id(),
            ManualPromptDraft {
                title: "历史记录".to_owned(),
                body: "第二版".to_owned(),
                description: None,
                category: Some("开发".to_owned()),
                tags: vec![],
                variables: vec![],
            },
            datetime!(2026-07-15 00:01 UTC),
        )
        .unwrap();

    let history = service.history(created.id()).unwrap();
    assert_eq!(
        history
            .iter()
            .map(|version| version.number())
            .collect::<Vec<_>>(),
        [1, 2]
    );
}

#[test]
fn service_searches_the_local_prompt_library() {
    let service = PromptService::new(Database::open_in_memory().unwrap().into_repository());
    service
        .create_manual_draft(
            ManualPromptDraft {
                title: "代码审查".to_owned(),
                body: "审查当前变更".to_owned(),
                description: None,
                category: Some("开发".to_owned()),
                tags: vec![],
                variables: vec![],
            },
            datetime!(2026-07-15 00:00 UTC),
        )
        .unwrap();

    let page = service.search(SearchQuery::new("代码审查")).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.hits[0].title, "代码审查");
}

#[test]
fn service_preserves_typed_variables_from_a_manual_draft() {
    let service = PromptService::new(Database::open_in_memory().unwrap().into_repository());
    let draft: ManualPromptDraft = serde_json::from_value(serde_json::json!({
        "title": "代码审查",
        "body": "审查 {{language}} 的变更",
        "description": null,
        "category": "开发",
        "tags": ["审查"],
        "variables": [{
            "name": "language",
            "kind": "text",
            "description": "目标编程语言",
            "defaultValue": "Rust",
            "required": true
        }]
    }))
    .unwrap();

    let created = service
        .create_manual_draft(draft, datetime!(2026-07-15 00:00 UTC))
        .unwrap();
    let variable = &created.current_version().content().variables()[0];

    assert_eq!(variable.name(), "language");
    assert_eq!(variable.default_value(), Some("Rust"));
    assert!(variable.required());
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
                variables: vec![],
            },
            datetime!(2026-07-15 00:00 UTC),
        )
        .unwrap();

    assert_eq!(service.list().unwrap(), vec![created]);
}

#[test]
fn user_can_archive_soft_delete_and_recover_through_the_service_boundary() {
    let service = PromptService::new(Database::open_in_memory().unwrap().into_repository());
    let created = service
        .create_manual_draft(
            ManualPromptDraft {
                title: "可恢复资产".to_owned(),
                body: "保留正文".to_owned(),
                description: None,
                category: Some("开发".to_owned()),
                tags: vec!["恢复".to_owned()],
                variables: vec![],
            },
            datetime!(2026-07-15 00:00 UTC),
        )
        .unwrap();

    assert_eq!(
        service
            .archive(created.id(), datetime!(2026-07-15 00:01 UTC))
            .unwrap()
            .status(),
        prompt_domain::PromptStatus::Archived
    );
    assert_eq!(
        service
            .soft_delete(created.id(), datetime!(2026-07-15 00:02 UTC))
            .unwrap()
            .status(),
        prompt_domain::PromptStatus::Deleted
    );
    assert_eq!(
        service
            .recover(created.id(), datetime!(2026-07-15 00:03 UTC))
            .unwrap()
            .status(),
        prompt_domain::PromptStatus::Inbox
    );
}
