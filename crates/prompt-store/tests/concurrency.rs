use std::sync::{Arc, Barrier};
use std::thread;

use prompt_domain::{Actor, AuditAction, Prompt, PromptContent, PromptSource, SourceKind};
use prompt_store::{BackupDestination, Database, SearchQuery, create_backup};
use tempfile::tempdir;
use time::macros::datetime;

fn prompt(index: usize) -> Prompt {
    let at = datetime!(2026-07-15 00:00 UTC);
    Prompt::new_inbox(
        PromptContent::new(
            format!("并发提示词 {index}"),
            format!("第 {index} 条用于验证并发检索与写入。"),
            None,
            Some("并发".to_owned()),
            vec!["并发".to_owned()],
        )
        .unwrap(),
        PromptSource::new(SourceKind::Manual, "并发测试", None, at).unwrap(),
        Actor::User,
        at,
    )
}

#[test]
fn permits_concurrent_search_write_and_backup_on_a_file_database() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("concurrency.db");
    let mut repository = Database::open(&path).unwrap().into_repository();
    repository.save(&prompt(0), AuditAction::Created).unwrap();
    drop(repository);

    let start = Arc::new(Barrier::new(3));
    let writer_path = path.clone();
    let writer_start = Arc::clone(&start);
    let writer = thread::spawn(move || {
        writer_start.wait();
        let mut repository = Database::open(writer_path).unwrap().into_repository();
        for index in 1..=20 {
            repository
                .save(&prompt(index), AuditAction::Created)
                .unwrap();
        }
    });
    let reader_path = path.clone();
    let reader_start = Arc::clone(&start);
    let reader = thread::spawn(move || {
        reader_start.wait();
        let repository = Database::open(reader_path).unwrap().into_repository();
        for _ in 0..20 {
            repository.search(SearchQuery::new("并发检索")).unwrap();
        }
    });

    start.wait();
    let backup = create_backup(&path, BackupDestination::Manual).unwrap();
    writer.join().unwrap();
    reader.join().unwrap();

    assert!(backup.path().exists());
    let repository = Database::open(&path).unwrap().into_repository();
    assert_eq!(repository.list().unwrap().len(), 21);
}
