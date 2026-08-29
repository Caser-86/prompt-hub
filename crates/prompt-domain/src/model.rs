use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::service::require_user_actor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromptId(Uuid);

impl PromptId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> Uuid {
        self.0
    }
}

impl Default for PromptId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromptVersionId(Uuid);

impl PromptVersionId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    #[must_use]
    pub const fn value(self) -> Uuid {
        self.0
    }

    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }
}

impl Default for PromptVersionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    User,
    Ai,
    Mcp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptStatus {
    Inbox,
    Published,
    Archived,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectivenessStatus {
    Unverified,
    Effective,
    Ineffective,
    NeedsRetest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Manual,
    FileImport,
    WebUrl,
    AiGenerated,
    Mcp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityStatus {
    Unknown,
    Confirmed,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableKind {
    Text,
    Number,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptVariable {
    name: String,
    kind: VariableKind,
    description: Option<String>,
    default_value: Option<String>,
    required: bool,
}

impl PromptVariable {
    pub fn new(
        name: impl Into<String>,
        kind: VariableKind,
        description: Option<String>,
        default_value: Option<String>,
        required: bool,
    ) -> Result<Self, DomainError> {
        let name = required_text(name, DomainError::VariableNameRequired)?;
        Ok(Self {
            name,
            kind,
            description: normalize_optional(description),
            default_value,
            required,
        })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn kind(&self) -> VariableKind {
        self.kind
    }

    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    #[must_use]
    pub fn default_value(&self) -> Option<&str> {
        self.default_value.as_deref()
    }

    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptContent {
    title: String,
    body: String,
    description: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    variables: Vec<PromptVariable>,
}

impl PromptContent {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        description: Option<String>,
        category: Option<String>,
        tags: Vec<String>,
    ) -> Result<Self, DomainError> {
        Self::with_variables(title, body, description, category, tags, Vec::new())
    }

    pub fn with_variables(
        title: impl Into<String>,
        body: impl Into<String>,
        description: Option<String>,
        category: Option<String>,
        tags: Vec<String>,
        variables: Vec<PromptVariable>,
    ) -> Result<Self, DomainError> {
        let title = required_text(title, DomainError::TitleRequired)?;
        let body = required_text(body, DomainError::BodyRequired)?;
        let mut tags = tags
            .into_iter()
            .filter_map(|value| normalize_optional(Some(value)))
            .collect::<Vec<_>>();
        tags.sort_unstable();
        tags.dedup();

        let mut names = HashSet::new();
        for variable in &variables {
            if !names.insert(variable.name.clone()) {
                return Err(DomainError::DuplicateVariableName(variable.name.clone()));
            }
        }

        Ok(Self {
            title,
            body,
            description: normalize_optional(description),
            category: normalize_optional(category),
            tags,
            variables,
        })
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    #[must_use]
    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    #[must_use]
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    #[must_use]
    pub fn variables(&self) -> &[PromptVariable] {
        &self.variables
    }

    fn has_classification(&self) -> bool {
        self.category.is_some() || !self.tags.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptSource {
    id: Uuid,
    kind: SourceKind,
    name: String,
    location: Option<String>,
    collected_at: OffsetDateTime,
    #[serde(default)]
    raw_excerpt: Option<String>,
    #[serde(default)]
    import_job_id: Option<String>,
}

impl PromptSource {
    pub fn new(
        kind: SourceKind,
        name: impl Into<String>,
        location: Option<String>,
        collected_at: OffsetDateTime,
    ) -> Result<Self, DomainError> {
        let name = required_text(name, DomainError::SourceNameRequired)?;
        let location = normalize_optional(location);
        if matches!(kind, SourceKind::FileImport | SourceKind::WebUrl) && location.is_none() {
            return Err(DomainError::SourceLocationRequired);
        }
        Ok(Self {
            id: Uuid::now_v7(),
            kind,
            name,
            location,
            collected_at,
            raw_excerpt: None,
            import_job_id: None,
        })
    }

    pub fn with_provenance(
        kind: SourceKind,
        name: impl Into<String>,
        location: Option<String>,
        collected_at: OffsetDateTime,
        raw_excerpt: Option<String>,
        import_job_id: Option<String>,
    ) -> Result<Self, DomainError> {
        let mut source = Self::new(kind, name, location, collected_at)?;
        source.raw_excerpt = normalize_optional(raw_excerpt);
        source.import_job_id = normalize_optional(import_job_id);
        Ok(source)
    }

    #[must_use]
    pub const fn kind(&self) -> SourceKind {
        self.kind
    }

    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    #[must_use]
    pub const fn collected_at(&self) -> OffsetDateTime {
        self.collected_at
    }

    #[must_use]
    pub fn raw_excerpt(&self) -> Option<&str> {
        self.raw_excerpt.as_deref()
    }

    #[must_use]
    pub fn import_job_id(&self) -> Option<&str> {
        self.import_job_id.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compatibility {
    tool: String,
    model: Option<String>,
    status: CompatibilityStatus,
    notes: Option<String>,
    confirmed_at: Option<OffsetDateTime>,
}

impl Compatibility {
    pub fn new(
        tool: impl Into<String>,
        model: Option<String>,
        status: CompatibilityStatus,
        notes: Option<String>,
        confirmed_at: Option<OffsetDateTime>,
    ) -> Result<Self, DomainError> {
        let tool = required_text(tool, DomainError::ToolNameRequired)?;
        if status == CompatibilityStatus::Confirmed && confirmed_at.is_none() {
            return Err(DomainError::CompatibilityConfirmationRequired);
        }
        Ok(Self {
            tool,
            model: normalize_optional(model),
            status,
            notes: normalize_optional(notes),
            confirmed_at,
        })
    }

    #[must_use]
    pub fn tool(&self) -> &str {
        &self.tool
    }

    #[must_use]
    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    #[must_use]
    pub const fn status(&self) -> CompatibilityStatus {
        self.status
    }

    #[must_use]
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    #[must_use]
    pub const fn confirmed_at(&self) -> Option<OffsetDateTime> {
        self.confirmed_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationRecord {
    pub status: EffectivenessStatus,
    pub rating: Option<u8>,
    pub notes: Option<String>,
    pub validated_at: OffsetDateTime,
}

impl ValidationRecord {
    pub fn new(
        status: EffectivenessStatus,
        rating: Option<u8>,
        notes: Option<String>,
        validated_at: OffsetDateTime,
    ) -> Result<Self, DomainError> {
        if rating.is_some_and(|value| !(1..=5).contains(&value)) {
            return Err(DomainError::RatingOutOfRange);
        }
        Ok(Self {
            status,
            rating,
            notes: normalize_optional(notes),
            validated_at,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Created,
    Published,
    Revised,
    Restored,
    Archived,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub prompt_id: PromptId,
    pub action: AuditAction,
    pub actor: Actor,
    pub occurred_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptVersion {
    id: PromptVersionId,
    number: u32,
    content: PromptContent,
    actor: Actor,
    created_at: OffsetDateTime,
}

impl PromptVersion {
    fn new(number: u32, content: PromptContent, actor: Actor, created_at: OffsetDateTime) -> Self {
        Self {
            id: PromptVersionId::new(),
            number,
            content,
            actor,
            created_at,
        }
    }

    pub fn from_snapshot(
        id: PromptVersionId,
        number: u32,
        content: PromptContent,
        actor: Actor,
        created_at: OffsetDateTime,
    ) -> Result<Self, DomainError> {
        if number == 0 {
            return Err(DomainError::VersionNumberRequired);
        }
        Ok(Self {
            id,
            number,
            content,
            actor,
            created_at,
        })
    }

    #[must_use]
    pub const fn number(&self) -> u32 {
        self.number
    }

    #[must_use]
    pub const fn content(&self) -> &PromptContent {
        &self.content
    }

    #[must_use]
    pub const fn id(&self) -> PromptVersionId {
        self.id
    }

    #[must_use]
    pub const fn actor(&self) -> Actor {
        self.actor
    }

    #[must_use]
    pub const fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prompt {
    id: PromptId,
    status: PromptStatus,
    effectiveness: EffectivenessStatus,
    current_version: PromptVersion,
    sources: Vec<PromptSource>,
    compatibilities: Vec<Compatibility>,
    validations: Vec<ValidationRecord>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    #[serde(default)]
    imported_at: Option<OffsetDateTime>,
    #[serde(default)]
    last_validated_at: Option<OffsetDateTime>,
}

impl Prompt {
    #[must_use]
    pub fn new_inbox(
        content: PromptContent,
        source: PromptSource,
        actor: Actor,
        created_at: OffsetDateTime,
    ) -> Self {
        Self {
            id: PromptId::new(),
            status: PromptStatus::Inbox,
            effectiveness: EffectivenessStatus::Unverified,
            current_version: PromptVersion::new(1, content, actor, created_at),
            sources: vec![source],
            compatibilities: Vec::new(),
            validations: Vec::new(),
            created_at,
            updated_at: created_at,
            imported_at: None,
            last_validated_at: None,
        }
    }

    #[must_use]
    pub fn new_imported_inbox(
        content: PromptContent,
        source: PromptSource,
        actor: Actor,
        imported_at: OffsetDateTime,
    ) -> Self {
        let mut prompt = Self::new_inbox(content, source, actor, imported_at);
        prompt.imported_at = Some(imported_at);
        prompt
    }

    pub fn publish(
        &mut self,
        effectiveness: EffectivenessStatus,
        published_at: OffsetDateTime,
    ) -> Result<(), DomainError> {
        if !self.current_version.content.has_classification() {
            return Err(DomainError::ClassificationRequired);
        }
        self.status = PromptStatus::Published;
        self.effectiveness = effectiveness;
        self.updated_at = published_at;
        Ok(())
    }

    pub fn revise(
        &mut self,
        content: PromptContent,
        actor: Actor,
        revised_at: OffsetDateTime,
    ) -> Result<&PromptVersion, DomainError> {
        require_user_actor(actor)?;
        self.current_version =
            PromptVersion::new(self.current_version.number + 1, content, actor, revised_at);
        self.updated_at = revised_at;
        Ok(&self.current_version)
    }

    pub fn restore(
        &mut self,
        historical: &PromptVersion,
        actor: Actor,
        restored_at: OffsetDateTime,
    ) -> Result<&PromptVersion, DomainError> {
        require_user_actor(actor)?;
        self.current_version = PromptVersion::new(
            self.current_version.number + 1,
            historical.content.clone(),
            actor,
            restored_at,
        );
        self.updated_at = restored_at;
        Ok(&self.current_version)
    }

    pub fn archive(
        &mut self,
        actor: Actor,
        archived_at: OffsetDateTime,
    ) -> Result<(), DomainError> {
        require_user_actor(actor)?;
        self.status = PromptStatus::Archived;
        self.updated_at = archived_at;
        Ok(())
    }

    pub fn soft_delete(
        &mut self,
        actor: Actor,
        deleted_at: OffsetDateTime,
    ) -> Result<(), DomainError> {
        require_user_actor(actor)?;
        self.status = PromptStatus::Deleted;
        self.updated_at = deleted_at;
        Ok(())
    }

    pub fn recover(
        &mut self,
        actor: Actor,
        recovered_at: OffsetDateTime,
    ) -> Result<(), DomainError> {
        require_user_actor(actor)?;
        self.status = PromptStatus::Inbox;
        self.updated_at = recovered_at;
        Ok(())
    }

    pub fn add_compatibility(
        &mut self,
        compatibility: Compatibility,
        actor: Actor,
        updated_at: OffsetDateTime,
    ) -> Result<(), DomainError> {
        require_user_actor(actor)?;
        self.compatibilities.retain(|current| {
            current.tool != compatibility.tool || current.model != compatibility.model
        });
        self.compatibilities.push(compatibility);
        self.updated_at = updated_at;
        Ok(())
    }

    pub fn record_validation(
        &mut self,
        validation: ValidationRecord,
        actor: Actor,
        updated_at: OffsetDateTime,
    ) -> Result<(), DomainError> {
        require_user_actor(actor)?;
        self.effectiveness = validation.status;
        self.validations.push(validation);
        self.last_validated_at = self.validations.last().map(|entry| entry.validated_at);
        self.updated_at = updated_at;
        Ok(())
    }

    #[must_use]
    pub const fn id(&self) -> PromptId {
        self.id
    }

    #[must_use]
    pub const fn current_version(&self) -> &PromptVersion {
        &self.current_version
    }

    #[must_use]
    pub const fn status(&self) -> PromptStatus {
        self.status
    }

    #[must_use]
    pub const fn effectiveness(&self) -> EffectivenessStatus {
        self.effectiveness
    }

    #[must_use]
    pub const fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }

    #[must_use]
    pub const fn updated_at(&self) -> OffsetDateTime {
        self.updated_at
    }

    #[must_use]
    pub const fn imported_at(&self) -> Option<OffsetDateTime> {
        self.imported_at
    }

    #[must_use]
    pub const fn last_validated_at(&self) -> Option<OffsetDateTime> {
        self.last_validated_at
    }

    #[must_use]
    pub fn is_inbox(&self) -> bool {
        self.status == PromptStatus::Inbox
    }

    #[must_use]
    pub fn sources(&self) -> &[PromptSource] {
        &self.sources
    }

    #[must_use]
    pub fn compatibilities(&self) -> &[Compatibility] {
        &self.compatibilities
    }

    #[must_use]
    pub fn validations(&self) -> &[ValidationRecord] {
        &self.validations
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("prompt title is required")]
    TitleRequired,
    #[error("prompt body is required")]
    BodyRequired,
    #[error("a category or tag is required before publication")]
    ClassificationRequired,
    #[error("external actors may create inbox drafts but cannot revise existing prompts")]
    ExternalWriteToPublishedPrompt,
    #[error("source name is required")]
    SourceNameRequired,
    #[error("file and URL sources require a location")]
    SourceLocationRequired,
    #[error("tool name is required")]
    ToolNameRequired,
    #[error("confirmed compatibility requires a confirmation time")]
    CompatibilityConfirmationRequired,
    #[error("variable name is required")]
    VariableNameRequired,
    #[error("duplicate variable name: {0}")]
    DuplicateVariableName(String),
    #[error("rating must be between 1 and 5")]
    RatingOutOfRange,
    #[error("version number must be greater than zero")]
    VersionNumberRequired,
}

fn required_text(value: impl Into<String>, error: DomainError) -> Result<String, DomainError> {
    let value = value.into().trim().to_owned();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_owned();
        (!value.is_empty()).then_some(value)
    })
}
