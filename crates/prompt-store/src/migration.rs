use std::collections::BTreeSet;
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
const PROMPT_METADATA_AND_USAGE_SCHEMA: &str =
    include_str!("../migrations/0008_prompt_metadata_and_usage.sql");
const PROMPT_VERSION_METADATA_SNAPSHOTS_SCHEMA: &str =
    include_str!("../migrations/0009_prompt_version_metadata_snapshots.sql");
const MIGRATIONS: &[(u32, &str)] = &[
    (1, INITIAL_SCHEMA),
    (2, SEARCH_SCHEMA),
    (3, FAVORITES_SCHEMA),
    (4, IMPORT_JOBS_SCHEMA),
    (5, SKILLS_SCHEMA),
    (6, SKILL_SNAPSHOTS_SCHEMA),
    (7, MIGRATION_LEDGER_SCHEMA),
    (8, PROMPT_METADATA_AND_USAGE_SCHEMA),
    (9, PROMPT_VERSION_METADATA_SNAPSHOTS_SCHEMA),
];

const MIGRATION_IDS: &[(&str, &str)] = &[
    ("legacy/0001-initial", INITIAL_SCHEMA),
    ("legacy/0002-search", SEARCH_SCHEMA),
    ("legacy/0003-favorites", FAVORITES_SCHEMA),
    ("legacy/0004-import-jobs", IMPORT_JOBS_SCHEMA),
    ("legacy/0005-skills", SKILLS_SCHEMA),
    ("legacy/0006-skill-snapshots", SKILL_SNAPSHOTS_SCHEMA),
    ("20260824_01_migration_ledger", MIGRATION_LEDGER_SCHEMA),
    (
        "20260829_01_prompt_metadata_and_usage",
        PROMPT_METADATA_AND_USAGE_SCHEMA,
    ),
    (
        "20260831_01_prompt_version_metadata_snapshots",
        PROMPT_VERSION_METADATA_SNAPSHOTS_SCHEMA,
    ),
];
const LEGACY_PROMPT_USAGE_ID: &str = "legacy/0.1.2-prompt-usage";
const LEGACY_PROMPT_USAGE_SQL: &str = "ALTER TABLE prompts ADD COLUMN last_used_at INTEGER;";
const PROMPT_METADATA_AND_USAGE_ID: &str = "20260829_01_prompt_metadata_and_usage";
const REQUIRED_LATEST_SCHEMA: &[(&str, &[&str])] = &[
    ("schema_migrations", &["version", "applied_at"]),
    (
        "prompts",
        &[
            "id",
            "status",
            "effectiveness",
            "current_version",
            "entity_json",
            "created_at",
            "updated_at",
            "deleted_at",
            "imported_at",
            "last_validated_at",
        ],
    ),
    (
        "prompt_versions",
        &[
            "prompt_id",
            "version_number",
            "version_id",
            "title",
            "body",
            "description",
            "content_json",
            "metadata_json",
            "actor",
            "created_at",
        ],
    ),
    ("categories", &["id", "name"]),
    (
        "prompt_version_categories",
        &["prompt_id", "version_number", "category_id"],
    ),
    ("tags", &["id", "name"]),
    (
        "prompt_version_tags",
        &["prompt_id", "version_number", "tag_id"],
    ),
    (
        "prompt_version_variables",
        &["prompt_id", "version_number", "name", "definition_json"],
    ),
    (
        "prompt_sources",
        &[
            "id",
            "prompt_id",
            "kind",
            "name",
            "location",
            "collected_at",
            "raw_excerpt",
            "import_job_id",
        ],
    ),
    (
        "compatibilities",
        &[
            "id",
            "prompt_id",
            "tool",
            "model",
            "status",
            "notes",
            "confirmed_at",
        ],
    ),
    (
        "validation_records",
        &[
            "id",
            "prompt_id",
            "status",
            "rating",
            "notes",
            "validated_at",
        ],
    ),
    (
        "audit_events",
        &["id", "prompt_id", "action", "actor", "occurred_at"],
    ),
    (
        "import_jobs",
        &[
            "id",
            "source_kind",
            "status",
            "started_at",
            "completed_at",
            "diagnostics_json",
            "source_path",
            "source_fingerprint",
        ],
    ),
    (
        "prompt_fts",
        &[
            "prompt_id",
            "title",
            "body",
            "description",
            "tags",
            "variables",
        ],
    ),
    ("prompt_favorites", &["prompt_id", "marked_at"]),
    (
        "import_job_items",
        &[
            "id",
            "job_id",
            "source_path",
            "body_fingerprint",
            "title",
            "outcome",
            "warnings_json",
            "error_message",
            "prompt_id",
            "recorded_at",
        ],
    ),
    (
        "skills",
        &[
            "id",
            "name",
            "description",
            "tool_kind",
            "source_kind",
            "source_location",
            "source_revision",
            "content_hash",
            "skill_markdown",
            "risk_flags",
            "review_status",
            "review_notes",
            "reviewed_at",
            "favorite",
            "created_at",
            "updated_at",
            "snapshot_path",
        ],
    ),
    (
        "skill_files",
        &["skill_id", "relative_path", "bytes", "sha256", "kind"],
    ),
    (
        "skill_installations",
        &[
            "id",
            "skill_id",
            "target_root",
            "install_path",
            "installed_hash",
            "backup_path",
            "installed_at",
            "last_verified_at",
        ],
    ),
    (
        "migration_ledger",
        &[
            "migration_id",
            "checksum_sha256",
            "applied_at",
            "provenance",
        ],
    ),
    ("prompt_usage", &["prompt_id", "use_count", "last_used_at"]),
];

pub const LATEST_SCHEMA_VERSION: u32 = 9;

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
        migrate_connection(&mut connection)?;
        Ok(Self {
            connection,
            migration_report: MigrationReport { backup_path },
        })
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let mut connection = Connection::open_in_memory()?;
        migrate_connection(&mut connection)?;
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

pub(crate) fn migrate_connection(connection: &mut Connection) -> Result<(), StoreError> {
    configure(connection)?;
    apply_migrations(connection, MIGRATIONS)
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
    let needs_recovery_backup =
        current_version == latest_version && latest_schema_needs_recovery_backup(&probe)?;
    drop(probe);
    if current_version > latest_version
        || (current_version == latest_version && !needs_recovery_backup)
    {
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
    crate::backup::copy_database(path, &backup_path)?;

    let backup = Connection::open(&backup_path)?;
    let integrity: String = backup.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(StoreError::BackupIntegrity(integrity));
    }
    Ok(Some(backup_path))
}

fn latest_schema_needs_recovery_backup(connection: &Connection) -> Result<bool, StoreError> {
    // A latest-version database normally needs no write on startup. If its
    // shape or migration ledger is damaged, startup may perform a repair or
    // fail closed; preserve the exact pre-repair bytes before either path.
    if validate_latest_schema(connection).is_err() {
        return Ok(true);
    }
    Ok(validate_ledger(connection, LATEST_SCHEMA_VERSION).is_err())
}

fn apply_migrations(
    connection: &mut Connection,
    migrations: &[(u32, &str)],
) -> Result<(), StoreError> {
    let current_version =
        connection.pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))?;

    if current_version > LATEST_SCHEMA_VERSION {
        return Err(StoreError::UnsupportedSchema {
            reason: format!(
                "schema version {current_version} is newer than this application supports"
            ),
        });
    }

    let has_ledger = table_exists(connection, "migration_ledger")?;
    let legacy_prompt_usage = current_version == 5 && has_legacy_prompt_usage_schema(connection)?;
    if has_ledger {
        repair_known_metadata_ledger_omission(connection, current_version)?;
        validate_ledger(connection, current_version)?;
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

    // Databases that already had the ledger must record every newly applied
    // canonical migration as part of the same transaction. Older builds
    // omitted this row for the v8 metadata migration, which made the next
    // startup reject an otherwise complete database.
    if has_ledger {
        for (version, sql) in migrations
            .iter()
            .copied()
            .filter(|(version, _)| *version > current_version)
        {
            if let Some((migration_id, _)) = MIGRATION_IDS
                .iter()
                .find(|(migration_id, _)| migration_id_version(migration_id) == version)
            {
                transaction.execute(
                    "INSERT OR IGNORE INTO migration_ledger(migration_id, checksum_sha256, applied_at, provenance)
                     VALUES (?1, ?2, unixepoch(), 'canonical')",
                    rusqlite::params![migration_id, checksum(sql)],
                )?;
            }
        }
    }

    if !has_ledger {
        backfill_ledger(&transaction, legacy_prompt_usage)?;
    }
    if legacy_prompt_usage {
        transaction.execute(
            "INSERT OR IGNORE INTO prompt_usage(prompt_id, use_count, last_used_at)
             SELECT id, 1, last_used_at FROM prompts WHERE last_used_at IS NOT NULL",
            [],
        )?;
    }
    validate_latest_schema(&transaction)?;
    if has_ledger {
        // Validate the post-migration ledger before committing. This prevents
        // an already-incomplete legacy ledger from being upgraded into a
        // seemingly healthy v8 database that only fails on its next startup.
        validate_ledger(&transaction, LATEST_SCHEMA_VERSION)?;
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

fn column_exists(connection: &Connection, table: &str, column: &str) -> Result<bool, StoreError> {
    let count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2",
        rusqlite::params![table, column],
        |row| row.get(0),
    )?;
    Ok(count == 1)
}

fn validate_latest_schema(connection: &Connection) -> Result<(), StoreError> {
    for (table, columns) in REQUIRED_LATEST_SCHEMA {
        if !table_exists(connection, table)? {
            return Err(StoreError::UnsupportedSchema {
                reason: format!("required table {table} is missing"),
            });
        }
        for column in *columns {
            if !column_exists(connection, table, column)? {
                return Err(StoreError::UnsupportedSchema {
                    reason: format!("required column {table}.{column} is missing"),
                });
            }
        }
    }
    Ok(())
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
            "INSERT OR IGNORE INTO migration_ledger(migration_id, checksum_sha256, applied_at, provenance)
             VALUES (?1, ?2, unixepoch(), 'canonical')",
            rusqlite::params![id, checksum(sql)],
        )?;
    }

    if legacy_prompt_usage {
        transaction.execute(
            "INSERT OR IGNORE INTO migration_ledger(migration_id, checksum_sha256, applied_at, provenance)
             VALUES (?1, ?2, unixepoch(), 'legacy_recovery')",
            rusqlite::params![
                LEGACY_PROMPT_USAGE_ID,
                checksum(LEGACY_PROMPT_USAGE_SQL)
            ],
        )?;
    }
    Ok(())
}

fn validate_ledger(connection: &Connection, current_version: u32) -> Result<(), StoreError> {
    validate_ledger_with_allowed_missing(connection, current_version, None)
}

fn validate_ledger_with_allowed_missing(
    connection: &Connection,
    current_version: u32,
    allowed_missing: Option<&str>,
) -> Result<(), StoreError> {
    let mut statement =
        connection.prepare("SELECT migration_id, checksum_sha256 FROM migration_ledger")?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut recorded_ids = BTreeSet::new();
    for row in rows {
        let (migration_id, stored_checksum) = row?;
        let expected_sql =
            if let Some((_, sql)) = MIGRATION_IDS.iter().find(|(id, _)| *id == migration_id) {
                *sql
            } else if migration_id == LEGACY_PROMPT_USAGE_ID {
                LEGACY_PROMPT_USAGE_SQL
            } else {
                return Err(StoreError::UnsupportedSchema {
                    reason: format!("unknown migration id {migration_id}"),
                });
            };
        if checksum(expected_sql) != stored_checksum {
            return Err(StoreError::MigrationChecksumConflict { migration_id });
        }
        recorded_ids.insert(migration_id);
    }

    if current_version == LATEST_SCHEMA_VERSION {
        let missing = MIGRATION_IDS
            .iter()
            .map(|(id, _)| *id)
            .find(|id| !recorded_ids.contains(*id));
        if let Some(migration_id) = missing.filter(|id| Some(*id) != allowed_missing) {
            return Err(StoreError::UnsupportedSchema {
                reason: format!(
                    "migration ledger is incomplete: missing canonical entry {migration_id}"
                ),
            });
        }
    }
    Ok(())
}

fn repair_known_metadata_ledger_omission(
    connection: &mut Connection,
    current_version: u32,
) -> Result<(), StoreError> {
    if current_version != LATEST_SCHEMA_VERSION
        || !table_exists(connection, "migration_ledger")?
        || !table_exists(connection, "prompt_usage")?
    {
        return Ok(());
    }

    let missing: i64 = connection.query_row(
        "SELECT COUNT(*) FROM migration_ledger WHERE migration_id = ?1",
        [PROMPT_METADATA_AND_USAGE_ID],
        |row| row.get(0),
    )?;
    if missing != 0 {
        return Ok(());
    }

    // Only backfill this one known omission after the complete v8 shape and
    // every other ledger entry have passed validation. Any unrelated gap,
    // unknown migration, checksum conflict, or schema damage still fails
    // closed below.
    validate_latest_schema(connection)?;
    validate_ledger_with_allowed_missing(
        connection,
        current_version,
        Some(PROMPT_METADATA_AND_USAGE_ID),
    )?;

    let transaction = connection.transaction()?;
    transaction.execute(
        "INSERT INTO migration_ledger(migration_id, checksum_sha256, applied_at, provenance)
         VALUES (?1, ?2, unixepoch(), 'legacy_recovery')",
        rusqlite::params![
            PROMPT_METADATA_AND_USAGE_ID,
            checksum(PROMPT_METADATA_AND_USAGE_SCHEMA)
        ],
    )?;
    transaction.commit()?;
    Ok(())
}

fn migration_id_version(migration_id: &str) -> u32 {
    MIGRATION_IDS
        .iter()
        .position(|(id, _)| *id == migration_id)
        .map_or(0, |index| index as u32 + 1)
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

    #[test]
    fn a_latest_schema_repair_creates_a_pre_repair_backup() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("prompt-hub.db");
        drop(Database::open(&path).unwrap());
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .execute(
                    "DELETE FROM migration_ledger WHERE migration_id = ?1",
                    [PROMPT_METADATA_AND_USAGE_ID],
                )
                .unwrap();
        }

        let reopened = Database::open(&path).unwrap();

        let backup = reopened
            .migration_report()
            .backup_path()
            .expect("latest-schema repair must preserve the pre-repair database");
        assert!(backup.exists());
        assert!(
            backup
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains("pre-migration")
        );
    }
}
