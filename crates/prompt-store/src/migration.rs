use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use sha2::{Digest, Sha256};

use crate::{PromptRepository, SkillRepository, StoreError};

const INITIAL_SCHEMA: &str = include_str!("../migrations/0001_initial.sql");
const SEARCH_SCHEMA: &str = include_str!("../migrations/0002_search.sql");
const FAVORITES_SCHEMA: &str = include_str!("../migrations/0003_favorites.sql");
const IMPORT_JOBS_SCHEMA: &str = include_str!("../migrations/0004_import_jobs.sql");
const SKILLS_SCHEMA: &str = include_str!("../migrations/0005_skills.sql");
const SKILL_SNAPSHOTS_SCHEMA: &str = include_str!("../migrations/0006_skill_snapshots.sql");
const MIGRATION_LEDGER_SCHEMA: &str = include_str!("../migrations/0007_migration_ledger.sql");
const MIGRATIONS: &[(u32, &str)] = &[
    (1, INITIAL_SCHEMA),
    (2, SEARCH_SCHEMA),
    (3, FAVORITES_SCHEMA),
    (4, IMPORT_JOBS_SCHEMA),
    (5, SKILLS_SCHEMA),
    (6, SKILL_SNAPSHOTS_SCHEMA),
    (7, MIGRATION_LEDGER_SCHEMA),
];

const MIGRATION_IDS: &[(&str, &str)] = &[
    ("legacy/0001-initial", INITIAL_SCHEMA),
    ("legacy/0002-search", SEARCH_SCHEMA),
    ("legacy/0003-favorites", FAVORITES_SCHEMA),
    ("legacy/0004-import-jobs", IMPORT_JOBS_SCHEMA),
    ("legacy/0005-skills", SKILLS_SCHEMA),
    ("legacy/0006-skill-snapshots", SKILL_SNAPSHOTS_SCHEMA),
    ("20260824_01_migration_ledger", MIGRATION_LEDGER_SCHEMA),
];

pub const LATEST_SCHEMA_VERSION: u32 = 7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationLedgerEntry {
    migration_id: String,
    checksum_sha256: String,
    applied_at: i64,
    provenance: String,
}

impl MigrationLedgerEntry {
    #[must_use]
    pub fn migration_id(&self) -> &str {
        &self.migration_id
    }
    #[must_use]
    pub fn checksum_sha256(&self) -> &str {
        &self.checksum_sha256
    }
    #[must_use]
    pub const fn applied_at(&self) -> i64 {
        self.applied_at
    }
    #[must_use]
    pub fn provenance(&self) -> &str {
        &self.provenance
    }
}

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

    #[must_use]
    pub fn into_skill_repository(self) -> SkillRepository {
        SkillRepository::new(self.connection)
    }

    pub fn migration_ledger(&self) -> Result<Vec<MigrationLedgerEntry>, StoreError> {
        let mut statement = self.connection.prepare(
            "SELECT migration_id, checksum_sha256, applied_at, provenance
             FROM migration_ledger ORDER BY migration_id ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(MigrationLedgerEntry {
                migration_id: row.get(0)?,
                checksum_sha256: row.get(1)?,
                applied_at: row.get(2)?,
                provenance: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
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

    let has_ledger = table_exists(connection, "migration_ledger")?;
    let legacy_prompt_usage = current_version == 5 && has_legacy_prompt_usage_schema(connection)?;
    if has_ledger {
        validate_ledger(connection)?;
    } else if current_version > 0 && !has_supported_legacy_schema(connection)? {
        return Err(StoreError::UnsupportedSchema {
            reason: "required Prompt Hub tables are missing or ambiguous".to_owned(),
        });
    }

    let transaction = connection.transaction()?;

    // Version 0.1.2 used migration number 5 for `prompts.last_used_at`.
    // This branch later used the same number for the Skill tables. Detect that
    // released schema by structure, rather than treating its user_version as
    // the Skill migration, then create the missing tables before migration 6.
    if legacy_prompt_usage {
        transaction.execute_batch(SKILLS_SCHEMA)?;
    }

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

    if !has_ledger {
        backfill_ledger(&transaction, legacy_prompt_usage)?;
    }
    transaction.commit()?;
    Ok(())
}

fn table_exists(connection: &Connection, name: &str) -> Result<bool, StoreError> {
    let count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [name],
        |row| row.get(0),
    )?;
    Ok(count == 1)
}

fn has_supported_legacy_schema(connection: &Connection) -> Result<bool, StoreError> {
    let prompts = table_exists(connection, "prompts")?;
    let migrations = table_exists(connection, "schema_migrations")?;
    Ok(prompts && migrations)
}

fn checksum(sql: &str) -> String {
    let digest = Sha256::digest(sql.as_bytes());
    hex::encode(digest)
}

fn backfill_ledger(
    transaction: &rusqlite::Transaction<'_>,
    legacy_prompt_usage: bool,
) -> Result<(), StoreError> {
    for (id, sql) in MIGRATION_IDS {
        transaction.execute(
            "INSERT INTO migration_ledger(migration_id, checksum_sha256, applied_at, provenance)
             VALUES (?1, ?2, unixepoch(), 'canonical')",
            rusqlite::params![id, checksum(sql)],
        )?;
    }

    if legacy_prompt_usage {
        transaction.execute(
            "INSERT INTO migration_ledger(migration_id, checksum_sha256, applied_at, provenance)
             VALUES (?1, ?2, unixepoch(), 'legacy_recovery')",
            rusqlite::params![
                "legacy/0.1.2-prompt-usage",
                checksum("ALTER TABLE prompts ADD COLUMN last_used_at INTEGER;")
            ],
        )?;
    }
    Ok(())
}

fn validate_ledger(connection: &Connection) -> Result<(), StoreError> {
    let mut statement =
        connection.prepare("SELECT migration_id, checksum_sha256 FROM migration_ledger")?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (migration_id, stored_checksum) = row?;
        let Some((_, sql)) = MIGRATION_IDS.iter().find(|(id, _)| *id == migration_id) else {
            return Err(StoreError::UnsupportedSchema {
                reason: format!("unknown migration id {migration_id}"),
            });
        };
        if checksum(sql) != stored_checksum {
            return Err(StoreError::MigrationChecksumConflict { migration_id });
        }
    }
    Ok(())
}

fn has_legacy_prompt_usage_schema(connection: &Connection) -> Result<bool, StoreError> {
    let has_skills_table: i64 = connection.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'skills'",
        [],
        |row| row.get(0),
    )?;
    if has_skills_table != 0 {
        return Ok(false);
    }

    let has_last_used_at: i64 = connection.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('prompts') WHERE name = 'last_used_at'",
        [],
        |row| row.get(0),
    )?;
    Ok(has_last_used_at != 0)
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
