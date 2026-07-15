use prompt_domain::{
    Actor, AuditAction, EffectivenessStatus, Prompt, PromptContent, PromptSource, SourceKind,
};
use prompt_store::{Database, StoreError};
use rusqlite::Connection;
use tempfile::tempdir;
use time::macros::datetime;

fn prompt(body: &str) -> Prompt {
    Prompt::new_inbox(
        PromptContent::new(
            "代码审查助手",
            body,
            Some("检查代码并给出证据".to_owned()),
            Some("开发".to_owned()),
            vec!["代码审查".to_owned()],
        )
        .unwrap(),
        PromptSource::new(
            SourceKind::Manual,
            "手动录入",
            None,
            datetime!(2026-07-15 00:00 UTC),
        )
        .unwrap(),
        Actor::User,
        datetime!(2026-07-15 00:00 UTC),
    )
}

#[test]
fn saves_and_loads_a_prompt_with_its_provenance() {
    let database = Database::open_in_memory().unwrap();
    let mut repository = database.into_repository();
    let prompt = prompt("检查当前变更");

    repository
        .save(&prompt, AuditAction::Created)
        .expect("new prompt should save");
    let loaded = repository
        .get(prompt.id())
        .expect("query should succeed")
        .expect("prompt should exist");

    assert_eq!(loaded, prompt);
    assert_eq!(loaded.sources().len(), 1);
}

#[test]
fn rolls_back_version_and_current_pointer_when_audit_insert_fails() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("rollback.db");
    let database = Database::open(&path).unwrap();
    let mut repository = database.into_repository();
    let mut prompt = prompt("第一版");
    repository.save(&prompt, AuditAction::Created).unwrap();
    drop(repository);

    let raw = Connection::open(&path).unwrap();
    raw.execute_batch(
        "CREATE TRIGGER reject_audit
         BEFORE INSERT ON audit_events
         BEGIN
           SELECT RAISE(ABORT, 'injected audit failure');
         END;",
    )
    .unwrap();
    drop(raw);

    prompt
        .revise(
            PromptContent::new(
                "代码审查助手",
                "第二版",
                None,
                Some("开发".to_owned()),
                vec!["代码审查".to_owned()],
            )
            .unwrap(),
            Actor::User,
            datetime!(2026-07-15 00:01 UTC),
        )
        .unwrap();

    let database = Database::open(&path).unwrap();
    let mut repository = database.into_repository();
    let error = repository
        .save(&prompt, AuditAction::Revised)
        .expect_err("injected audit failure should abort the transaction");
    assert!(matches!(error, StoreError::Sqlite(_)));
    drop(repository);

    let raw = Connection::open(&path).unwrap();
    let version_count: i64 = raw
        .query_row("SELECT COUNT(*) FROM prompt_versions", [], |row| row.get(0))
        .unwrap();
    let current_number: i64 = raw
        .query_row("SELECT current_version FROM prompts", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version_count, 1);
    assert_eq!(current_number, 1);
}

#[test]
fn persists_publication_state_without_creating_a_duplicate_version() {
    let database = Database::open_in_memory().unwrap();
    let mut repository = database.into_repository();
    let mut prompt = prompt("待发布");
    repository.save(&prompt, AuditAction::Created).unwrap();

    prompt
        .publish(
            EffectivenessStatus::Unverified,
            datetime!(2026-07-15 00:01 UTC),
        )
        .unwrap();
    repository.save(&prompt, AuditAction::Published).unwrap();

    let loaded = repository.get(prompt.id()).unwrap().unwrap();
    assert_eq!(loaded, prompt);
    assert_eq!(repository.version_count(prompt.id()).unwrap(), 1);
}

#[test]
fn persists_soft_deletion_with_its_recovery_timestamp() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("soft-delete.db");
    let database = Database::open(&path).unwrap();
    let mut repository = database.into_repository();
    let mut prompt = prompt("待删除");
    repository.save(&prompt, AuditAction::Created).unwrap();

    let deleted_at = datetime!(2026-07-15 00:03 UTC);
    prompt.soft_delete(Actor::User, deleted_at).unwrap();
    repository.save(&prompt, AuditAction::Deleted).unwrap();

    drop(repository);
    let raw = Connection::open(&path).unwrap();
    let deleted_at_in_store: Option<i64> = raw
        .query_row(
            "SELECT deleted_at FROM prompts WHERE id = ?1",
            [prompt.id().value().to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(deleted_at_in_store, Some(deleted_at.unix_timestamp()));
}

#[test]
fn lists_prompts_by_most_recent_update_for_the_library() {
    let database = Database::open_in_memory().unwrap();
    let mut repository = database.into_repository();
    let older = prompt("较早提示词");
    let mut newer = prompt("较新提示词");
    repository.save(&older, AuditAction::Created).unwrap();
    repository.save(&newer, AuditAction::Created).unwrap();
    newer
        .archive(Actor::User, datetime!(2026-07-15 00:01 UTC))
        .unwrap();
    repository.save(&newer, AuditAction::Archived).unwrap();

    let prompts = repository.list().unwrap();

    assert_eq!(prompts, vec![newer, older]);
}

#[test]
fn returns_immutable_version_history_in_ascending_order() {
    let database = Database::open_in_memory().unwrap();
    let mut repository = database.into_repository();
    let mut prompt = prompt("第一版");
    repository.save(&prompt, AuditAction::Created).unwrap();
    prompt
        .revise(
            PromptContent::new(
                "代码审查助手",
                "第二版",
                None,
                Some("开发".to_owned()),
                vec!["代码审查".to_owned()],
            )
            .unwrap(),
            Actor::User,
            datetime!(2026-07-15 00:01 UTC),
        )
        .unwrap();
    repository.save(&prompt, AuditAction::Revised).unwrap();

    let history = repository.history(prompt.id()).unwrap();

    assert_eq!(history.len(), 2);
    assert_eq!(history[0].number(), 1);
    assert_eq!(history[0].content().body(), "第一版");
    assert_eq!(history[1].number(), 2);
    assert_eq!(history[1].content().body(), "第二版");
}

#[test]
fn persists_favorite_state_without_mutating_prompt_versions() {
    let database = Database::open_in_memory().unwrap();
    let mut repository = database.into_repository();
    let prompt = prompt("收藏提示词");
    repository.save(&prompt, AuditAction::Created).unwrap();

    repository
        .set_favorite(prompt.id(), true, datetime!(2026-07-15 00:01 UTC))
        .unwrap();
    assert!(repository.is_favorite(prompt.id()).unwrap());
    assert_eq!(repository.version_count(prompt.id()).unwrap(), 1);

    repository
        .set_favorite(prompt.id(), false, datetime!(2026-07-15 00:02 UTC))
        .unwrap();
    assert!(!repository.is_favorite(prompt.id()).unwrap());
}

#[test]
fn persists_import_job_path_fingerprint_and_terminal_diagnostics() {
    let database = Database::open_in_memory().unwrap();
    let mut repository = database.into_repository();
    let started_at = datetime!(2026-07-15 00:00 UTC);
    let job = repository
        .start_import_job(
            "file_import",
            "C:/提示词/导入.md",
            Some("source-fingerprint"),
            started_at,
        )
        .unwrap();

    repository
        .record_import_job_item(
            job.id(),
            "C:/提示词/导入.md",
            Some("body-fingerprint"),
            Some("导入标题"),
            "imported",
            "[]",
            None,
            None,
            started_at,
        )
        .unwrap();
    repository
        .finish_import_job(
            job.id(),
            "completed",
            r#"{"imported":1,"skippedDuplicates":0,"failed":0}"#,
            datetime!(2026-07-15 00:01 UTC),
        )
        .unwrap();

    let stored = repository.import_job(job.id()).unwrap().unwrap();
    assert_eq!(stored.status(), "completed");
    assert_eq!(stored.source_path(), Some("C:/提示词/导入.md"));
    assert!(stored.completed_at().is_some());
    assert!(stored.diagnostics_json().contains("imported"));
}
