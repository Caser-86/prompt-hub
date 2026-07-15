use std::sync::Mutex;

use prompt_domain::{
    Actor, AuditAction, EffectivenessStatus, Prompt, PromptContent, PromptId, PromptSource,
    SourceKind,
};
use serde::Serialize;
use time::OffsetDateTime;

use prompt_store::{LATEST_SCHEMA_VERSION, PromptRepository};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualPromptDraft {
    pub title: String,
    pub body: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
}

pub struct PromptService {
    repository: Mutex<PromptRepository>,
}

impl PromptService {
    #[must_use]
    pub const fn new(repository: PromptRepository) -> Self {
        Self {
            repository: Mutex::new(repository),
        }
    }

    pub fn create_manual_draft(
        &self,
        draft: ManualPromptDraft,
        created_at: OffsetDateTime,
    ) -> Result<Prompt, String> {
        let content = PromptContent::new(
            draft.title,
            draft.body,
            draft.description,
            draft.category,
            draft.tags,
        )
        .map_err(|error| error.to_string())?;
        let source = PromptSource::new(SourceKind::Manual, "手动录入", None, created_at)
            .map_err(|error| error.to_string())?;
        let prompt = Prompt::new_inbox(content, source, Actor::User, created_at);
        self.save(&prompt, AuditAction::Created)?;
        Ok(prompt)
    }

    pub fn publish(&self, id: PromptId, published_at: OffsetDateTime) -> Result<Prompt, String> {
        self.modify(id, |prompt| {
            prompt
                .publish(EffectivenessStatus::Unverified, published_at)
                .map_err(|error| error.to_string())?;
            Ok(AuditAction::Published)
        })
    }

    pub fn revise(
        &self,
        id: PromptId,
        draft: ManualPromptDraft,
        revised_at: OffsetDateTime,
    ) -> Result<Prompt, String> {
        let content = PromptContent::new(
            draft.title,
            draft.body,
            draft.description,
            draft.category,
            draft.tags,
        )
        .map_err(|error| error.to_string())?;
        self.modify(id, |prompt| {
            prompt
                .revise(content, Actor::User, revised_at)
                .map_err(|error| error.to_string())?;
            Ok(AuditAction::Revised)
        })
    }

    pub fn restore_version(
        &self,
        id: PromptId,
        version_number: u32,
        restored_at: OffsetDateTime,
    ) -> Result<Prompt, String> {
        let history = self
            .repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .history(id)
            .map_err(|error| error.to_string())?;
        let version = history
            .iter()
            .find(|version| version.number() == version_number)
            .ok_or_else(|| "requested prompt version was not found".to_owned())?;
        self.modify(id, |prompt| {
            prompt
                .restore(version, Actor::User, restored_at)
                .map_err(|error| error.to_string())?;
            Ok(AuditAction::Restored)
        })
    }

    fn modify(
        &self,
        id: PromptId,
        operation: impl FnOnce(&mut Prompt) -> Result<AuditAction, String>,
    ) -> Result<Prompt, String> {
        let mut repository = self
            .repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?;
        let mut prompt = repository
            .get(id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "prompt was not found".to_owned())?;
        let action = operation(&mut prompt)?;
        repository
            .save(&prompt, action)
            .map_err(|error| error.to_string())?;
        Ok(prompt)
    }

    fn save(&self, prompt: &Prompt, action: AuditAction) -> Result<(), String> {
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .save(prompt, action)
            .map_err(|error| error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationStatus {
    app_version: &'static str,
    database_schema_version: u32,
    offline_capable: bool,
}

#[tauri::command]
pub fn get_application_status() -> ApplicationStatus {
    ApplicationStatus {
        app_version: env!("CARGO_PKG_VERSION"),
        database_schema_version: LATEST_SCHEMA_VERSION,
        offline_capable: true,
    }
}
