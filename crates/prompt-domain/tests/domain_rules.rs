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
