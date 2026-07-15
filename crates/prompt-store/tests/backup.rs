use prompt_store::{
    BackupDestination, create_backup, preview_restore, prune_backups, restore_backup,
};
use rusqlite::Connection;
use tempfile::tempdir;

fn database_with_marker(path: &std::path::Path, value: &str) {
    let connection = Connection::open(path).unwrap();
    connection
        .execute_batch("CREATE TABLE marker(value TEXT NOT NULL) STRICT;")
        .unwrap();
    connection
        .execute("INSERT INTO marker(value) VALUES (?1)", [value])
        .unwrap();
}

#[test]
fn creates_an_integrity_checked_backup_with_metadata() {
    let directory = tempdir().unwrap();
    let source = directory.path().join("library.db");
    database_with_marker(&source, "original");

    let backup = create_backup(&source, BackupDestination::Manual).unwrap();

    assert!(backup.path().exists());
    assert!(backup.byte_len() > 0);
    assert_eq!(backup.schema_version(), 0);
    assert_eq!(backup.destination(), BackupDestination::Manual);
}

#[test]
fn restore_preview_is_read_only_and_reports_backup_contents() {
    let directory = tempdir().unwrap();
    let source = directory.path().join("library.db");
    database_with_marker(&source, "original");
    let backup = create_backup(&source, BackupDestination::Manual).unwrap();

    let preview = preview_restore(backup.path(), &source).unwrap();

    assert!(preview.target_exists());
    assert_eq!(preview.backup_schema_version(), 0);
    let connection = Connection::open(&source).unwrap();
    let value: String = connection
        .query_row("SELECT value FROM marker", [], |row| row.get(0))
        .unwrap();
    assert_eq!(value, "original");
}

#[test]
fn restore_creates_a_pre_replacement_safety_backup() {
    let directory = tempdir().unwrap();
    let source = directory.path().join("library.db");
    database_with_marker(&source, "original");
    let backup = create_backup(&source, BackupDestination::Manual).unwrap();

    let connection = Connection::open(&source).unwrap();
    connection
        .execute("UPDATE marker SET value = 'changed'", [])
        .unwrap();
    drop(connection);

    let report = restore_backup(backup.path(), &source).unwrap();

    let restored = Connection::open(&source).unwrap();
    let value: String = restored
        .query_row("SELECT value FROM marker", [], |row| row.get(0))
        .unwrap();
    assert_eq!(value, "original");
    drop(restored);
    let safety = Connection::open(report.pre_replacement_backup().unwrap()).unwrap();
    let previous: String = safety
        .query_row("SELECT value FROM marker", [], |row| row.get(0))
        .unwrap();
    assert_eq!(previous, "changed");
}

#[test]
fn corrupt_backup_is_rejected_before_a_restore_can_start() {
    let directory = tempdir().unwrap();
    let corrupt = directory.path().join("corrupt.db");
    std::fs::write(&corrupt, b"not a sqlite database").unwrap();
    let target = directory.path().join("target.db");

    assert!(preview_restore(&corrupt, &target).is_err());
    assert!(restore_backup(&corrupt, &target).is_err());
    assert!(!target.exists());
}

#[test]
fn restores_an_open_repository_from_a_verified_backup() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("library.db");
    database_with_marker(&path, "original");
    let backup = create_backup(&path, BackupDestination::Manual).unwrap();
    let database = prompt_store::Database::open(&path).unwrap();
    let mut repository = database.into_repository();
    let raw = Connection::open(&path).unwrap();
    raw.execute("UPDATE marker SET value = 'changed'", [])
        .unwrap();
    drop(raw);

    repository.restore_from_backup(backup.path()).unwrap();
    drop(repository);
    let restored = Connection::open(&path).unwrap();
    let value: String = restored
        .query_row("SELECT value FROM marker", [], |row| row.get(0))
        .unwrap();
    assert_eq!(value, "original");
}

#[test]
fn retention_prunes_only_application_named_backups_beyond_the_requested_count() {
    let directory = tempdir().unwrap();
    let source = directory.path().join("library.db");
    database_with_marker(&source, "original");
    let backup_directory = directory.path().join("backups");
    std::fs::create_dir(&backup_directory).unwrap();
    for name in [
        "library-manual-0001.db",
        "library-manual-0002.db",
        "library-manual-0003.db",
        "other.db",
    ] {
        std::fs::write(backup_directory.join(name), b"marker").unwrap();
    }

    assert_eq!(prune_backups(&source, 2).unwrap(), 1);
    assert!(!backup_directory.join("library-manual-0001.db").exists());
    assert!(backup_directory.join("library-manual-0002.db").exists());
    assert!(backup_directory.join("library-manual-0003.db").exists());
    assert!(backup_directory.join("other.db").exists());
}
