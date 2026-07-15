use std::path::PathBuf;
use std::sync::Mutex;

use prompt_domain::{
    Actor, AuditAction, Compatibility, CompatibilityStatus, EffectivenessStatus, Prompt,
    PromptContent, PromptId, PromptSource, PromptVariable, PromptVersion, SourceKind,
    ValidationRecord, VariableKind,
};
use secrecy::SecretString;
use serde::Serialize;
use tauri::State;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use prompt_ai::{CredentialStore, SystemCredentialAdapter};
use prompt_import::normalized_body_fingerprint;
use prompt_import::parse_file;
use prompt_store::{
    BackupDestination, LATEST_SCHEMA_VERSION, PromptRepository, SearchFilters, SearchPage,
    SearchQuery, create_backup, preview_restore,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualPromptDraft {
    pub title: String,
    pub body: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub variables: Vec<ManualPromptVariable>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualPromptVariable {
    pub name: String,
    pub kind: VariableKind,
    pub description: Option<String>,
    pub default_value: Option<String>,
    pub required: bool,
}

impl ManualPromptVariable {
    fn into_domain(self) -> Result<PromptVariable, String> {
        PromptVariable::new(
            self.name,
            self.kind,
            self.description,
            self.default_value,
            self.required,
        )
        .map_err(|error| error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualCompatibility {
    pub tool: String,
    pub model: Option<String>,
    pub status: CompatibilityStatus,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualValidation {
    pub status: EffectivenessStatus,
    pub rating: Option<u8>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptSearchFilterInput {
    pub favorite: Option<bool>,
    pub status: Option<prompt_domain::PromptStatus>,
    pub effectiveness: Option<EffectivenessStatus>,
    pub source_kind: Option<SourceKind>,
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub tool: Option<String>,
    pub model: Option<String>,
    pub minimum_rating: Option<u8>,
    pub updated_after: Option<String>,
    pub updated_before: Option<String>,
}

impl PromptSearchFilterInput {
    fn into_store(self) -> Result<SearchFilters, String> {
        Ok(SearchFilters {
            favorite: self.favorite,
            status: self.status,
            effectiveness: self.effectiveness,
            source_kind: self.source_kind,
            category: self.category,
            tags: self.tags,
            tool: self.tool,
            model: self.model,
            minimum_rating: self.minimum_rating,
            updated_after: parse_optional_timestamp(self.updated_after)?,
            updated_before: parse_optional_timestamp(self.updated_before)?,
        })
    }
}

fn parse_optional_timestamp(value: Option<String>) -> Result<Option<OffsetDateTime>, String> {
    value
        .map(|timestamp| {
            OffsetDateTime::parse(&timestamp, &Rfc3339).map_err(|error| error.to_string())
        })
        .transpose()
}

pub struct PromptService {
    repository: Mutex<PromptRepository>,
}

pub struct FileImportOutcome {
    pub drafts: Vec<Prompt>,
    pub skipped_duplicates: usize,
}

pub struct BackupService {
    database_path: PathBuf,
}

pub struct AiSettingsService {
    credentials: SystemCredentialAdapter,
}

impl AiSettingsService {
    pub const fn new(credentials: SystemCredentialAdapter) -> Self {
        Self { credentials }
    }

    fn status(&self, provider_id: String) -> Result<AiCredentialStatus, String> {
        Ok(AiCredentialStatus {
            configured: self
                .credentials
                .load(&provider_id)
                .map_err(|error| error.to_string())?
                .is_some(),
        })
    }

    fn save(&self, provider_id: String, secret: String) -> Result<AiCredentialStatus, String> {
        self.credentials
            .save(&provider_id, SecretString::from(secret))
            .map_err(|error| error.to_string())?;
        Ok(AiCredentialStatus { configured: true })
    }
}

impl BackupService {
    #[must_use]
    pub const fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }

    fn create_manual_backup(&self) -> Result<BackupInfo, String> {
        BackupInfo::from_store(
            create_backup(&self.database_path, BackupDestination::Manual)
                .map_err(|error| error.to_string())?,
        )
    }

    fn preview_restore(&self, path: PathBuf) -> Result<BackupRestorePreview, String> {
        Ok(BackupRestorePreview::from_store(
            preview_restore(&path, &self.database_path).map_err(|error| error.to_string())?,
        ))
    }

    fn create_pre_restore_backup(&self) -> Result<BackupInfo, String> {
        BackupInfo::from_store(
            create_backup(&self.database_path, BackupDestination::PreRestore)
                .map_err(|error| error.to_string())?,
        )
    }
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
        let variables = draft
            .variables
            .into_iter()
            .map(ManualPromptVariable::into_domain)
            .collect::<Result<Vec<_>, _>>()?;
        let content = PromptContent::with_variables(
            draft.title,
            draft.body,
            draft.description,
            draft.category,
            draft.tags,
            variables,
        )
        .map_err(|error| error.to_string())?;
        let source = PromptSource::new(SourceKind::Manual, "手动录入", None, created_at)
            .map_err(|error| error.to_string())?;
        let prompt = Prompt::new_inbox(content, source, Actor::User, created_at);
        self.save(&prompt, AuditAction::Created)?;
        Ok(prompt)
    }

    pub fn import_file_to_inbox(
        &self,
        path: PathBuf,
        created_at: OffsetDateTime,
    ) -> Result<FileImportOutcome, String> {
        let candidates = parse_file(&path).map_err(|error| error.to_string())?;
        let existing = self
            .list()?
            .into_iter()
            .map(|prompt| normalized_body_fingerprint(prompt.current_version().content().body()))
            .collect::<std::collections::HashSet<_>>();
        let mut drafts = Vec::with_capacity(candidates.len());
        let mut skipped_duplicates = 0;
        for candidate in candidates {
            if existing.contains(&normalized_body_fingerprint(&candidate.body)) {
                skipped_duplicates += 1;
                continue;
            }
            let content =
                PromptContent::new(candidate.title, candidate.body, None, None, Vec::new())
                    .map_err(|error| error.to_string())?;
            let source = PromptSource::new(
                SourceKind::FileImport,
                "文件导入",
                Some(candidate.source_path),
                created_at,
            )
            .map_err(|error| error.to_string())?;
            let prompt = Prompt::new_inbox(content, source, Actor::User, created_at);
            self.save(&prompt, AuditAction::Created)?;
            drafts.push(prompt);
        }
        Ok(FileImportOutcome {
            drafts,
            skipped_duplicates,
        })
    }

    pub fn list(&self) -> Result<Vec<Prompt>, String> {
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .list()
            .map_err(|error| error.to_string())
    }

    pub fn history(&self, id: PromptId) -> Result<Vec<PromptVersion>, String> {
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .history(id)
            .map_err(|error| error.to_string())
    }

    pub fn search(&self, query: SearchQuery) -> Result<SearchPage, String> {
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .search(query)
            .map_err(|error| error.to_string())
    }

    pub fn set_favorite(
        &self,
        id: PromptId,
        favorite: bool,
        marked_at: OffsetDateTime,
    ) -> Result<(), String> {
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .set_favorite(id, favorite, marked_at)
            .map_err(|error| error.to_string())
    }

    pub fn restore_from_backup(&self, path: PathBuf) -> Result<(), String> {
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .restore_from_backup(&path)
            .map_err(|error| error.to_string())
    }

    pub fn publish(&self, id: PromptId, published_at: OffsetDateTime) -> Result<Prompt, String> {
        self.modify(id, |prompt| {
            prompt
                .publish(EffectivenessStatus::Unverified, published_at)
                .map_err(|error| error.to_string())?;
            Ok(AuditAction::Published)
        })
    }

    pub fn archive(&self, id: PromptId, archived_at: OffsetDateTime) -> Result<Prompt, String> {
        self.modify(id, |prompt| {
            prompt
                .archive(Actor::User, archived_at)
                .map_err(|error| error.to_string())?;
            Ok(AuditAction::Archived)
        })
    }

    pub fn batch_archive(
        &self,
        ids: Vec<PromptId>,
        archived_at: OffsetDateTime,
    ) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        for id in ids {
            self.archive(id, archived_at)?;
        }
        Ok(())
    }

    pub fn soft_delete(&self, id: PromptId, deleted_at: OffsetDateTime) -> Result<Prompt, String> {
        self.modify(id, |prompt| {
            prompt
                .soft_delete(Actor::User, deleted_at)
                .map_err(|error| error.to_string())?;
            Ok(AuditAction::Deleted)
        })
    }

    pub fn recover(&self, id: PromptId, recovered_at: OffsetDateTime) -> Result<Prompt, String> {
        self.modify(id, |prompt| {
            prompt
                .recover(Actor::User, recovered_at)
                .map_err(|error| error.to_string())?;
            Ok(AuditAction::Restored)
        })
    }

    pub fn record_compatibility(
        &self,
        id: PromptId,
        metadata: ManualCompatibility,
        updated_at: OffsetDateTime,
    ) -> Result<Prompt, String> {
        let compatibility = Compatibility::new(
            metadata.tool,
            metadata.model,
            metadata.status,
            metadata.notes,
            (metadata.status == CompatibilityStatus::Confirmed).then_some(updated_at),
        )
        .map_err(|error| error.to_string())?;
        self.modify(id, |prompt| {
            prompt
                .add_compatibility(compatibility, Actor::User, updated_at)
                .map_err(|error| error.to_string())?;
            Ok(AuditAction::Revised)
        })
    }

    pub fn record_validation(
        &self,
        id: PromptId,
        metadata: ManualValidation,
        updated_at: OffsetDateTime,
    ) -> Result<Prompt, String> {
        let validation =
            ValidationRecord::new(metadata.status, metadata.rating, metadata.notes, updated_at)
                .map_err(|error| error.to_string())?;
        self.modify(id, |prompt| {
            prompt
                .record_validation(validation, Actor::User, updated_at)
                .map_err(|error| error.to_string())?;
            Ok(AuditAction::Revised)
        })
    }

    pub fn revise(
        &self,
        id: PromptId,
        draft: ManualPromptDraft,
        revised_at: OffsetDateTime,
    ) -> Result<Prompt, String> {
        let variables = draft
            .variables
            .into_iter()
            .map(ManualPromptVariable::into_domain)
            .collect::<Result<Vec<_>, _>>()?;
        let content = PromptContent::with_variables(
            draft.title,
            draft.body,
            draft.description,
            draft.category,
            draft.tags,
            variables,
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
pub struct PromptListItem {
    id: String,
    title: String,
    status: prompt_domain::PromptStatus,
    effectiveness: EffectivenessStatus,
    category: Option<String>,
    tags: Vec<String>,
    source_names: Vec<String>,
    favorite: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptHistoryItem {
    number: u32,
    body: String,
    created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    imported: usize,
    skipped_duplicates: usize,
}

impl PromptHistoryItem {
    fn from_version(version: PromptVersion) -> Result<Self, String> {
        Ok(Self {
            number: version.number(),
            body: version.content().body().to_owned(),
            created_at: format_timestamp(version.created_at())?,
        })
    }
}

impl PromptListItem {
    fn from_prompt(prompt: Prompt, favorite: bool) -> Result<Self, String> {
        let content = prompt.current_version().content();
        Ok(Self {
            id: prompt.id().value().to_string(),
            title: content.title().to_owned(),
            status: prompt.status(),
            effectiveness: prompt.effectiveness(),
            category: content.category().map(str::to_owned),
            tags: content.tags().to_vec(),
            source_names: prompt
                .sources()
                .iter()
                .map(|source| source.name().to_owned())
                .collect(),
            favorite,
            created_at: format_timestamp(prompt.created_at())?,
            updated_at: format_timestamp(prompt.updated_at())?,
        })
    }
}

fn format_timestamp(timestamp: OffsetDateTime) -> Result<String, String> {
    timestamp
        .format(&Rfc3339)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_prompts(service: State<'_, PromptService>) -> Result<Vec<PromptListItem>, String> {
    let repository = service
        .repository
        .lock()
        .map_err(|_| "prompt repository is unavailable".to_owned())?;
    repository
        .list()
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|prompt| {
            let favorite = repository
                .is_favorite(prompt.id())
                .map_err(|error| error.to_string())?;
            PromptListItem::from_prompt(prompt, favorite)
        })
        .collect()
}

#[tauri::command]
pub fn prompt_history(
    service: State<'_, PromptService>,
    id: PromptId,
) -> Result<Vec<PromptHistoryItem>, String> {
    service
        .history(id)?
        .into_iter()
        .map(PromptHistoryItem::from_version)
        .collect()
}

#[tauri::command]
pub fn restore_prompt_version(
    service: State<'_, PromptService>,
    id: PromptId,
    version_number: u32,
) -> Result<Prompt, String> {
    service.restore_version(id, version_number, OffsetDateTime::now_utc())
}

#[tauri::command]
pub fn search_prompts(
    service: State<'_, PromptService>,
    text: String,
    limit: Option<u32>,
    offset: Option<u32>,
    filters: Option<PromptSearchFilterInput>,
) -> Result<SearchPage, String> {
    let query = SearchQuery::new(text)
        .with_filters(filters.unwrap_or_default().into_store()?)
        .with_page(limit.unwrap_or(20), offset.unwrap_or(0));
    service.search(query)
}

#[tauri::command]
pub fn create_manual_prompt_draft(
    service: State<'_, PromptService>,
    draft: ManualPromptDraft,
) -> Result<Prompt, String> {
    service.create_manual_draft(draft, OffsetDateTime::now_utc())
}

#[tauri::command]
pub fn import_file_to_inbox(
    service: State<'_, PromptService>,
    path: String,
) -> Result<ImportResult, String> {
    let outcome = service.import_file_to_inbox(PathBuf::from(path), OffsetDateTime::now_utc())?;
    Ok(ImportResult {
        imported: outcome.drafts.len(),
        skipped_duplicates: outcome.skipped_duplicates,
    })
}

#[tauri::command]
pub fn publish_prompt(service: State<'_, PromptService>, id: PromptId) -> Result<Prompt, String> {
    service.publish(id, OffsetDateTime::now_utc())
}

#[tauri::command]
pub fn revise_prompt(
    service: State<'_, PromptService>,
    id: PromptId,
    draft: ManualPromptDraft,
) -> Result<Prompt, String> {
    service.revise(id, draft, OffsetDateTime::now_utc())
}

#[tauri::command]
pub fn archive_prompt(service: State<'_, PromptService>, id: PromptId) -> Result<Prompt, String> {
    service.archive(id, OffsetDateTime::now_utc())
}

#[tauri::command]
pub fn batch_archive_prompts(
    service: State<'_, PromptService>,
    ids: Vec<PromptId>,
) -> Result<(), String> {
    service.batch_archive(ids, OffsetDateTime::now_utc())
}

#[tauri::command]
pub fn soft_delete_prompt(
    service: State<'_, PromptService>,
    id: PromptId,
) -> Result<Prompt, String> {
    service.soft_delete(id, OffsetDateTime::now_utc())
}

#[tauri::command]
pub fn recover_prompt(service: State<'_, PromptService>, id: PromptId) -> Result<Prompt, String> {
    service.recover(id, OffsetDateTime::now_utc())
}

#[tauri::command]
pub fn set_prompt_favorite(
    service: State<'_, PromptService>,
    id: PromptId,
    favorite: bool,
) -> Result<(), String> {
    service.set_favorite(id, favorite, OffsetDateTime::now_utc())
}

#[tauri::command]
pub fn record_prompt_compatibility(
    service: State<'_, PromptService>,
    id: PromptId,
    metadata: ManualCompatibility,
) -> Result<Prompt, String> {
    service.record_compatibility(id, metadata, OffsetDateTime::now_utc())
}

#[tauri::command]
pub fn record_prompt_validation(
    service: State<'_, PromptService>,
    id: PromptId,
    metadata: ManualValidation,
) -> Result<Prompt, String> {
    service.record_validation(id, metadata, OffsetDateTime::now_utc())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationStatus {
    app_version: &'static str,
    database_schema_version: u32,
    offline_capable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiCredentialStatus {
    configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    path: String,
    byte_len: u64,
    schema_version: u32,
}

impl BackupInfo {
    fn from_store(backup: prompt_store::BackupMetadata) -> Result<Self, String> {
        Ok(Self {
            path: backup.path().to_string_lossy().into_owned(),
            byte_len: backup.byte_len(),
            schema_version: backup.schema_version(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRestorePreview {
    target_exists: bool,
    backup_schema_version: u32,
    backup_byte_len: u64,
}

impl BackupRestorePreview {
    const fn from_store(preview: prompt_store::RestorePreview) -> Self {
        Self {
            target_exists: preview.target_exists(),
            backup_schema_version: preview.backup_schema_version(),
            backup_byte_len: preview.backup_byte_len(),
        }
    }
}

#[tauri::command]
pub fn get_application_status() -> ApplicationStatus {
    ApplicationStatus {
        app_version: env!("CARGO_PKG_VERSION"),
        database_schema_version: LATEST_SCHEMA_VERSION,
        offline_capable: true,
    }
}

#[tauri::command]
pub fn get_ai_credential_status(
    service: State<'_, AiSettingsService>,
    provider_id: String,
) -> Result<AiCredentialStatus, String> {
    service.status(provider_id)
}

#[tauri::command]
pub fn save_ai_credential(
    service: State<'_, AiSettingsService>,
    provider_id: String,
    secret: String,
) -> Result<AiCredentialStatus, String> {
    service.save(provider_id, secret)
}

#[tauri::command]
pub fn create_manual_backup(service: State<'_, BackupService>) -> Result<BackupInfo, String> {
    service.create_manual_backup()
}

#[tauri::command]
pub fn preview_backup_restore(
    service: State<'_, BackupService>,
    path: String,
) -> Result<BackupRestorePreview, String> {
    service.preview_restore(PathBuf::from(path))
}

#[tauri::command]
pub fn restore_backup(
    prompts: State<'_, PromptService>,
    backups: State<'_, BackupService>,
    path: String,
) -> Result<BackupInfo, String> {
    preview_restore(&PathBuf::from(&path), &backups.database_path)
        .map_err(|error| error.to_string())?;
    let safety_backup = backups.create_pre_restore_backup()?;
    prompts.restore_from_backup(PathBuf::from(path))?;
    Ok(safety_backup)
}

#[cfg(test)]
mod backup_service_tests {
    use super::*;

    #[test]
    fn desktop_backup_service_creates_and_previews_a_verified_backup() {
        let directory = tempfile::tempdir().unwrap();
        let database_path = directory.path().join("prompt-hub.db");
        let database = prompt_store::Database::open(&database_path).unwrap();
        drop(database);
        let service = BackupService::new(database_path);

        let backup = service.create_manual_backup().unwrap();
        let preview = service.preview_restore(PathBuf::from(backup.path)).unwrap();

        assert!(preview.target_exists);
        assert_eq!(preview.backup_schema_version, LATEST_SCHEMA_VERSION);
        assert!(preview.backup_byte_len > 0);
    }
}
