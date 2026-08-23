use std::fs;

use prompt_store::{Database, LATEST_SCHEMA_VERSION};
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn initializes_and_reopens_the_latest_schema_idempotently() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("prompt-hub.db");

    let first = Database::open(&path).expect("new database should migrate");
    assert_eq!(first.schema_version().unwrap(), LATEST_SCHEMA_VERSION);
    assert!(first.migration_report().backup_path().is_none());
    drop(first);

    let second = Database::open(&path).expect("reopening must be idempotent");
    assert_eq!(second.schema_version().unwrap(), LATEST_SCHEMA_VERSION);
    assert!(second.migration_report().backup_path().is_none());
}

#[test]
fn creates_a_verified_backup_before_upgrading_an_existing_database() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("legacy.db");
    let legacy = Connection::open(&path).unwrap();
    legacy
        .execute_batch(
            "PRAGMA user_version = 0;
             CREATE TABLE legacy_marker(value TEXT NOT NULL);
             INSERT INTO legacy_marker(value) VALUES ('keep-me');",
        )
        .unwrap();
    drop(legacy);

    let database = Database::open(&path).expect("legacy database should migrate");
    let backup = database
        .migration_report()
        .backup_path()
        .expect("existing database needs a pre-migration backup");

    assert!(backup.exists());
    assert!(fs::metadata(backup).unwrap().len() > 0);
    let backup_database = Connection::open(backup).unwrap();
    let marker: String = backup_database
        .query_row("SELECT value FROM legacy_marker", [], |row| row.get(0))
        .unwrap();
    assert_eq!(marker, "keep-me");
}

#[test]
fn upgrades_a_v2_database_with_the_favorites_migration() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("v2.db");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(include_str!("../migrations/0001_initial.sql"))
        .unwrap();
    connection
        .execute_batch(include_str!("../migrations/0002_search.sql"))
        .unwrap();
    connection
        .execute_batch("PRAGMA user_version = 2;")
        .unwrap();
    drop(connection);

    let database = Database::open(&path).unwrap();
    assert_eq!(database.schema_version().unwrap(), LATEST_SCHEMA_VERSION);
    let backup = database.migration_report().backup_path().unwrap();
    assert!(backup.exists());
    let upgraded = Connection::open(&path).unwrap();
    let exists: i64 = upgraded
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'prompt_favorites'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(exists, 1);
}

#[test]
fn upgrades_a_v3_database_with_import_job_item_tracking() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("v3.db");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(include_str!("../migrations/0001_initial.sql"))
        .unwrap();
    connection
        .execute_batch(include_str!("../migrations/0002_search.sql"))
        .unwrap();
    connection
        .execute_batch(include_str!("../migrations/0003_favorites.sql"))
        .unwrap();
    connection
        .execute_batch("PRAGMA user_version = 3;")
        .unwrap();
    drop(connection);

    let database = Database::open(&path).unwrap();
    assert_eq!(database.schema_version().unwrap(), LATEST_SCHEMA_VERSION);
    let upgraded = Connection::open(&path).unwrap();
    let item_table_exists: i64 = upgraded
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'import_job_items'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(item_table_exists, 1);
}

#[test]
fn upgrades_a_v4_database_with_skill_asset_tables() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("v4.db");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(include_str!("../migrations/0001_initial.sql"))
        .unwrap();
    connection
        .execute_batch(include_str!("../migrations/0002_search.sql"))
        .unwrap();
    connection
        .execute_batch(include_str!("../migrations/0003_favorites.sql"))
        .unwrap();
    connection
        .execute_batch(include_str!("../migrations/0004_import_jobs.sql"))
        .unwrap();
    connection
        .execute_batch("PRAGMA user_version = 4;")
        .unwrap();
    drop(connection);

    let database = Database::open(&path).unwrap();
    assert_eq!(database.schema_version().unwrap(), LATEST_SCHEMA_VERSION);
    let upgraded = Connection::open(&path).unwrap();
    let table_exists: i64 = upgraded
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'skills'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(table_exists, 1);
}

#[test]
fn upgrades_a_v5_database_with_skill_snapshot_tracking() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("v5.db");
    let connection = Connection::open(&path).unwrap();
    for migration in [
        include_str!("../migrations/0001_initial.sql"),
        include_str!("../migrations/0002_search.sql"),
        include_str!("../migrations/0003_favorites.sql"),
        include_str!("../migrations/0004_import_jobs.sql"),
        include_str!("../migrations/0005_skills.sql"),
    ] {
        connection.execute_batch(migration).unwrap();
    }
    connection
        .execute_batch("PRAGMA user_version = 5;")
        .unwrap();
    drop(connection);

    let database = Database::open(&path).unwrap();
    assert_eq!(database.schema_version().unwrap(), LATEST_SCHEMA_VERSION);
    let upgraded = Connection::open(&path).unwrap();
    let snapshot_column: String = upgraded
        .query_row(
            "SELECT name FROM pragma_table_info('skills') WHERE name = 'snapshot_path'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(snapshot_column, "snapshot_path");
}

#[test]
fn upgrades_the_legacy_v5_prompt_usage_schema_without_losing_usage_data() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("legacy-v5-prompt-usage.db");
    let connection = Connection::open(&path).unwrap();
    for migration in [
        include_str!("../migrations/0001_initial.sql"),
        include_str!("../migrations/0002_search.sql"),
        include_str!("../migrations/0003_favorites.sql"),
        include_str!("../migrations/0004_import_jobs.sql"),
    ] {
        connection.execute_batch(migration).unwrap();
    }
    connection
        .execute_batch(
            "ALTER TABLE prompts ADD COLUMN last_used_at INTEGER;
             INSERT INTO prompts(
                id, status, effectiveness, current_version, entity_json, created_at, updated_at, deleted_at
             ) VALUES ('legacy-prompt', 'inbox', 'unverified', 1, '{}', 1, 1, NULL);
             UPDATE prompts SET last_used_at = 42 WHERE id = 'legacy-prompt';
             PRAGMA user_version = 5;",
        )
        .unwrap();
    drop(connection);

    let database = Database::open(&path)
        .expect("legacy prompt-usage v5 database should upgrade to the skill schema");
    assert!(
        database
            .migration_ledger()
            .unwrap()
            .iter()
            .any(|entry| entry.migration_id() == "legacy/0.1.2-prompt-usage")
    );
    assert_eq!(database.schema_version().unwrap(), LATEST_SCHEMA_VERSION);
    drop(database);

    let upgraded = Connection::open(&path).unwrap();
    let skills_table_exists: i64 = upgraded
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'skills'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(skills_table_exists, 1);
    let last_used_at: i64 = upgraded
        .query_row(
            "SELECT last_used_at FROM prompts WHERE id = 'legacy-prompt'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(last_used_at, 42);
}

#[test]
fn records_an_immutable_migration_ledger_for_a_fresh_database() {
    let database = Database::open_in_memory().expect("fresh database should create a ledger");
    let ledger = database
        .migration_ledger()
        .expect("fresh database should expose its migration ledger");

    assert!(
        ledger
            .iter()
            .any(|entry| entry.migration_id() == "legacy/0001-initial")
    );
    assert!(
        ledger
            .iter()
            .any(|entry| entry.migration_id() == "20260824_01_migration_ledger")
    );
    assert!(
        ledger
            .iter()
            .all(|entry| !entry.checksum_sha256().is_empty())
    );
}

#[test]
fn rejects_a_changed_checksum_before_executing_new_sql() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("checksum-conflict.db");
    let database = Database::open(&path).expect("database should initialize");
    drop(database);

    let connection = Connection::open(&path).unwrap();
    connection
        .execute(
            "UPDATE migration_ledger SET checksum_sha256 = 'tampered' WHERE migration_id = ?1",
            ["20260824_01_migration_ledger"],
        )
        .unwrap();
    drop(connection);

    let error = match Database::open(&path) {
        Ok(_) => panic!("tampered ledger must fail closed"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("migration checksum conflict"));
}

#[test]
fn rejects_an_unknown_migration_id_before_opening_the_database() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("unknown-migration.db");
    drop(Database::open(&path).unwrap());

    let connection = Connection::open(&path).unwrap();
    connection
        .execute(
            "INSERT INTO migration_ledger(migration_id, checksum_sha256, applied_at, provenance)
             VALUES ('future/unknown', 'abc', 1, 'canonical')",
            [],
        )
        .unwrap();
    drop(connection);

    let error = match Database::open(&path) {
        Ok(_) => panic!("unknown migration must fail closed"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("unknown migration id"));
}

#[test]
fn restores_a_legacy_backup_only_after_migrating_it_to_the_current_schema() {
    let directory = tempdir().unwrap();
    let legacy_path = directory.path().join("legacy-v4.db");
    let legacy = Connection::open(&legacy_path).unwrap();
    for migration in [
        include_str!("../migrations/0001_initial.sql"),
        include_str!("../migrations/0002_search.sql"),
        include_str!("../migrations/0003_favorites.sql"),
        include_str!("../migrations/0004_import_jobs.sql"),
    ] {
        legacy.execute_batch(migration).unwrap();
    }
    legacy.execute_batch("PRAGMA user_version = 4;").unwrap();
    drop(legacy);

    let current_path = directory.path().join("current.db");
    let mut repository = Database::open(&current_path).unwrap().into_repository();
    repository.restore_from_backup(&legacy_path).unwrap();
    drop(repository);

    let restored = Connection::open(&current_path).unwrap();
    let version: u32 = restored
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    let has_skills_table: i64 = restored
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'skills'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, LATEST_SCHEMA_VERSION);
    assert_eq!(has_skills_table, 1);
}

#[test]
fn rejects_an_ambiguous_nonzero_schema_without_writing_new_tables() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("ambiguous.db");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL) STRICT;
             PRAGMA user_version = 5;",
        )
        .unwrap();
    drop(connection);

    let error = match Database::open(&path) {
        Ok(_) => panic!("ambiguous schema must fail closed"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("not a supported Prompt Hub history")
    );
    let unchanged = Connection::open(&path).unwrap();
    let skills: i64 = unchanged
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'skills'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(skills, 0);
}
