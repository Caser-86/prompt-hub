use prompt_skill::{SkillCandidate, SkillFileKind, SkillRisk};
use rusqlite::{Connection, OptionalExtension, params};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::StoreError;

pub struct SkillRepository {
    connection: Connection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillReviewStatus {
    PendingReview,
    Approved,
    Rejected,
    RiskPendingConfirmation,
}

impl SkillReviewStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::PendingReview => "pending_review",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::RiskPendingConfirmation => "risk_pending_confirmation",
        }
    }

    fn parse(value: String) -> Result<Self, StoreError> {
        match value.as_str() {
            "pending_review" => Ok(Self::PendingReview),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "risk_pending_confirmation" => Ok(Self::RiskPendingConfirmation),
            _ => Err(StoreError::Domain(
                "stored Skill review status is invalid".to_owned(),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillSource {
    kind: String,
    location: String,
    revision: Option<String>,
}

impl SkillSource {
    #[must_use]
    pub fn local_directory(location: impl Into<String>) -> Self {
        Self {
            kind: "local_directory".to_owned(),
            location: location.into(),
            revision: None,
        }
    }

    #[must_use]
    pub fn git_repository(location: impl Into<String>, revision: impl Into<String>) -> Self {
        Self {
            kind: "git_repository".to_owned(),
            location: location.into(),
            revision: Some(revision.into()),
        }
    }

    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    #[must_use]
    pub fn location(&self) -> &str {
        &self.location
    }

    #[must_use]
    pub fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSkillFile {
    relative_path: String,
    bytes: u64,
    sha256: String,
    kind: String,
}

impl StoredSkillFile {
    #[must_use]
    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSkill {
    id: String,
    name: String,
    description: String,
    source: SkillSource,
    snapshot_path: Option<String>,
    content_hash: String,
    skill_markdown: String,
    risks: Vec<String>,
    review_status: SkillReviewStatus,
    review_notes: Option<String>,
    favorite: bool,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    files: Vec<StoredSkillFile>,
}

impl StoredSkill {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
    #[must_use]
    pub fn source(&self) -> &SkillSource {
        &self.source
    }
    #[must_use]
    pub fn snapshot_path(&self) -> Option<&str> {
        self.snapshot_path.as_deref()
    }
    #[must_use]
    pub fn content_hash(&self) -> &str {
        &self.content_hash
    }
    #[must_use]
    pub fn skill_markdown(&self) -> &str {
        &self.skill_markdown
    }
    #[must_use]
    pub fn risks(&self) -> &[String] {
        &self.risks
    }
    #[must_use]
    pub const fn review_status(&self) -> SkillReviewStatus {
        self.review_status
    }
    #[must_use]
    pub fn review_notes(&self) -> Option<&str> {
        self.review_notes.as_deref()
    }
    #[must_use]
    pub const fn favorite(&self) -> bool {
        self.favorite
    }
    #[must_use]
    pub fn files(&self) -> &[StoredSkillFile] {
        &self.files
    }
    #[must_use]
    pub const fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
    #[must_use]
    pub const fn updated_at(&self) -> OffsetDateTime {
        self.updated_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillSummary {
    id: String,
    name: String,
    description: String,
    source: SkillSource,
    risks: Vec<String>,
    review_status: SkillReviewStatus,
    favorite: bool,
    updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillInstallation {
    skill_id: String,
    target_root: String,
    install_path: String,
    installed_hash: String,
    backup_path: Option<String>,
    installed_at: OffsetDateTime,
    last_verified_at: Option<OffsetDateTime>,
}

impl SkillInstallation {
    #[must_use]
    pub fn skill_id(&self) -> &str {
        &self.skill_id
    }
    #[must_use]
    pub fn target_root(&self) -> &str {
        &self.target_root
    }
    #[must_use]
    pub fn install_path(&self) -> &str {
        &self.install_path
    }
    #[must_use]
    pub fn installed_hash(&self) -> &str {
        &self.installed_hash
    }
    #[must_use]
    pub fn backup_path(&self) -> Option<&str> {
        self.backup_path.as_deref()
    }
    #[must_use]
    pub const fn installed_at(&self) -> OffsetDateTime {
        self.installed_at
    }
    #[must_use]
    pub const fn last_verified_at(&self) -> Option<OffsetDateTime> {
        self.last_verified_at
    }
}

impl SkillSummary {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
    #[must_use]
    pub fn source(&self) -> &SkillSource {
        &self.source
    }
    #[must_use]
    pub fn risks(&self) -> &[String] {
        &self.risks
    }
    #[must_use]
    pub const fn review_status(&self) -> SkillReviewStatus {
        self.review_status
    }
    #[must_use]
    pub const fn favorite(&self) -> bool {
        self.favorite
    }
    #[must_use]
    pub const fn updated_at(&self) -> OffsetDateTime {
        self.updated_at
    }
    #[must_use]
    pub const fn skill_markdown(&self) -> Option<&str> {
        None
    }
}

impl SkillRepository {
    pub(crate) const fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub fn save_candidate(
        &mut self,
        candidate: &SkillCandidate,
        source: &SkillSource,
        created_at: OffsetDateTime,
    ) -> Result<StoredSkill, StoreError> {
        self.save_candidate_with_snapshot(candidate, source, None, created_at)
    }

    pub fn save_candidate_with_snapshot(
        &mut self,
        candidate: &SkillCandidate,
        source: &SkillSource,
        snapshot_path: Option<&str>,
        created_at: OffsetDateTime,
    ) -> Result<StoredSkill, StoreError> {
        validate_source(source)?;
        if let Some(existing) = self
            .connection
            .query_row(
                "SELECT id FROM skills WHERE source_location = ?1 AND content_hash = ?2",
                params![source.location(), candidate.content_hash()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
        {
            self.connection.execute(
                "UPDATE skills SET source_revision = ?1, snapshot_path = COALESCE(?2, snapshot_path), updated_at = ?3 WHERE id = ?4",
                params![source.revision(), snapshot_path, created_at.unix_timestamp(), existing],
            )?;
            return self
                .get_skill(&existing)?
                .ok_or_else(|| StoreError::Domain("stored Skill disappeared".to_owned()));
        }
        let id = Uuid::now_v7().to_string();
        let risks = candidate.risks().iter().map(risk_name).collect::<Vec<_>>();
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO skills(id, name, description, tool_kind, source_kind, source_location, source_revision, content_hash, skill_markdown, risk_flags, review_status, review_notes, reviewed_at, favorite, created_at, updated_at, snapshot_path)
             VALUES (?1, ?2, ?3, 'codex', ?4, ?5, ?6, ?7, ?8, ?9, 'pending_review', NULL, NULL, 0, ?10, ?10, ?11)",
            params![id, candidate.name(), candidate.description(), source.kind(), source.location(), source.revision(), candidate.content_hash(), candidate.skill_markdown(), serde_json::to_string(&risks)?, created_at.unix_timestamp(), snapshot_path],
        )?;
        for file in candidate.files() {
            transaction.execute(
                "INSERT INTO skill_files(skill_id, relative_path, bytes, sha256, kind) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, file.relative_path(), i64::try_from(file.bytes()).map_err(|_| StoreError::Domain("Skill file size is too large".to_owned()))?, file.sha256(), file_kind_name(file.kind())],
            )?;
        }
        transaction.commit()?;
        self.get_skill(&id)?
            .ok_or_else(|| StoreError::Domain("stored Skill disappeared".to_owned()))
    }

    pub fn list_skills(&self) -> Result<Vec<SkillSummary>, StoreError> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, description, source_kind, source_location, source_revision, risk_flags, review_status, favorite, updated_at
             FROM skills ORDER BY favorite DESC, updated_at DESC, id ASC",
        )?;
        statement
            .query_map([], |row| {
                let risks: Vec<String> =
                    serde_json::from_str(&row.get::<_, String>(6)?).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            6,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?;
                Ok(SkillSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    source: SkillSource {
                        kind: row.get(3)?,
                        location: row.get(4)?,
                        revision: row.get(5)?,
                    },
                    risks,
                    review_status: SkillReviewStatus::parse(row.get(7)?)
                        .map_err(domain_to_sql_error)?,
                    favorite: row.get::<_, i64>(8)? != 0,
                    updated_at: timestamp(row.get(9)?).map_err(domain_to_sql_error)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn get_skill(&self, id: &str) -> Result<Option<StoredSkill>, StoreError> {
        let row = self.connection.query_row(
            "SELECT id, name, description, source_kind, source_location, source_revision, content_hash, skill_markdown, risk_flags, review_status, review_notes, favorite, created_at, updated_at, snapshot_path FROM skills WHERE id = ?1", [id],
            |row| {
                let risks: Vec<String> = serde_json::from_str(&row.get::<_, String>(8)?).map_err(|error| rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(error)))?;
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, SkillSource { kind: row.get(3)?, location: row.get(4)?, revision: row.get(5)? }, row.get::<_, String>(6)?, row.get::<_, String>(7)?, risks, row.get::<_, String>(9)?, row.get::<_, Option<String>>(10)?, row.get::<_, i64>(11)? != 0, row.get::<_, i64>(12)?, row.get::<_, i64>(13)?, row.get::<_, Option<String>>(14)?))
            },
        ).optional()?;
        let Some((
            id,
            name,
            description,
            source,
            content_hash,
            skill_markdown,
            risks,
            review_status,
            review_notes,
            favorite,
            created_at,
            updated_at,
            snapshot_path,
        )) = row
        else {
            return Ok(None);
        };
        let mut statement = self.connection.prepare("SELECT relative_path, bytes, sha256, kind FROM skill_files WHERE skill_id = ?1 ORDER BY relative_path ASC")?;
        let files = statement
            .query_map([&id], |row| {
                Ok(StoredSkillFile {
                    relative_path: row.get(0)?,
                    bytes: u64::try_from(row.get::<_, i64>(1)?)
                        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, 0))?,
                    sha256: row.get(2)?,
                    kind: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(StoredSkill {
            id,
            name,
            description,
            source,
            snapshot_path,
            content_hash,
            skill_markdown,
            risks,
            review_status: SkillReviewStatus::parse(review_status)?,
            review_notes,
            favorite,
            created_at: timestamp(created_at)?,
            updated_at: timestamp(updated_at)?,
            files,
        }))
    }

    pub fn set_review(
        &mut self,
        id: &str,
        status: SkillReviewStatus,
        notes: Option<&str>,
        reviewed_at: OffsetDateTime,
    ) -> Result<(), StoreError> {
        let changed = self.connection.execute("UPDATE skills SET review_status = ?2, review_notes = ?3, reviewed_at = ?4, updated_at = ?4 WHERE id = ?1", params![id, status.as_str(), notes.map(str::trim).filter(|value| !value.is_empty()), reviewed_at.unix_timestamp()])?;
        if changed == 0 {
            return Err(StoreError::Domain("Skill was not found".to_owned()));
        }
        Ok(())
    }

    pub fn set_favorite(
        &mut self,
        id: &str,
        favorite: bool,
        updated_at: OffsetDateTime,
    ) -> Result<(), StoreError> {
        let changed = self.connection.execute(
            "UPDATE skills SET favorite = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, i64::from(favorite), updated_at.unix_timestamp()],
        )?;
        if changed == 0 {
            return Err(StoreError::Domain("Skill was not found".to_owned()));
        }
        Ok(())
    }

    pub fn record_installation(
        &mut self,
        skill_id: &str,
        target_root: &str,
        install_path: &str,
        installed_hash: &str,
        backup_path: Option<&str>,
        installed_at: OffsetDateTime,
    ) -> Result<(), StoreError> {
        if self.get_skill(skill_id)?.is_none() {
            return Err(StoreError::Domain("Skill was not found".to_owned()));
        }
        self.connection.execute(
            "INSERT INTO skill_installations(id, skill_id, target_root, install_path, installed_hash, backup_path, installed_at, last_verified_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
             ON CONFLICT(skill_id) DO UPDATE SET target_root = excluded.target_root, install_path = excluded.install_path, installed_hash = excluded.installed_hash, backup_path = excluded.backup_path, installed_at = excluded.installed_at, last_verified_at = excluded.last_verified_at",
            params![Uuid::now_v7().to_string(), skill_id, target_root, install_path, installed_hash, backup_path, installed_at.unix_timestamp()],
        )?;
        Ok(())
    }

    pub fn installation(&self, skill_id: &str) -> Result<Option<SkillInstallation>, StoreError> {
        self.connection.query_row(
            "SELECT skill_id, target_root, install_path, installed_hash, backup_path, installed_at, last_verified_at FROM skill_installations WHERE skill_id = ?1", [skill_id],
            |row| Ok(SkillInstallation { skill_id: row.get(0)?, target_root: row.get(1)?, install_path: row.get(2)?, installed_hash: row.get(3)?, backup_path: row.get(4)?, installed_at: timestamp(row.get(5)?).map_err(domain_to_sql_error)?, last_verified_at: row.get::<_, Option<i64>>(6)?.map(|value| timestamp(value).map_err(domain_to_sql_error)).transpose()? }),
        ).optional().map_err(StoreError::from)
    }

    pub fn mark_installation_verified(
        &mut self,
        skill_id: &str,
        verified_at: OffsetDateTime,
    ) -> Result<(), StoreError> {
        let changed = self.connection.execute(
            "UPDATE skill_installations SET last_verified_at = ?2 WHERE skill_id = ?1",
            params![skill_id, verified_at.unix_timestamp()],
        )?;
        if changed == 0 {
            return Err(StoreError::Domain(
                "Skill installation was not found".to_owned(),
            ));
        }
        Ok(())
    }
}

fn validate_source(source: &SkillSource) -> Result<(), StoreError> {
    if source.kind.trim().is_empty() || source.location.trim().is_empty() {
        return Err(StoreError::Domain("Skill source is required".to_owned()));
    }
    Ok(())
}

fn risk_name(risk: &SkillRisk) -> String {
    match risk {
        SkillRisk::ContainsScript => "contains_script",
        SkillRisk::ContainsBinary => "contains_binary",
        SkillRisk::ContainsHiddenFile => "contains_hidden_file",
    }
    .to_owned()
}

fn file_kind_name(kind: SkillFileKind) -> &'static str {
    match kind {
        SkillFileKind::SkillMarkdown => "skill_markdown",
        SkillFileKind::Script => "script",
        SkillFileKind::Binary => "binary",
        SkillFileKind::Hidden => "hidden",
        SkillFileKind::Text => "text",
    }
}

fn timestamp(value: i64) -> Result<OffsetDateTime, StoreError> {
    OffsetDateTime::from_unix_timestamp(value).map_err(|error| StoreError::Clock(error.to_string()))
}

fn domain_to_sql_error(error: StoreError) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(error))
}
