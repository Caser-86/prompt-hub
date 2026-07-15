use prompt_domain::{
    Actor, AuditAction, Prompt, PromptContent, PromptId, PromptVersion, PromptVersionId,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::Serialize;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::search::refresh_search_index;

pub struct PromptRepository {
    pub(crate) connection: Connection,
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
                        id, status, effectiveness, current_version, entity_json, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        prompt_id,
                        wire_value(&prompt.status())?,
                        wire_value(&prompt.effectiveness())?,
                        incoming_version,
                        serde_json::to_string(prompt)?,
                        prompt.created_at().unix_timestamp(),
                        prompt.updated_at().unix_timestamp(),
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
             deleted_at = CASE WHEN ?2 = 'deleted' THEN ?6 ELSE NULL END
         WHERE id = ?1",
        params![
            prompt_id,
            wire_value(&prompt.status())?,
            wire_value(&prompt.effectiveness())?,
            prompt.current_version().number(),
            serde_json::to_string(prompt)?,
            prompt.updated_at().unix_timestamp(),
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
                id, prompt_id, kind, name, location, collected_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                source.id().to_string(),
                prompt_id,
                wire_value(&source.kind())?,
                source.name(),
                source.location(),
                source.collected_at().unix_timestamp(),
            ],
        )?;
    }
    Ok(())
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
}
