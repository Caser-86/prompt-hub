use prompt_domain::{
    Actor, AuditAction, Prompt, PromptContent, PromptId, PromptVersion, PromptVersionId,
};
use rusqlite::backup::Backup;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::Serialize;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::search::refresh_search_index;

pub struct PromptRepository {
    pub(crate) connection: Connection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromptUsageStats {
    use_count: i64,
    last_used_at: Option<OffsetDateTime>,
}

impl PromptUsageStats {
    #[must_use]
    pub const fn use_count(&self) -> i64 {
        self.use_count
    }

    #[must_use]
    pub const fn last_used_at(&self) -> Option<OffsetDateTime> {
        self.last_used_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportJob {
    id: String,
    source_kind: String,
    source_path: Option<String>,
    status: String,
    started_at: OffsetDateTime,
    completed_at: Option<OffsetDateTime>,
    diagnostics_json: String,
}

pub struct ImportJobItemRecord<'a> {
    pub job_id: &'a str,
    pub source_path: &'a str,
    pub body_fingerprint: Option<&'a str>,
    pub title: Option<&'a str>,
    pub outcome: &'a str,
    pub warnings_json: &'a str,
    pub error_message: Option<&'a str>,
    pub prompt_id: Option<PromptId>,
    pub recorded_at: OffsetDateTime,
}

impl ImportJob {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }
    #[must_use]
    pub fn source_kind(&self) -> &str {
        &self.source_kind
    }
    #[must_use]
    pub fn source_path(&self) -> Option<&str> {
        self.source_path.as_deref()
    }
    #[must_use]
    pub const fn completed_at(&self) -> Option<OffsetDateTime> {
        self.completed_at
    }
    #[must_use]
    pub const fn started_at(&self) -> OffsetDateTime {
        self.started_at
    }
    #[must_use]
    pub fn diagnostics_json(&self) -> &str {
        &self.diagnostics_json
    }
}

impl PromptRepository {
    pub(crate) const fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub fn save(&mut self, prompt: &Prompt, action: AuditAction) -> Result<(), StoreError> {
        let prompt_id = prompt.id().value().to_string();
        let transaction = self.connection.transaction()?;
        let stored_version = transaction
            .query_row(
                "SELECT current_version FROM prompts WHERE id = ?1",
                [&prompt_id],
                |row| row.get::<_, u32>(0),
            )
            .optional()?;
        let incoming_version = prompt.current_version().number();

        match stored_version {
            None => {
                transaction.execute(
                    "INSERT INTO prompts(
                        id, status, effectiveness, current_version, entity_json, created_at, updated_at, imported_at, last_validated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        prompt_id,
                        wire_value(&prompt.status())?,
                        wire_value(&prompt.effectiveness())?,
                        incoming_version,
                        serde_json::to_string(prompt)?,
                        prompt.created_at().unix_timestamp(),
                        prompt.updated_at().unix_timestamp(),
                        prompt.imported_at().map(|value| value.unix_timestamp()),
                        prompt.last_validated_at().map(|value| value.unix_timestamp()),
                    ],
                )?;
                insert_version(&transaction, prompt_id.as_str(), prompt.current_version())?;
                insert_sources(&transaction, prompt_id.as_str(), prompt)?;
            }
            Some(version) if incoming_version == version => {
                update_prompt(&transaction, prompt_id.as_str(), prompt)?;
            }
            Some(version) if incoming_version == version + 1 => {
                insert_version(&transaction, prompt_id.as_str(), prompt.current_version())?;
                update_prompt(&transaction, prompt_id.as_str(), prompt)?;
            }
            Some(version) => {
                return Err(StoreError::VersionConflict {
                    stored: version,
                    incoming: incoming_version,
                });
            }
        }

        sync_current_metadata(&transaction, prompt_id.as_str(), prompt)?;
        refresh_search_index(&transaction, prompt_id.as_str(), prompt.current_version())?;

        transaction.execute(
            "INSERT INTO audit_events(id, prompt_id, action, actor, occurred_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                Uuid::now_v7().to_string(),
                prompt_id,
                wire_value(&action)?,
                wire_value(&prompt.current_version().actor())?,
                prompt.updated_at().unix_timestamp(),
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn get(&self, id: PromptId) -> Result<Option<Prompt>, StoreError> {
        let serialized = self
            .connection
            .query_row(
                "SELECT entity_json FROM prompts WHERE id = ?1",
                [id.value().to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        serialized
            .map(|value| serde_json::from_str(&value).map_err(StoreError::from))
            .transpose()
    }

    pub fn permanently_delete(&mut self, id: PromptId) -> Result<(), StoreError> {
        let prompt_id = id.value().to_string();
        let transaction = self.connection.transaction()?;
        let status: Option<String> = transaction
            .query_row(
                "SELECT status FROM prompts WHERE id = ?1",
                [&prompt_id],
                |row| row.get(0),
            )
            .optional()?;
        if status.as_deref() != Some("deleted") {
            return Err(StoreError::Domain(
                "only soft-deleted prompts can be permanently removed".to_owned(),
            ));
        }
        transaction.execute("DELETE FROM prompt_fts WHERE prompt_id = ?1", [&prompt_id])?;
        transaction.execute("DELETE FROM prompts WHERE id = ?1", [&prompt_id])?;
        transaction.commit()?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Prompt>, StoreError> {
        let mut statement = self
            .connection
            .prepare("SELECT entity_json FROM prompts ORDER BY updated_at DESC, id ASC")?;
        let serialized = statement.query_map([], |row| row.get::<_, String>(0))?;
        serialized
            .map(|value| {
                let value = value?;
                serde_json::from_str(&value).map_err(StoreError::from)
            })
            .collect()
    }

    pub fn history(&self, id: PromptId) -> Result<Vec<PromptVersion>, StoreError> {
        let mut statement = self.connection.prepare(
            "SELECT version_id, version_number, content_json, actor, created_at
             FROM prompt_versions WHERE prompt_id = ?1 ORDER BY version_number ASC",
        )?;
        let rows = statement.query_map([id.value().to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u32>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?;
        rows.map(|row| {
            let (version_id, number, content, actor, created_at) = row?;
            let version_id = PromptVersionId::from_uuid(Uuid::parse_str(&version_id)?);
            let content: PromptContent = serde_json::from_str(&content)?;
            let actor: Actor = serde_json::from_value(serde_json::Value::String(actor))?;
            let created_at = OffsetDateTime::from_unix_timestamp(created_at)
                .map_err(|error| StoreError::Clock(error.to_string()))?;
            PromptVersion::from_snapshot(version_id, number, content, actor, created_at)
                .map_err(|error| StoreError::Domain(error.to_string()))
        })
        .collect()
    }

    pub fn version_count(&self, id: PromptId) -> Result<u32, StoreError> {
        let count = self.connection.query_row(
            "SELECT COUNT(*) FROM prompt_versions WHERE prompt_id = ?1",
            [id.value().to_string()],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn set_favorite(
        &mut self,
        id: PromptId,
        favorite: bool,
        marked_at: OffsetDateTime,
    ) -> Result<(), StoreError> {
        if favorite {
            self.connection.execute(
                "INSERT INTO prompt_favorites(prompt_id, marked_at) VALUES (?1, ?2)
                 ON CONFLICT(prompt_id) DO UPDATE SET marked_at = excluded.marked_at",
                params![id.value().to_string(), marked_at.unix_timestamp()],
            )?;
        } else {
            self.connection.execute(
                "DELETE FROM prompt_favorites WHERE prompt_id = ?1",
                [id.value().to_string()],
            )?;
        }
        Ok(())
    }

    pub fn is_favorite(&self, id: PromptId) -> Result<bool, StoreError> {
        let found: Option<u8> = self
            .connection
            .query_row(
                "SELECT 1 FROM prompt_favorites WHERE prompt_id = ?1",
                [id.value().to_string()],
                |row| row.get(0),
            )
            .optional()?;
        Ok(found.is_some())
    }

    pub fn record_use(
        &mut self,
        id: PromptId,
        used_at: OffsetDateTime,
    ) -> Result<PromptUsageStats, StoreError> {
        let prompt_id = id.value().to_string();
        let transaction = self.connection.transaction()?;
        if !prompt_exists(&transaction, &prompt_id)? {
            return Err(StoreError::PromptMissing);
        }
        transaction.execute(
            "INSERT INTO prompt_usage(prompt_id, use_count, last_used_at) VALUES (?1, 1, ?2)
             ON CONFLICT(prompt_id) DO UPDATE SET use_count = prompt_usage.use_count + 1, last_used_at = excluded.last_used_at",
            params![prompt_id, used_at.unix_timestamp()],
        )?;
        let stats = usage_stats_for(&transaction, id)?;
        transaction.commit()?;
        Ok(stats)
    }

    pub fn merge_legacy_usage(
        &mut self,
        id: PromptId,
        count: i64,
    ) -> Result<PromptUsageStats, StoreError> {
        let prompt_id = id.value().to_string();
        let transaction = self.connection.transaction()?;
        if !prompt_exists(&transaction, &prompt_id)? {
            return Err(StoreError::PromptMissing);
        }
        if count > 0 {
            transaction.execute(
                "INSERT INTO prompt_usage(prompt_id, use_count, last_used_at) VALUES (?1, ?2, NULL)
                 ON CONFLICT(prompt_id) DO UPDATE SET use_count = MAX(prompt_usage.use_count, excluded.use_count)",
                params![prompt_id, count],
            )?;
        }
        let stats = usage_stats_for(&transaction, id)?;
        transaction.commit()?;
        Ok(stats)
    }

    pub fn usage_stats(&self, id: PromptId) -> Result<PromptUsageStats, StoreError> {
        let found: Option<u8> = self
            .connection
            .query_row(
                "SELECT 1 FROM prompts WHERE id = ?1",
                [id.value().to_string()],
                |row| row.get(0),
            )
            .optional()?;
        if found.is_none() {
            return Err(StoreError::PromptMissing);
        }
        self.connection
            .query_row(
                "SELECT use_count, last_used_at FROM prompt_usage WHERE prompt_id = ?1",
                [id.value().to_string()],
                prompt_usage_from_row,
            )
            .optional()
            .map(|value| value.unwrap_or_default())
            .map_err(StoreError::from)
    }

    pub fn restore_from_backup(&mut self, backup_path: &std::path::Path) -> Result<(), StoreError> {
        let source = Connection::open(backup_path)?;
        let mut migrated = Connection::open_in_memory()?;
        Backup::new(&source, &mut migrated)?.run_to_completion(
            64,
            std::time::Duration::from_millis(5),
            None,
        )?;
        crate::migration::migrate_connection(&mut migrated)?;
        Backup::new(&migrated, &mut self.connection)?.run_to_completion(
            64,
            std::time::Duration::from_millis(5),
            None,
        )?;
        Ok(())
    }

    pub fn start_import_job(
        &mut self,
        source_kind: &str,
        source_path: &str,
        source_fingerprint: Option<&str>,
        started_at: OffsetDateTime,
    ) -> Result<ImportJob, StoreError> {
        let id = Uuid::now_v7().to_string();
        self.connection.execute(
            "INSERT INTO import_jobs(id, source_kind, status, started_at, diagnostics_json, source_path, source_fingerprint)
             VALUES (?1, ?2, 'running', ?3, '{}', ?4, ?5)",
            params![id, source_kind, started_at.unix_timestamp(), source_path, source_fingerprint],
        )?;
        self.import_job(&id)?.ok_or(StoreError::ImportJobMissing)
    }

    pub fn record_import_job_item(
        &mut self,
        item: ImportJobItemRecord<'_>,
    ) -> Result<(), StoreError> {
        self.connection.execute(
            "INSERT INTO import_job_items(id, job_id, source_path, body_fingerprint, title, outcome, warnings_json, error_message, prompt_id, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![Uuid::now_v7().to_string(), item.job_id, item.source_path, item.body_fingerprint, item.title, item.outcome, item.warnings_json, item.error_message, item.prompt_id.map(|id| id.value().to_string()), item.recorded_at.unix_timestamp()],
        )?;
        Ok(())
    }

    pub fn finish_import_job(
        &mut self,
        job_id: &str,
        status: &str,
        diagnostics_json: &str,
        completed_at: OffsetDateTime,
    ) -> Result<(), StoreError> {
        self.connection.execute(
            "UPDATE import_jobs SET status = ?2, completed_at = ?3, diagnostics_json = ?4 WHERE id = ?1",
            params![job_id, status, completed_at.unix_timestamp(), diagnostics_json],
        )?;
        Ok(())
    }

    pub fn import_job(&self, job_id: &str) -> Result<Option<ImportJob>, StoreError> {
        self.connection.query_row(
            "SELECT id, source_kind, source_path, status, started_at, completed_at, diagnostics_json FROM import_jobs WHERE id = ?1",
            [job_id],
            |row| {
                let started_at = OffsetDateTime::from_unix_timestamp(row.get(4)?)
                    .map_err(|error| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Integer, Box::new(error)))?;
                let completed_at = row.get::<_, Option<i64>>(5)?.map(OffsetDateTime::from_unix_timestamp).transpose()
                    .map_err(|error| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Integer, Box::new(error)))?;
                Ok(ImportJob { id: row.get(0)?, source_kind: row.get(1)?, source_path: row.get(2)?, status: row.get(3)?, started_at, completed_at, diagnostics_json: row.get(6)? })
            },
        ).optional().map_err(StoreError::from)
    }

    pub fn recent_import_jobs(&self, limit: u32) -> Result<Vec<ImportJob>, StoreError> {
        let mut statement = self.connection.prepare(
            "SELECT id, source_kind, source_path, status, started_at, completed_at, diagnostics_json
             FROM import_jobs ORDER BY started_at DESC, id DESC LIMIT ?1",
        )?;
        let rows = statement.query_map([i64::from(limit)], |row| {
            let started_at = OffsetDateTime::from_unix_timestamp(row.get(4)?).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Integer,
                    Box::new(error),
                )
            })?;
            let completed_at = row
                .get::<_, Option<i64>>(5)?
                .map(OffsetDateTime::from_unix_timestamp)
                .transpose()
                .map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Integer,
                        Box::new(error),
                    )
                })?;
            Ok(ImportJob {
                id: row.get(0)?,
                source_kind: row.get(1)?,
                source_path: row.get(2)?,
                status: row.get(3)?,
                started_at,
                completed_at,
                diagnostics_json: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }
}

fn sync_current_metadata(
    transaction: &Transaction<'_>,
    prompt_id: &str,
    prompt: &Prompt,
) -> Result<(), StoreError> {
    transaction.execute(
        "DELETE FROM compatibilities WHERE prompt_id = ?1",
        [prompt_id],
    )?;
    for compatibility in prompt.compatibilities() {
        transaction.execute(
            "INSERT INTO compatibilities(
                prompt_id, tool, model, status, notes, confirmed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                prompt_id,
                compatibility.tool(),
                compatibility.model(),
                wire_value(&compatibility.status())?,
                compatibility.notes(),
                compatibility
                    .confirmed_at()
                    .map(|value| value.unix_timestamp()),
            ],
        )?;
    }

    transaction.execute(
        "DELETE FROM validation_records WHERE prompt_id = ?1",
        [prompt_id],
    )?;
    for validation in prompt.validations() {
        transaction.execute(
            "INSERT INTO validation_records(
                prompt_id, status, rating, notes, validated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                prompt_id,
                wire_value(&validation.status)?,
                validation.rating,
                validation.notes,
                validation.validated_at.unix_timestamp(),
            ],
        )?;
    }
    Ok(())
}

fn update_prompt(
    transaction: &Transaction<'_>,
    prompt_id: &str,
    prompt: &Prompt,
) -> Result<(), StoreError> {
    transaction.execute(
        "UPDATE prompts
         SET status = ?2,
             effectiveness = ?3,
             current_version = ?4,
             entity_json = ?5,
             updated_at = ?6,
             imported_at = ?7,
             last_validated_at = ?8,
             deleted_at = CASE WHEN ?2 = 'deleted' THEN ?6 ELSE NULL END
         WHERE id = ?1",
        params![
            prompt_id,
            wire_value(&prompt.status())?,
            wire_value(&prompt.effectiveness())?,
            prompt.current_version().number(),
            serde_json::to_string(prompt)?,
            prompt.updated_at().unix_timestamp(),
            prompt.imported_at().map(|value| value.unix_timestamp()),
            prompt
                .last_validated_at()
                .map(|value| value.unix_timestamp()),
        ],
    )?;
    Ok(())
}

fn insert_version(
    transaction: &Transaction<'_>,
    prompt_id: &str,
    version: &PromptVersion,
) -> Result<(), StoreError> {
    let content = version.content();
    transaction.execute(
        "INSERT INTO prompt_versions(
            prompt_id, version_number, version_id, title, body, description,
            content_json, actor, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            prompt_id,
            version.number(),
            version.id().value().to_string(),
            content.title(),
            content.body(),
            content.description(),
            serde_json::to_string(content)?,
            wire_value(&version.actor())?,
            version.created_at().unix_timestamp(),
        ],
    )?;

    if let Some(category) = content.category() {
        transaction.execute(
            "INSERT INTO categories(name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
            [category],
        )?;
        transaction.execute(
            "INSERT INTO prompt_version_categories(prompt_id, version_number, category_id)
             SELECT ?1, ?2, id FROM categories WHERE name = ?3",
            params![prompt_id, version.number(), category],
        )?;
    }

    for tag in content.tags() {
        transaction.execute(
            "INSERT INTO tags(name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
            [tag],
        )?;
        transaction.execute(
            "INSERT INTO prompt_version_tags(prompt_id, version_number, tag_id)
             SELECT ?1, ?2, id FROM tags WHERE name = ?3",
            params![prompt_id, version.number(), tag],
        )?;
    }

    for variable in content.variables() {
        transaction.execute(
            "INSERT INTO prompt_version_variables(
                prompt_id, version_number, name, definition_json
             ) VALUES (?1, ?2, ?3, ?4)",
            params![
                prompt_id,
                version.number(),
                variable.name(),
                serde_json::to_string(variable)?,
            ],
        )?;
    }
    Ok(())
}

fn insert_sources(
    transaction: &Transaction<'_>,
    prompt_id: &str,
    prompt: &Prompt,
) -> Result<(), StoreError> {
    for source in prompt.sources() {
        transaction.execute(
            "INSERT INTO prompt_sources(
                id, prompt_id, kind, name, location, collected_at, raw_excerpt, import_job_id
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                source.id().to_string(),
                prompt_id,
                wire_value(&source.kind())?,
                source.name(),
                source.location(),
                source.collected_at().unix_timestamp(),
                source.raw_excerpt(),
                source.import_job_id(),
            ],
        )?;
    }
    Ok(())
}

fn prompt_exists(transaction: &Transaction<'_>, prompt_id: &str) -> Result<bool, StoreError> {
    let found: Option<u8> = transaction
        .query_row("SELECT 1 FROM prompts WHERE id = ?1", [prompt_id], |row| {
            row.get(0)
        })
        .optional()?;
    Ok(found.is_some())
}

fn usage_stats_for(
    transaction: &Transaction<'_>,
    id: PromptId,
) -> Result<PromptUsageStats, StoreError> {
    transaction
        .query_row(
            "SELECT use_count, last_used_at FROM prompt_usage WHERE prompt_id = ?1",
            [id.value().to_string()],
            prompt_usage_from_row,
        )
        .optional()
        .map(|value| {
            value.unwrap_or(PromptUsageStats {
                use_count: 0,
                last_used_at: None,
            })
        })
        .map_err(StoreError::from)
}

impl Default for PromptUsageStats {
    fn default() -> Self {
        Self {
            use_count: 0,
            last_used_at: None,
        }
    }
}

fn prompt_usage_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PromptUsageStats> {
    let last_used_at = row
        .get::<_, Option<i64>>(1)?
        .map(OffsetDateTime::from_unix_timestamp)
        .transpose()
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Integer,
                Box::new(error),
            )
        })?;
    Ok(PromptUsageStats {
        use_count: row.get(0)?,
        last_used_at,
    })
}

fn wire_value(value: &impl Serialize) -> Result<String, StoreError> {
    match serde_json::to_value(value)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(StoreError::WireValue),
    }
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("SQLite operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("database file operation failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("stored prompt could not be serialized: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("stored UUID is invalid: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("system clock is before the Unix epoch: {0}")]
    Clock(String),
    #[error("stored domain snapshot is invalid: {0}")]
    Domain(String),
    #[error("pre-migration backup failed integrity check: {0}")]
    BackupIntegrity(String),
    #[error("wire enum did not serialize to a string")]
    WireValue,
    #[error("prompt version conflict: stored {stored}, incoming {incoming}")]
    VersionConflict { stored: u32, incoming: u32 },
    #[error("import job was not found after it was created")]
    ImportJobMissing,
    #[error("prompt was not found")]
    PromptMissing,
    #[error("migration checksum conflict for {migration_id}")]
    MigrationChecksumConflict { migration_id: String },
    #[error("database schema is not a supported Prompt Hub history: {reason}")]
    UnsupportedSchema { reason: String },
    #[error("database recovery is required ({code})")]
    RecoveryRequired { code: String, safe_message: String },
}
