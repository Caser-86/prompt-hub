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
