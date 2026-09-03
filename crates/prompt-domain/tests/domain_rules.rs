use prompt_domain::{
    Actor, Compatibility, CompatibilityStatus, DomainError, EffectivenessStatus, Prompt,
    PromptContent, PromptSource, SourceKind,
};
use time::macros::datetime;

fn valid_content(body: &str) -> PromptContent {
    PromptContent::new(
        "代码审查助手",
        body,
        Some("检查代码缺陷并给出证据".to_owned()),
        Some("开发".to_owned()),
        vec!["代码审查".to_owned()],
    )
    .expect("fixture content should be valid")
}

fn manual_source() -> PromptSource {
    PromptSource::new(
        SourceKind::Manual,
        "手动录入",
        None,
        datetime!(2026-07-15 00:00 UTC),
    )
    .expect("fixture source should be valid")
}

#[test]
fn publishing_requires_a_category_or_tag() {
    let content = PromptContent::new("标题", "正文", None, None, Vec::new())
        .expect("inbox content may be unclassified");
    let mut prompt = Prompt::new_inbox(
        content,
        manual_source(),
        Actor::User,
        datetime!(2026-07-15 00:00 UTC),
    );

    let error = prompt
        .publish(
            EffectivenessStatus::Unverified,
            datetime!(2026-07-15 00:01 UTC),
        )
        .expect_err("unclassified prompt must not be published");

    assert_eq!(error, DomainError::ClassificationRequired);
    assert!(prompt.is_inbox());
}

#[test]
fn mcp_may_create_an_inbox_draft_but_cannot_modify_a_published_prompt() {
    let mut prompt = Prompt::new_inbox(
        valid_content("检查当前代码"),
        PromptSource::new(
            SourceKind::Mcp,
            "Codex MCP",
            None,
            datetime!(2026-07-15 00:00 UTC),
        )
        .expect("MCP source should be valid"),
        Actor::Mcp,
        datetime!(2026-07-15 00:00 UTC),
    );

    assert!(prompt.is_inbox());
    prompt
        .publish(
            EffectivenessStatus::Unverified,
            datetime!(2026-07-15 00:01 UTC),
        )
        .expect("user-reviewed fixture may be published");

    let error = prompt
        .revise(
            valid_content("尝试覆盖正式内容"),
            Actor::Mcp,
            datetime!(2026-07-15 00:02 UTC),
        )
        .expect_err("MCP must not modify published prompts");

    assert_eq!(error, DomainError::ExternalWriteToPublishedPrompt);
    assert_eq!(prompt.current_version().content().body(), "检查当前代码");
}

#[test]
fn restoring_history_creates_a_new_version_instead_of_rewinding() {
    let mut prompt = Prompt::new_inbox(
        valid_content("第一版"),
        manual_source(),
        Actor::User,
        datetime!(2026-07-15 00:00 UTC),
    );
    let first = prompt.current_version().clone();

    prompt
        .revise(
            valid_content("第二版"),
            Actor::User,
            datetime!(2026-07-15 00:01 UTC),
        )
        .expect("user may revise an inbox prompt");
    let restored = prompt
        .restore(&first, Actor::User, datetime!(2026-07-15 00:02 UTC))
        .expect("user may restore history");

    assert_eq!(restored.number(), 3);
    assert_eq!(restored.content().body(), "第一版");
    assert_eq!(prompt.current_version().number(), 3);
}

#[test]
fn confirmed_compatibility_requires_a_confirmation_time() {
    let error = Compatibility::new(
        "Codex",
        Some("gpt-5.6-sol".to_owned()),
        CompatibilityStatus::Confirmed,
        None,
        None,
    )
    .expect_err("confirmed compatibility requires evidence time");

    assert_eq!(error, DomainError::CompatibilityConfirmationRequired);
}

#[test]
fn provenance_source_keeps_excerpt_and_import_job_outside_prompt_content() {
    let source = PromptSource::with_provenance(
        SourceKind::FileImport,
        "文件导入",
        Some("C:/提示词/会议.md".to_owned()),
        datetime!(2026-08-29 08:00 UTC),
        Some("原文的短摘录".to_owned()),
        Some("import-job-1".to_owned()),
    )
    .expect("valid source evidence should be accepted");

    assert_eq!(source.raw_excerpt(), Some("原文的短摘录"));
    assert_eq!(source.import_job_id(), Some("import-job-1"));
}

#[test]
fn provenance_source_rejects_unbounded_excerpt_data() {
    let error = PromptSource::with_provenance(
        SourceKind::FileImport,
        "文件导入",
        Some("C:/提示词/导入.md".to_owned()),
        datetime!(2026-08-29 08:00 UTC),
        Some("x".repeat(4097)),
        Some("import-job-1".to_owned()),
    )
    .expect_err("source evidence must have a bounded excerpt");

    assert_eq!(error, DomainError::SourceExcerptTooLong);
}

#[test]
fn imported_prompt_tracks_import_and_last_validation_times() {
    let imported_at = datetime!(2026-08-29 08:00 UTC);
    let mut prompt = Prompt::new_imported_inbox(
        valid_content("从文件导入的正文"),
        PromptSource::new(
            SourceKind::FileImport,
            "文件导入",
            Some("C:/提示词/导入.md".to_owned()),
            imported_at,
        )
        .unwrap(),
        Actor::User,
        imported_at,
    );
    let validated_at = datetime!(2026-08-29 09:00 UTC);
    prompt
        .record_validation(
            prompt_domain::ValidationRecord::new(
                EffectivenessStatus::Effective,
                Some(5),
                None,
                validated_at,
            )
            .unwrap(),
            Actor::User,
            validated_at,
        )
        .unwrap();

    assert_eq!(prompt.imported_at(), Some(imported_at));
    assert_eq!(prompt.last_validated_at(), Some(validated_at));
}

#[test]
fn public_enum_values_have_stable_snake_case_wire_names() {
    assert_eq!(
        serde_json::to_string(&EffectivenessStatus::NeedsRetest).unwrap(),
        "\"needs_retest\""
    );
    assert_eq!(
        serde_json::to_string(&SourceKind::AiGenerated).unwrap(),
        "\"ai_generated\""
    );
}

#[test]
fn user_can_archive_and_soft_delete_a_prompt_without_losing_its_current_version() {
    let mut prompt = Prompt::new_inbox(
        valid_content("保留可恢复的提示词"),
        manual_source(),
        Actor::User,
        datetime!(2026-07-15 00:00 UTC),
    );
    let version = prompt.current_version().number();

    prompt
        .archive(Actor::User, datetime!(2026-07-15 00:01 UTC))
        .expect("a user may archive a prompt");
    assert_eq!(prompt.status(), prompt_domain::PromptStatus::Archived);

    prompt
        .soft_delete(Actor::User, datetime!(2026-07-15 00:02 UTC))
        .expect("a user may soft-delete an archived prompt");
    assert_eq!(prompt.status(), prompt_domain::PromptStatus::Deleted);
    assert_eq!(prompt.current_version().number(), version);
}

#[test]
fn recovering_a_soft_deleted_prompt_returns_it_to_the_inbox_without_rewriting_history() {
    let mut prompt = Prompt::new_inbox(
        valid_content("可恢复的提示词"),
        manual_source(),
        Actor::User,
        datetime!(2026-07-15 00:00 UTC),
    );
    let version = prompt.current_version().number();
    prompt
        .soft_delete(Actor::User, datetime!(2026-07-15 00:01 UTC))
        .unwrap();

    prompt
        .recover(Actor::User, datetime!(2026-07-15 00:02 UTC))
        .expect("a user may recover a soft-deleted prompt");

    assert_eq!(prompt.status(), prompt_domain::PromptStatus::Inbox);
    assert_eq!(prompt.current_version().number(), version);
}

#[test]
fn metadata_revisions_create_a_new_version_snapshot() {
    let mut prompt = Prompt::new_inbox(
        valid_content("可追溯的提示词"),
        manual_source(),
        Actor::User,
        datetime!(2026-07-15 00:00 UTC),
    );
    prompt
        .record_validation(
            prompt_domain::ValidationRecord::new(
                EffectivenessStatus::Effective,
                Some(5),
                Some("已验证".to_owned()),
                datetime!(2026-07-15 00:01 UTC),
            )
            .unwrap(),
            Actor::User,
            datetime!(2026-07-15 00:01 UTC),
        )
        .unwrap();

    let version = prompt
        .revise_metadata(Actor::User, datetime!(2026-07-15 00:01 UTC))
        .expect("metadata changes should create a version");

    assert_eq!(version.number(), 2);
    assert_eq!(version.content().body(), "可追溯的提示词");
    assert_eq!(prompt.current_version().number(), 2);
}
