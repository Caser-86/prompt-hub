use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use crate::{PromptRepository, StoreError};

const INITIAL_SCHEMA: &str = include_str!("../migrations/0001_initial.sql");
const MIGRATIONS: &[(u32, &str)] = &[(1, INITIAL_SCHEMA)];

pub const LATEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Default)]
pub struct MigrationReport {
    backup_path: Option<PathBuf>,
}

impl MigrationReport {
    #[must_use]
    pub fn backup_path(&self) -> Option<&Path> {
        self.backup_path.as_deref()
    }
}

pub struct Database {
    connection: Connection,
    migration_report: MigrationReport,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let backup_path = backup_before_migration(path, LATEST_SCHEMA_VERSION)?;
        let mut connection = Connection::open(path)?;
        configure(&connection)?;
        apply_migrations(&mut connection, MIGRATIONS)?;
        Ok(Self {
            connection,
            migration_report: MigrationReport { backup_path },
        })
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let mut connection = Connection::open_in_memory()?;
        configure(&connection)?;
        apply_migrations(&mut connection, MIGRATIONS)?;
        Ok(Self {
            connection,
            migration_report: MigrationReport::default(),
        })
    }

    pub fn schema_version(&self) -> Result<u32, StoreError> {
        let version = self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))?;
        Ok(version)
    }

    #[must_use]
    pub const fn migration_report(&self) -> &MigrationReport {
        &self.migration_report
    }

    #[must_use]
    pub fn into_repository(self) -> PromptRepository {
        PromptRepository::new(self.connection)
    }
}

fn configure(connection: &Connection) -> Result<(), StoreError> {
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(())
}

fn backup_before_migration(
    path: &Path,
    latest_version: u32,
) -> Result<Option<PathBuf>, StoreError> {
    if !path.exists() || fs::metadata(path)?.len() == 0 {
        return Ok(None);
    }

    let probe = Connection::open(path)?;
    let current_version =
        probe.pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))?;
    drop(probe);
    if current_version >= latest_version {
        return Ok(None);
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| StoreError::Clock(error.to_string()))?
        .as_secs();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map_or_else(String::new, |value| format!("{value}."));
    let backup_path = path.with_extension(format!(
        "{extension}v{current_version}.pre-migration.{timestamp}.bak"
    ));
    fs::copy(path, &backup_path)?;

    let backup = Connection::open(&backup_path)?;
    let integrity: String = backup.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(StoreError::BackupIntegrity(integrity));
    }
    Ok(Some(backup_path))
}

fn apply_migrations(
    connection: &mut Connection,
    migrations: &[(u32, &str)],
) -> Result<(), StoreError> {
    let current_version =
        connection.pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))?;
    let transaction = connection.transaction()?;
    for (version, sql) in migrations
        .iter()
        .copied()
        .filter(|(version, _)| *version > current_version)
    {
        transaction.execute_batch(sql)?;
        transaction.execute(
            "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, unixepoch())",
            [version],
        )?;
        transaction.pragma_update(None, "user_version", version)?;
    }
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_migration_rolls_back_the_whole_batch() {
        let mut connection = Connection::open_in_memory().unwrap();
        let migrations = [
            (
                1,
                "CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL) STRICT;
                 CREATE TABLE first_table(id INTEGER PRIMARY KEY) STRICT;",
            ),
            (2, "CREATE TABLE invalid syntax"),
        ];

        assert!(apply_migrations(&mut connection, &migrations).is_err());
        let table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'first_table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let version: u32 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(table_count, 0);
        assert_eq!(version, 0);
    }
}
