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
fn latest_schema_keeps_source_evidence_and_usage_in_dedicated_tables() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("v8-metadata.db");
    drop(Database::open(&path).expect("fresh database should migrate"));
    let connection = Connection::open(path).unwrap();
    let source_columns: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('prompt_sources') WHERE name IN ('raw_excerpt', 'import_job_id')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let usage_table: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'prompt_usage'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(source_columns, 2);
    assert_eq!(usage_table, 1);
}

#[test]
fn upgrades_a_v7_database_to_the_metadata_and_usage_schema() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("v7-to-v8.db");
    drop(Database::open(&path).unwrap());
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "DROP INDEX idx_prompt_usage_last_used_at;
             DROP TABLE prompt_usage;
             ALTER TABLE prompt_sources DROP COLUMN raw_excerpt;
             ALTER TABLE prompt_sources DROP COLUMN import_job_id;
             ALTER TABLE prompts DROP COLUMN imported_at;
             ALTER TABLE prompts DROP COLUMN last_validated_at;
             DELETE FROM migration_ledger WHERE migration_id = '20260829_01_prompt_metadata_and_usage';
             DELETE FROM schema_migrations WHERE version = 8;
             PRAGMA user_version = 7;",
        )
        .unwrap();
    drop(connection);
    let upgraded = Database::open(&path).unwrap();
    assert_eq!(upgraded.schema_version().unwrap(), LATEST_SCHEMA_VERSION);
    drop(upgraded);
    let reopened = Database::open(&path)
        .expect("an upgraded v7 database must reopen with a complete migration ledger");
    assert_eq!(reopened.schema_version().unwrap(), LATEST_SCHEMA_VERSION);
}

#[test]
fn reopening_the_latest_schema_is_read_only_during_another_write_transaction() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("latest-read-only-open.db");
    drop(Database::open(&path).unwrap());

    let writer = Connection::open(&path).unwrap();
    writer
        .execute_batch("PRAGMA journal_mode = WAL; BEGIN IMMEDIATE;")
        .unwrap();

    let reopened = Database::open(&path)
        .expect("opening a complete latest schema must not compete for a write lock");
    assert_eq!(reopened.schema_version().unwrap(), LATEST_SCHEMA_VERSION);

    writer.execute_batch("ROLLBACK;").unwrap();
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
fn pre_migration_backup_includes_committed_rows_still_in_the_wal() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("wal-legacy.db");
    let writer = Connection::open(&path).unwrap();
    for migration in [
        include_str!("../migrations/0001_initial.sql"),
        include_str!("../migrations/0002_search.sql"),
        include_str!("../migrations/0003_favorites.sql"),
        include_str!("../migrations/0004_import_jobs.sql"),
    ] {
        writer.execute_batch(migration).unwrap();
    }
    writer
        .execute_batch(
            "PRAGMA user_version = 4;
             PRAGMA journal_mode = WAL;
             PRAGMA wal_autocheckpoint = 0;
             INSERT INTO prompts(
                id, status, effectiveness, current_version, entity_json, created_at, updated_at, deleted_at
             ) VALUES ('wal-prompt', 'inbox', 'unverified', 1, '{}', 1, 1, NULL);",
        )
        .unwrap();

    let database = Database::open(&path).expect("a WAL database should migrate safely");
    let backup = database
        .migration_report()
        .backup_path()
        .expect("the WAL database needs a pre-migration backup");
    let backup_database = Connection::open(backup).unwrap();
    let preserved: i64 = backup_database
        .query_row(
            "SELECT COUNT(*) FROM prompts WHERE id = 'wal-prompt'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(preserved, 1);
    drop(writer);
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
    let migrated_usage: (i64, i64) = upgraded
        .query_row(
            "SELECT use_count, last_used_at FROM prompt_usage WHERE prompt_id = 'legacy-prompt'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(migrated_usage, (1, 42));
    drop(upgraded);

    let reopened = Database::open(&path)
        .expect("a recovered legacy prompt-usage database must reopen normally");
    assert_eq!(reopened.schema_version().unwrap(), LATEST_SCHEMA_VERSION);
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
fn rejects_a_latest_version_database_with_an_empty_ledger() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("empty-ledger.db");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE migration_ledger(
                migration_id TEXT PRIMARY KEY,
                checksum_sha256 TEXT NOT NULL,
                applied_at INTEGER NOT NULL,
                provenance TEXT NOT NULL
             ) STRICT;
             PRAGMA user_version = 8;",
        )
        .unwrap();
    drop(connection);

    let error = match Database::open(&path) {
        Ok(_) => panic!("an empty committed ledger must fail closed"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("migration ledger is incomplete"));
}

#[test]
fn rejects_a_latest_version_database_with_a_missing_canonical_ledger_entry() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("incomplete-ledger.db");
    drop(Database::open(&path).unwrap());

    let connection = Connection::open(&path).unwrap();
    connection
        .execute(
            "DELETE FROM migration_ledger WHERE migration_id = ?1",
            ["legacy/0004-import-jobs"],
        )
        .unwrap();
    drop(connection);

    let error = match Database::open(&path) {
        Ok(_) => panic!("a missing canonical ledger entry must fail closed"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("migration ledger is incomplete"));
}

#[test]
fn repairs_a_known_missing_metadata_ledger_entry_without_rewriting_data() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("missing-metadata-ledger.db");
    let database = Database::open(&path).unwrap();
    drop(database);

    let connection = Connection::open(&path).unwrap();
    connection
        .execute(
            "DELETE FROM migration_ledger WHERE migration_id = ?1",
            ["20260829_01_prompt_metadata_and_usage"],
        )
        .unwrap();
    drop(connection);

    let reopened =
        Database::open(&path).expect("the known v8 ledger omission should be repaired safely");
    assert!(
        reopened
            .migration_ledger()
            .unwrap()
            .iter()
            .any(|entry| entry.migration_id() == "20260829_01_prompt_metadata_and_usage")
    );
}

#[test]
fn rejects_a_database_from_a_future_schema_version() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("future.db");
    drop(Database::open(&path).unwrap());

    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch("PRAGMA user_version = 9;")
        .unwrap();
    drop(connection);

    let error = match Database::open(&path) {
        Ok(_) => panic!("a future schema version must fail closed"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("newer than this application supports")
    );
}

#[test]
fn rejects_a_latest_schema_missing_a_required_table_without_recreating_it() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("missing-table.db");
    drop(Database::open(&path).unwrap());

    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch("DROP TABLE prompt_favorites;")
        .unwrap();
    drop(connection);

    let error = match Database::open(&path) {
        Ok(_) => panic!("a latest schema missing a required table must fail closed"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("required table prompt_favorites is missing")
    );

    let unchanged = Connection::open(&path).unwrap();
    let favorites: i64 = unchanged
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'prompt_favorites'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(favorites, 0);
}

#[test]
fn rejects_a_latest_schema_missing_a_critical_column() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("missing-column.db");
    drop(Database::open(&path).unwrap());

    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch("ALTER TABLE skills DROP COLUMN snapshot_path;")
        .unwrap();
    drop(connection);

    let error = match Database::open(&path) {
        Ok(_) => panic!("a latest schema missing a critical column must fail closed"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("required column skills.snapshot_path is missing")
    );
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
