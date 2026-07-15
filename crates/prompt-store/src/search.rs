use prompt_domain::{EffectivenessStatus, PromptId, PromptStatus, PromptVersion, SourceKind};
use rusqlite::types::Value;
use rusqlite::{Transaction, params, params_from_iter};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{PromptRepository, StoreError};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchFilters {
    pub favorite: Option<bool>,
    pub status: Option<PromptStatus>,
    pub effectiveness: Option<EffectivenessStatus>,
    pub source_kind: Option<SourceKind>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub tool: Option<String>,
    pub model: Option<String>,
    pub minimum_rating: Option<u8>,
    pub updated_after: Option<OffsetDateTime>,
    pub updated_before: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    text: String,
    filters: SearchFilters,
    limit: u32,
    offset: u32,
}

impl SearchQuery {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into().trim().to_owned(),
            filters: SearchFilters::default(),
            limit: 20,
            offset: 0,
        }
    }

    #[must_use]
    pub fn with_filters(mut self, filters: SearchFilters) -> Self {
        self.filters = filters;
        self
    }

    #[must_use]
    pub fn with_page(mut self, limit: u32, offset: u32) -> Self {
        self.limit = limit.clamp(1, 100);
        self.offset = offset;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub id: PromptId,
    pub title: String,
    pub snippet: String,
    pub status: PromptStatus,
    pub effectiveness: EffectivenessStatus,
    pub rating: Option<u8>,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub hits: Vec<SearchHit>,
    pub total: u64,
}

impl PromptRepository {
    pub fn search(&self, query: SearchQuery) -> Result<SearchPage, StoreError> {
        let use_fts = query.text.chars().count() >= 3;
        let mut values = Vec::<Value>::new();
        let matches_sql = if use_fts {
            values.push(Value::Text(fts_literal(&query.text)));
            "SELECT
                prompt_id,
                title,
                snippet(prompt_fts, 2, '<mark>', '</mark>', '…', 24) AS snippet,
                bm25(prompt_fts, 10.0, 5.0, 2.0, 1.0, 1.0) AS text_rank
             FROM prompt_fts
             WHERE prompt_fts MATCH ?"
        } else if query.text.is_empty() {
            "SELECT prompt_id, title, body AS snippet, 0.0 AS text_rank FROM prompt_fts"
        } else {
            let pattern = format!("%{}%", escape_like(&query.text));
            values.push(Value::Text(pattern.clone()));
            values.push(Value::Text(pattern.clone()));
            values.push(Value::Text(pattern.clone()));
            values.push(Value::Text(pattern.clone()));
            values.push(Value::Text(pattern));
            "SELECT prompt_id, title, body AS snippet, 0.0 AS text_rank
             FROM prompt_fts
             WHERE title LIKE ? ESCAPE '\\'
                OR body LIKE ? ESCAPE '\\'
                OR description LIKE ? ESCAPE '\\'
                OR tags LIKE ? ESCAPE '\\'
                OR variables LIKE ? ESCAPE '\\'"
        };

        let mut sql = format!(
            "WITH matches AS ({matches_sql})
             SELECT
                p.id,
                m.title,
                m.snippet,
                p.status,
                p.effectiveness,
                p.updated_at,
                (
                    SELECT vr.rating FROM validation_records vr
                    WHERE vr.prompt_id = p.id
                    ORDER BY vr.validated_at DESC, vr.id DESC LIMIT 1
                ) AS rating,
                COUNT(*) OVER() AS full_count
             FROM matches m
             JOIN prompts p ON p.id = m.prompt_id
             WHERE 1 = 1"
        );
        append_filters(&mut sql, &mut values, &query.filters)?;
        sql.push_str(
            " ORDER BY
                m.text_rank ASC,
                CASE p.effectiveness
                    WHEN 'effective' THEN 0
                    WHEN 'needs_retest' THEN 1
                    WHEN 'unverified' THEN 2
                    WHEN 'ineffective' THEN 3
                    ELSE 4
                END ASC,
                COALESCE(rating, 0) DESC,
                p.updated_at DESC,
                p.id ASC
              LIMIT ? OFFSET ?",
        );
        values.push(Value::Integer(i64::from(query.limit)));
        values.push(Value::Integer(i64::from(query.offset)));

        let mut statement = self.connection.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(values.iter()), |row| {
            Ok(RawSearchHit {
                id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
                status: row.get(3)?,
                effectiveness: row.get(4)?,
                updated_at: row.get(5)?,
                rating: row.get(6)?,
                full_count: row.get(7)?,
            })
        })?;

        let mut hits = Vec::new();
        let mut total = 0;
        for row in rows {
            let row = row?;
            total = row.full_count;
            let snippet = if use_fts || query.text.is_empty() {
                row.snippet
            } else {
                highlight_literal(&row.snippet, &query.text)
            };
            hits.push(SearchHit {
                id: PromptId::from_uuid(Uuid::parse_str(&row.id)?),
                title: row.title,
                snippet,
                status: parse_wire(&row.status)?,
                effectiveness: parse_wire(&row.effectiveness)?,
                rating: row.rating,
                updated_at: OffsetDateTime::from_unix_timestamp(row.updated_at)
                    .map_err(|error| StoreError::Clock(error.to_string()))?,
            });
        }
        Ok(SearchPage {
            hits,
            total: total.try_into().unwrap_or(u64::MAX),
        })
    }

    pub fn search_index_is_consistent(&self) -> Result<bool, StoreError> {
        let consistent = self.connection.query_row(
            "SELECT
                (SELECT COUNT(*) FROM prompts) = (SELECT COUNT(*) FROM prompt_fts)
                AND NOT EXISTS (
                    SELECT 1
                    FROM prompts p
                    LEFT JOIN prompt_fts f ON f.prompt_id = p.id
                    WHERE f.rowid IS NULL
                )",
            [],
            |row| row.get(0),
        )?;
        Ok(consistent)
    }

    pub fn rebuild_search_index(&mut self) -> Result<(), StoreError> {
        let transaction = self.connection.transaction()?;
        transaction.execute("DELETE FROM prompt_fts", [])?;
        transaction.execute_batch(
            "INSERT INTO prompt_fts(prompt_id, title, body, description, tags, variables)
             SELECT
                p.id,
                v.title,
                v.body,
                COALESCE(v.description, ''),
                COALESCE((
                    SELECT group_concat(t.name, ' ')
                    FROM prompt_version_tags pvt
                    JOIN tags t ON t.id = pvt.tag_id
                    WHERE pvt.prompt_id = p.id
                      AND pvt.version_number = p.current_version
                ), ''),
                COALESCE((
                    SELECT group_concat(pvv.name, ' ')
                    FROM prompt_version_variables pvv
                    WHERE pvv.prompt_id = p.id
                      AND pvv.version_number = p.current_version
                ), '')
             FROM prompts p
             JOIN prompt_versions v
               ON v.prompt_id = p.id AND v.version_number = p.current_version;",
        )?;
        transaction.commit()?;
        Ok(())
    }
}

struct RawSearchHit {
    id: String,
    title: String,
    snippet: String,
    status: String,
    effectiveness: String,
    updated_at: i64,
    rating: Option<u8>,
    full_count: i64,
}

fn append_filters(
    sql: &mut String,
    values: &mut Vec<Value>,
    filters: &SearchFilters,
) -> Result<(), StoreError> {
    if filters.favorite == Some(true) {
        sql.push_str(" AND EXISTS (SELECT 1 FROM prompt_favorites pf WHERE pf.prompt_id = p.id)");
    }
    if let Some(status) = filters.status {
        sql.push_str(" AND p.status = ?");
        values.push(Value::Text(wire_value(&status)?));
    }
    if let Some(effectiveness) = filters.effectiveness {
        sql.push_str(" AND p.effectiveness = ?");
        values.push(Value::Text(wire_value(&effectiveness)?));
    }
    if let Some(source_kind) = filters.source_kind {
        sql.push_str(
            " AND EXISTS (
                SELECT 1 FROM prompt_sources ps
                WHERE ps.prompt_id = p.id AND ps.kind = ?
              )",
        );
        values.push(Value::Text(wire_value(&source_kind)?));
    }
    if let Some(category) = &filters.category {
        sql.push_str(
            " AND EXISTS (
                SELECT 1
                FROM prompt_version_categories pvc
                JOIN categories c ON c.id = pvc.category_id
                WHERE pvc.prompt_id = p.id
                  AND pvc.version_number = p.current_version
                  AND c.name = ?
              )",
        );
        values.push(Value::Text(category.clone()));
    }
    for tag in &filters.tags {
        sql.push_str(
            " AND EXISTS (
                SELECT 1
                FROM prompt_version_tags pvt
                JOIN tags t ON t.id = pvt.tag_id
                WHERE pvt.prompt_id = p.id
                  AND pvt.version_number = p.current_version
                  AND t.name = ?
              )",
        );
        values.push(Value::Text(tag.clone()));
    }
    if filters.tool.is_some() || filters.model.is_some() {
        sql.push_str(
            " AND EXISTS (
                SELECT 1 FROM compatibilities c
                WHERE c.prompt_id = p.id",
        );
        if let Some(tool) = &filters.tool {
            sql.push_str(" AND c.tool = ?");
            values.push(Value::Text(tool.clone()));
        }
        if let Some(model) = &filters.model {
            sql.push_str(" AND c.model = ?");
            values.push(Value::Text(model.clone()));
        }
        sql.push(')');
    }
    if let Some(minimum_rating) = filters.minimum_rating {
        sql.push_str(
            " AND COALESCE((
                SELECT vr.rating FROM validation_records vr
                WHERE vr.prompt_id = p.id
                ORDER BY vr.validated_at DESC, vr.id DESC LIMIT 1
              ), 0) >= ?",
        );
        values.push(Value::Integer(i64::from(minimum_rating)));
    }
    if let Some(updated_after) = filters.updated_after {
        sql.push_str(" AND p.updated_at >= ?");
        values.push(Value::Integer(updated_after.unix_timestamp()));
    }
    if let Some(updated_before) = filters.updated_before {
        sql.push_str(" AND p.updated_at <= ?");
        values.push(Value::Integer(updated_before.unix_timestamp()));
    }
    Ok(())
}

pub(crate) fn refresh_search_index(
    transaction: &Transaction<'_>,
    prompt_id: &str,
    version: &PromptVersion,
) -> Result<(), StoreError> {
    let content = version.content();
    let tags = content.tags().join(" ");
    let variables = content
        .variables()
        .iter()
        .map(|variable| variable.name())
        .collect::<Vec<_>>()
        .join(" ");
    transaction.execute("DELETE FROM prompt_fts WHERE prompt_id = ?1", [prompt_id])?;
    transaction.execute(
        "INSERT INTO prompt_fts(prompt_id, title, body, description, tags, variables)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            prompt_id,
            content.title(),
            content.body(),
            content.description().unwrap_or_default(),
            tags,
            variables,
        ],
    )?;
    Ok(())
}

fn wire_value(value: &impl serde::Serialize) -> Result<String, StoreError> {
    match serde_json::to_value(value)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(StoreError::WireValue),
    }
}

fn parse_wire<T: DeserializeOwned>(value: &str) -> Result<T, StoreError> {
    serde_json::from_value(serde_json::Value::String(value.to_owned())).map_err(StoreError::from)
}

fn fts_literal(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn highlight_literal(value: &str, needle: &str) -> String {
    value.replacen(needle, &format!("<mark>{needle}</mark>"), 1)
}
