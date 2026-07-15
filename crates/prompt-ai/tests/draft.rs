use prompt_ai::{AiDraft, DraftDestination};
use time::macros::datetime;

#[test]
fn ai_results_are_always_inbox_drafts() {
    let draft = AiDraft::new(
        "代码审查",
        "审查当前变更",
        "gpt-5",
        "根据用户的代码审查需求生成",
        datetime!(2026-07-15 00:00 UTC),
    )
    .unwrap();

    assert_eq!(draft.destination(), DraftDestination::Inbox);
}
