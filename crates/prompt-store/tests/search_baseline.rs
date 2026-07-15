use std::time::Instant;

use prompt_store::{BackupDestination, Database, SearchFilters, SearchQuery, create_backup};
use rusqlite::{Connection, params};
use tempfile::tempdir;

fn seed_database(path: &std::path::Path, count: usize) {
    drop(Database::open(path).unwrap());
    let mut connection = Connection::open(path).unwrap();
    connection
        .execute_batch("PRAGMA foreign_keys = ON")
        .unwrap();
    let transaction = connection.transaction().unwrap();
    transaction
        .execute("INSERT INTO categories(name) VALUES ('基准')", [])
        .unwrap();
    for index in 0..count {
        let id = format!("{index:08x}-0000-0000-0000-000000000000");
        let title = format!("性能检索样本 {index}");
        let body = format!("第 {index} 条中文提示词用于性能检索和索引验证。");
        transaction.execute(
            "INSERT INTO prompts(id, status, effectiveness, current_version, entity_json, created_at, updated_at) VALUES (?1, 'inbox', 'unverified', 1, '{}', 0, 0)",
            [&id],
        ).unwrap();
        transaction.execute(
            "INSERT INTO prompt_versions(prompt_id, version_number, version_id, title, body, description, content_json, actor, created_at) VALUES (?1, 1, ?2, ?3, ?4, NULL, '{}', 'user', 0)",
            params![&id, format!("{index:08x}-0000-0000-0000-000000000001"), title, body],
        ).unwrap();
        transaction.execute(
            "INSERT INTO prompt_version_categories(prompt_id, version_number, category_id) VALUES (?1, 1, 1)",
            [&id],
        ).unwrap();
        transaction.execute(
            "INSERT INTO prompt_fts(prompt_id, title, body, description, tags, variables) VALUES (?1, ?2, ?3, '', '性能 检索', '')",
            params![&id, format!("性能检索样本 {index}"), format!("第 {index} 条中文提示词用于性能检索和索引验证。")],
        ).unwrap();
    }
    transaction.commit().unwrap();
}

#[test]
#[ignore = "explicit local performance baseline"]
fn reports_file_backed_search_rebuild_and_backup_baselines() {
    for count in [1_000, 10_000, 50_000] {
        let directory = tempdir().unwrap();
        let path = directory.path().join(format!("search-{count}.db"));
        seed_database(&path, count);
        let database = Database::open(&path).unwrap();
        let mut repository = database.into_repository();

        let cold_started = Instant::now();
        let cold = repository
            .search(SearchQuery::new("性能检索").with_page(20, 0))
            .unwrap();
        let cold_elapsed = cold_started.elapsed();
        let warm_started = Instant::now();
        let warm = repository
            .search(SearchQuery::new("性能检索").with_page(20, 0))
            .unwrap();
        let warm_elapsed = warm_started.elapsed();
        let filtered_started = Instant::now();
        let filtered = repository
            .search(SearchQuery::new("性能检索").with_filters(SearchFilters {
                category: Some("基准".to_owned()),
                ..SearchFilters::default()
            }))
            .unwrap();
        let filtered_elapsed = filtered_started.elapsed();
        let rebuild_started = Instant::now();
        repository.rebuild_search_index().unwrap();
        let rebuild_elapsed = rebuild_started.elapsed();
        let backup_started = Instant::now();
        let backup = create_backup(&path, BackupDestination::Manual).unwrap();
        let backup_elapsed = backup_started.elapsed();

        assert_eq!(cold.total, count as u64);
        assert_eq!(warm.total, count as u64);
        assert_eq!(filtered.total, count as u64);
        assert!(backup.path().exists());
        println!(
            "PERF count={count} cold_us={} warm_us={} filtered_us={} rebuild_us={} backup_us={}",
            cold_elapsed.as_micros(),
            warm_elapsed.as_micros(),
            filtered_elapsed.as_micros(),
            rebuild_elapsed.as_micros(),
            backup_elapsed.as_micros(),
        );
    }
}
