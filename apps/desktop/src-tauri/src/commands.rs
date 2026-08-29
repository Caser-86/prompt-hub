use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

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
use tokio::sync::watch;

use prompt_ai::{
    CredentialStore, DraftGenerator, GenerationRequest, OpenAiCompatibleProvider,
    SystemCredentialAdapter,
};
use prompt_import::{
    ImportCandidate, UrlPolicy, fetch_url, normalized_body_fingerprint, parse_file, scan_folder,
};
use prompt_skill::{
    GitSkillSource, InstallMode, InstallRequest, install_skill as install_reviewed_skill,
    scan_skill, snapshot_git_skill,
};
use prompt_store::{
    BackupDestination, LATEST_SCHEMA_VERSION, PromptRepository, PromptUsageStats, SearchFilters,
    SearchPage, SearchQuery, SearchSort, SkillRepository, SkillReviewStatus, SkillSource,
    StoredSkill, create_backup, create_backup_in_directory, preview_restore, prune_backups,
};
use uuid::Uuid;

use crate::bootstrap::{self, BootstrapRuntime, BootstrapStatus};

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

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiGenerationRequestInput {
    pub task_id: String,
    pub endpoint: String,
    pub provider_id: String,
    pub instruction: String,
    pub input_summary: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConnectionRequestInput {
    pub endpoint: String,
    pub provider_id: String,
    pub model: String,
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

pub struct SkillService {
    repository: Mutex<SkillRepository>,
    backup_root: PathBuf,
    snapshot_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillReviewInput {
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInstallInput {
    pub target_root: String,
    pub destination_name: String,
    pub replace_after_backup: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitSkillCollectionInput {
    pub repository_url: String,
    pub commit: String,
    pub subdirectory: String,
}

#[derive(Default)]
pub struct AiCancellationRegistry {
    tasks: Mutex<HashMap<String, watch::Sender<bool>>>,
}

impl AiCancellationRegistry {
    pub fn register(&self, task_id: String) -> Result<watch::Receiver<bool>, String> {
        let (sender, receiver) = watch::channel(false);
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|_| "AI cancellation registry is unavailable".to_owned())?;
        if tasks.insert(task_id, sender).is_some() {
            return Err("AI generation task is already active".to_owned());
        }
        Ok(receiver)
    }

    #[must_use]
    pub fn cancel(&self, task_id: &str) -> bool {
        self.tasks
            .lock()
            .ok()
            .and_then(|tasks| tasks.get(task_id).cloned())
            .is_some_and(|sender| sender.send(true).is_ok())
    }

    pub fn finish(&self, task_id: &str) {
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.remove(task_id);
        }
    }
}

pub struct FileImportOutcome {
    pub drafts: Vec<Prompt>,
    pub skipped_duplicates: usize,
    pub failed: usize,
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

    fn create_manual_backup(&self, directory: Option<PathBuf>) -> Result<BackupInfo, String> {
        let backup = match directory {
            Some(directory) => create_backup_in_directory(
                &self.database_path,
                &directory,
                BackupDestination::Manual,
            ),
            None => create_backup(&self.database_path, BackupDestination::Manual),
        };
        BackupInfo::from_store(backup.map_err(|error| error.to_string())?)
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

    fn mcp_setup(&self) -> McpSetupInfo {
        McpSetupInfo {
            database_path: self.database_path.to_string_lossy().into_owned(),
            database_available: prompt_store::Database::open(&self.database_path).is_ok(),
            configuration: serde_json::json!({
                "mcpServers": {
                    "prompt-hub": {
                        "command": "prompt-mcp",
                        "env": { "PROMPT_HUB_DATABASE_PATH": self.database_path }
                    }
                }
            })
            .to_string(),
        }
    }

    fn prune_backups(&self, retain: usize) -> Result<usize, String> {
        prune_backups(&self.database_path, retain).map_err(|error| error.to_string())
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

    pub async fn create_ai_draft(
        &self,
        request: AiGenerationRequestInput,
        generated_at: OffsetDateTime,
        cancellation: watch::Receiver<bool>,
    ) -> Result<Prompt, String> {
        let provider = OpenAiCompatibleProvider::new(request.endpoint, Duration::from_secs(45))
            .map_err(|error| error.to_string())?;
        let credentials = SystemCredentialAdapter::new("Prompt Hub", "default")
            .map_err(|error| error.to_string())?;
        let generator = DraftGenerator::new(provider, credentials);
        let draft = generator
            .generate_cancellable(
                &request.provider_id,
                GenerationRequest {
                    instruction: request.instruction,
                    input_summary: request.input_summary,
                    model: request.model,
                },
                generated_at,
                cancellation.clone(),
            )
            .await
            .map_err(|error| error.to_string())?;
        if *cancellation.borrow() {
            return Err("AI generation was cancelled".to_owned());
        }
        let content = PromptContent::new(
            draft.title(),
            draft.body(),
            Some(format!(
                "AI 生成；模型：{}；输入摘要：{}",
                draft.model(),
                draft.input_summary()
            )),
            None,
            Vec::new(),
        )
        .map_err(|error| error.to_string())?;
        let source = PromptSource::new(
            SourceKind::AiGenerated,
            "AI 生成",
            Some(format!("模型：{}", draft.model())),
            draft.generated_at(),
        )
        .map_err(|error| error.to_string())?;
        let prompt = Prompt::new_inbox(content, source, Actor::User, draft.generated_at());
        self.save(&prompt, AuditAction::Created)?;
        Ok(prompt)
    }

    pub async fn test_ai_connection(
        &self,
        request: AiConnectionRequestInput,
    ) -> Result<(), String> {
        let provider = OpenAiCompatibleProvider::new(request.endpoint, Duration::from_secs(15))
            .map_err(|error| error.to_string())?;
        let credentials = SystemCredentialAdapter::new("Prompt Hub", "default")
            .map_err(|error| error.to_string())?;
        let credential = credentials
            .load(&request.provider_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "AI credential is not configured".to_owned())?;
        provider
            .test_connection(credential)
            .await
            .map_err(|error| error.to_string())
    }

    pub fn create_ai_optimization_draft(
        &self,
        original_id: PromptId,
        title: String,
        body: String,
        model: String,
        generated_at: OffsetDateTime,
    ) -> Result<Prompt, String> {
        let original = self
            .repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .get(original_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "prompt was not found".to_owned())?;
        let content = PromptContent::new(
            title,
            body,
            Some(format!(
                "AI 优化；模型：{}；原提示词：{}",
                model,
                original.id().value()
            )),
            original
                .current_version()
                .content()
                .category()
                .map(str::to_owned),
            original.current_version().content().tags().to_vec(),
        )
        .map_err(|error| error.to_string())?;
        let source = PromptSource::new(
            SourceKind::AiGenerated,
            "AI 优化",
            Some(format!(
                "原提示词：{}；模型：{}",
                original.id().value(),
                model
            )),
            generated_at,
        )
        .map_err(|error| error.to_string())?;
        let prompt = Prompt::new_inbox(content, source, Actor::User, generated_at);
        self.save(&prompt, AuditAction::Created)?;
        Ok(prompt)
    }

    pub async fn optimize_ai_prompt(
        &self,
        original_id: PromptId,
        request: AiGenerationRequestInput,
        generated_at: OffsetDateTime,
        cancellation: watch::Receiver<bool>,
    ) -> Result<Prompt, String> {
        let original_body = self
            .repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .get(original_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "prompt was not found".to_owned())?
            .current_version()
            .content()
            .body()
            .to_owned();
        let provider = OpenAiCompatibleProvider::new(request.endpoint, Duration::from_secs(45))
            .map_err(|error| error.to_string())?;
        let credentials = SystemCredentialAdapter::new("Prompt Hub", "default")
            .map_err(|error| error.to_string())?;
        let draft = DraftGenerator::new(provider, credentials)
            .generate_cancellable(
                &request.provider_id,
                GenerationRequest {
                    instruction: request.instruction,
                    input_summary: original_body,
                    model: request.model,
                },
                generated_at,
                cancellation.clone(),
            )
            .await
            .map_err(|error| error.to_string())?;
        if *cancellation.borrow() {
            return Err("AI generation was cancelled".to_owned());
        }
        self.create_ai_optimization_draft(
            original_id,
            draft.title().to_owned(),
            draft.body().to_owned(),
            draft.model().to_owned(),
            generated_at,
        )
    }

    pub fn import_file_to_inbox(
        &self,
        path: PathBuf,
        created_at: OffsetDateTime,
    ) -> Result<FileImportOutcome, String> {
        self.import_path_to_inbox(path, "file_import", "文件导入", parse_file, created_at)
    }

    pub fn import_folder_to_inbox(
        &self,
        path: PathBuf,
        created_at: OffsetDateTime,
    ) -> Result<FileImportOutcome, String> {
        self.import_path_to_inbox(path, "folder_import", "文件夹导入", scan_folder, created_at)
    }

    pub fn import_url_to_inbox(
        &self,
        url: String,
        created_at: OffsetDateTime,
    ) -> Result<FileImportOutcome, String> {
        let job_id = self
            .repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .start_import_job("web_url", &url, None, created_at)
            .map_err(|error| error.to_string())?
            .id()
            .to_owned();
        let fetched = match fetch_url(&url, UrlPolicy::default()) {
            Ok(fetched) => fetched,
            Err(error) => {
                let error = error.to_string();
                self.record_url_import_item(
                    &job_id,
                    &url,
                    None,
                    None,
                    "failed",
                    &[],
                    Some(&error),
                    None,
                    created_at,
                )?;
                self.finish_import_job(&job_id, "failed", 0, 0, 1, created_at)?;
                return Err(error);
            }
        };
        let fingerprint = normalized_body_fingerprint(&fetched.text);
        let canonical_url = fetched.canonical_url;
        let title = fetched.title.unwrap_or_else(|| "网页导入提示词".to_owned());
        let warnings = fetched.warnings;
        let existing = match self.list() {
            Ok(prompts) => prompts,
            Err(error) => {
                self.record_url_import_item(
                    &job_id,
                    &canonical_url,
                    Some(&fingerprint),
                    Some(&title),
                    "failed",
                    &warnings,
                    Some(&error),
                    None,
                    created_at,
                )?;
                self.finish_import_job(&job_id, "completed_with_errors", 0, 0, 1, created_at)?;
                return Err(error);
            }
        };
        if existing.into_iter().any(|prompt| {
            normalized_body_fingerprint(prompt.current_version().content().body()) == fingerprint
        }) {
            self.record_url_import_item(
                &job_id,
                &canonical_url,
                Some(&fingerprint),
                Some(&title),
                "duplicate",
                &warnings,
                None,
                None,
                created_at,
            )?;
            self.finish_import_job(&job_id, "completed", 0, 1, 0, created_at)?;
            return Ok(FileImportOutcome {
                drafts: Vec::new(),
                skipped_duplicates: 1,
                failed: 0,
            });
        }
        let raw_excerpt = source_excerpt(&fetched.text);
        let content = match PromptContent::new(
            title.clone(),
            fetched.text,
            if warnings.is_empty() {
                None
            } else {
                Some(warnings.join("；"))
            },
            None,
            Vec::new(),
        ) {
            Ok(content) => content,
            Err(error) => {
                let error = error.to_string();
                self.record_url_import_item(
                    &job_id,
                    &canonical_url,
                    Some(&fingerprint),
                    Some(&title),
                    "failed",
                    &warnings,
                    Some(&error),
                    None,
                    created_at,
                )?;
                self.finish_import_job(&job_id, "completed_with_errors", 0, 0, 1, created_at)?;
                return Err(error);
            }
        };
        let source = match PromptSource::with_provenance(
            SourceKind::WebUrl,
            "网页导入",
            Some(canonical_url.clone()),
            fetched.retrieved_at,
            Some(raw_excerpt),
            Some(job_id.clone()),
        ) {
            Ok(source) => source,
            Err(error) => {
                let error = error.to_string();
                self.record_url_import_item(
                    &job_id,
                    &canonical_url,
                    Some(&fingerprint),
                    Some(&title),
                    "failed",
                    &warnings,
                    Some(&error),
                    None,
                    created_at,
                )?;
                self.finish_import_job(&job_id, "completed_with_errors", 0, 0, 1, created_at)?;
                return Err(error);
            }
        };
        let prompt = Prompt::new_imported_inbox(content, source, Actor::User, created_at);
        if let Err(error) = self.save(&prompt, AuditAction::Created) {
            self.record_url_import_item(
                &job_id,
                &canonical_url,
                Some(&fingerprint),
                Some(prompt.current_version().content().title()),
                "failed",
                &warnings,
                Some(&error),
                None,
                created_at,
            )?;
            self.finish_import_job(&job_id, "completed_with_errors", 0, 0, 1, created_at)?;
            return Err(error);
        }
        self.record_url_import_item(
            &job_id,
            &canonical_url,
            Some(&fingerprint),
            Some(prompt.current_version().content().title()),
            "imported",
            &warnings,
            None,
            Some(prompt.id()),
            created_at,
        )?;
        self.finish_import_job(&job_id, "completed", 1, 0, 0, created_at)?;
        Ok(FileImportOutcome {
            drafts: vec![prompt],
            skipped_duplicates: 0,
            failed: 0,
        })
    }

    fn import_path_to_inbox(
        &self,
        path: PathBuf,
        source_kind: &str,
        source_name: &str,
        parse: fn(&std::path::Path) -> Result<Vec<ImportCandidate>, prompt_import::FileImportError>,
        created_at: OffsetDateTime,
    ) -> Result<FileImportOutcome, String> {
        let job_id = self
            .repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .start_import_job(source_kind, &path.to_string_lossy(), None, created_at)
            .map_err(|error| error.to_string())?
            .id()
            .to_owned();
        let candidates = match parse(&path) {
            Ok(candidates) => candidates,
            Err(error) => {
                self.finish_import_job(&job_id, "failed", 0, 0, 1, created_at)?;
                return Err(error.to_string());
            }
        };
        let outcome =
            self.import_candidates_to_inbox(candidates, source_name, &job_id, created_at)?;
        self.finish_import_job(
            &job_id,
            if outcome.failed == 0 {
                "completed"
            } else {
                "completed_with_errors"
            },
            outcome.drafts.len(),
            outcome.skipped_duplicates,
            outcome.failed,
            created_at,
        )?;
        Ok(outcome)
    }

    fn import_candidates_to_inbox(
        &self,
        candidates: Vec<ImportCandidate>,
        source_name: &str,
        job_id: &str,
        created_at: OffsetDateTime,
    ) -> Result<FileImportOutcome, String> {
        let existing = self
            .list()?
            .into_iter()
            .map(|prompt| normalized_body_fingerprint(prompt.current_version().content().body()))
            .collect::<std::collections::HashSet<_>>();
        let mut fingerprints = existing;
        let candidate_count = candidates.len();
        let mut drafts = Vec::with_capacity(candidates.len());
        let mut skipped_duplicates = 0;
        for candidate in candidates {
            let fingerprint = normalized_body_fingerprint(&candidate.body);
            if !fingerprints.insert(fingerprint) {
                skipped_duplicates += 1;
                self.record_import_item(job_id, &candidate, "duplicate", None, None, created_at)?;
                continue;
            }
            let content = match PromptContent::new(
                candidate.title.clone(),
                candidate.body.clone(),
                None,
                None,
                Vec::new(),
            ) {
                Ok(content) => content,
                Err(error) => {
                    self.record_import_item(
                        job_id,
                        &candidate,
                        "failed",
                        Some(error.to_string()),
                        None,
                        created_at,
                    )?;
                    continue;
                }
            };
            let source = PromptSource::with_provenance(
                SourceKind::FileImport,
                source_name,
                Some(candidate.source_path.clone()),
                created_at,
                Some(source_excerpt(&candidate.body)),
                Some(job_id.to_owned()),
            )
            .map_err(|error| error.to_string())?;
            let prompt = Prompt::new_imported_inbox(content, source, Actor::User, created_at);
            if let Err(error) = self.save(&prompt, AuditAction::Created) {
                self.record_import_item(
                    job_id,
                    &candidate,
                    "failed",
                    Some(error),
                    None,
                    created_at,
                )?;
                continue;
            }
            self.record_import_item(
                job_id,
                &candidate,
                "imported",
                None,
                Some(prompt.id()),
                created_at,
            )?;
            drafts.push(prompt);
        }
        let imported = drafts.len();
        Ok(FileImportOutcome {
            drafts,
            skipped_duplicates,
            failed: candidate_count.saturating_sub(imported + skipped_duplicates),
        })
    }

    fn record_import_item(
        &self,
        job_id: &str,
        candidate: &ImportCandidate,
        outcome: &str,
        error: Option<String>,
        prompt_id: Option<PromptId>,
        recorded_at: OffsetDateTime,
    ) -> Result<(), String> {
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .record_import_job_item(prompt_store::ImportJobItemRecord {
                job_id,
                source_path: &candidate.source_path,
                body_fingerprint: Some(&normalized_body_fingerprint(&candidate.body)),
                title: Some(&candidate.title),
                outcome,
                warnings_json: &serde_json::to_string(&candidate.warnings)
                    .map_err(|error| error.to_string())?,
                error_message: error.as_deref(),
                prompt_id,
                recorded_at,
            })
            .map_err(|error| error.to_string())
    }

    #[allow(clippy::too_many_arguments)]
    fn record_url_import_item(
        &self,
        job_id: &str,
        source_path: &str,
        body_fingerprint: Option<&str>,
        title: Option<&str>,
        outcome: &str,
        warnings: &[String],
        error_message: Option<&str>,
        prompt_id: Option<PromptId>,
        recorded_at: OffsetDateTime,
    ) -> Result<(), String> {
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .record_import_job_item(prompt_store::ImportJobItemRecord {
                job_id,
                source_path,
                body_fingerprint,
                title,
                outcome,
                warnings_json: &serde_json::to_string(warnings)
                    .map_err(|error| error.to_string())?,
                error_message,
                prompt_id,
                recorded_at,
            })
            .map_err(|error| error.to_string())
    }

    fn finish_import_job(
        &self,
        job_id: &str,
        status: &str,
        imported: usize,
        skipped_duplicates: usize,
        failed: usize,
        completed_at: OffsetDateTime,
    ) -> Result<(), String> {
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .finish_import_job(
                job_id,
                status,
                &serde_json::json!({
                    "imported": imported,
                    "skippedDuplicates": skipped_duplicates,
                    "failed": failed,
                })
                .to_string(),
                completed_at,
            )
            .map_err(|error| error.to_string())
    }

    pub fn list(&self) -> Result<Vec<Prompt>, String> {
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .list()
            .map_err(|error| error.to_string())
    }

    pub fn record_use(
        &self,
        id: PromptId,
        used_at: OffsetDateTime,
    ) -> Result<PromptUsageStats, String> {
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .record_use(id, used_at)
            .map_err(|error| error.to_string())
    }

    pub fn merge_legacy_usage(
        &self,
        entries: Vec<(PromptId, i64)>,
    ) -> Result<Vec<PromptUsageStats>, String> {
        let mut repository = self
            .repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?;
        entries
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .map(|(id, count)| {
                repository
                    .merge_legacy_usage(id, count)
                    .map_err(|error| error.to_string())
            })
            .collect()
    }

    pub fn recent_import_jobs(&self) -> Result<Vec<prompt_store::ImportJob>, String> {
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .recent_import_jobs(10)
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

    pub fn permanently_delete(
        &self,
        id: PromptId,
        backups: &BackupService,
    ) -> Result<BackupInfo, String> {
        let backup = BackupInfo::from_store(
            create_backup(&backups.database_path, BackupDestination::PermanentDelete)
                .map_err(|error| error.to_string())?,
        )?;
        self.repository
            .lock()
            .map_err(|_| "prompt repository is unavailable".to_owned())?
            .permanently_delete(id)
            .map_err(|error| error.to_string())?;
        Ok(backup)
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

impl SkillService {
    #[must_use]
    pub const fn new(repository: SkillRepository) -> Self {
        Self {
            repository: Mutex::new(repository),
            backup_root: PathBuf::new(),
            snapshot_root: PathBuf::new(),
        }
    }

    #[must_use]
    pub fn with_storage_roots(
        repository: SkillRepository,
        backup_root: PathBuf,
        snapshot_root: PathBuf,
    ) -> Self {
        Self {
            repository: Mutex::new(repository),
            backup_root,
            snapshot_root,
        }
    }

    pub fn collect_local_folder(&self, path: PathBuf) -> Result<SkillListItem, String> {
        if !path.is_absolute() {
            return Err("Skill folder path must be absolute".to_owned());
        }
        let path = path
            .canonicalize()
            .map_err(|_| "Skill folder is unavailable".to_owned())?;
        let candidate = scan_skill(&path).map_err(|error| error.to_string())?;
        let stored = self
            .repository
            .lock()
            .map_err(|_| "Skill repository is unavailable".to_owned())?
            .save_candidate(
                &candidate,
                &SkillSource::local_directory(path.to_string_lossy()),
                OffsetDateTime::now_utc(),
            )
            .map_err(|error| error.to_string())?;
        SkillListItem::from_stored(&stored)
    }

    pub fn collect_git_candidate(
        &self,
        input: GitSkillCollectionInput,
    ) -> Result<SkillListItem, String> {
        if self.snapshot_root.as_os_str().is_empty() {
            return Err("Skill snapshot storage is unavailable".to_owned());
        }
        let source = GitSkillSource::new(
            &input.repository_url,
            &input.commit,
            PathBuf::from(input.subdirectory),
        )
        .map_err(|error| error.to_string())?;
        std::fs::create_dir_all(&self.snapshot_root)
            .map_err(|_| "Skill snapshot storage is unavailable".to_owned())?;
        let snapshot = self.snapshot_root.join(Uuid::now_v7().to_string());
        let candidate =
            snapshot_git_skill(&source, &snapshot).map_err(|error| error.to_string())?;
        let stored = self
            .repository
            .lock()
            .map_err(|_| "Skill repository is unavailable".to_owned())?
            .save_candidate_with_snapshot(
                &candidate,
                &SkillSource::git_repository(source.repository_url(), source.commit()),
                Some(&snapshot.to_string_lossy()),
                OffsetDateTime::now_utc(),
            )
            .map_err(|error| error.to_string())?;
        SkillListItem::from_stored(&stored)
    }

    pub fn list(&self) -> Result<Vec<SkillListItem>, String> {
        self.repository
            .lock()
            .map_err(|_| "Skill repository is unavailable".to_owned())?
            .list_skills()
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(SkillListItem::from_summary)
            .collect()
    }

    pub fn get(&self, id: &str) -> Result<Option<SkillDetail>, String> {
        self.repository
            .lock()
            .map_err(|_| "Skill repository is unavailable".to_owned())?
            .get_skill(id)
            .map_err(|error| error.to_string())?
            .map(|skill| SkillDetail::from_stored(&skill))
            .transpose()
    }

    pub fn review(&self, id: &str, input: SkillReviewInput) -> Result<(), String> {
        let status = parse_skill_review_status(&input.status)?;
        self.repository
            .lock()
            .map_err(|_| "Skill repository is unavailable".to_owned())?
            .set_review(
                id,
                status,
                input.notes.as_deref(),
                OffsetDateTime::now_utc(),
            )
            .map_err(|error| error.to_string())
    }

    pub fn set_favorite(&self, id: &str, favorite: bool) -> Result<(), String> {
        self.repository
            .lock()
            .map_err(|_| "Skill repository is unavailable".to_owned())?
            .set_favorite(id, favorite, OffsetDateTime::now_utc())
            .map_err(|error| error.to_string())
    }

    pub fn install(
        &self,
        id: &str,
        input: SkillInstallInput,
    ) -> Result<SkillInstallationItem, String> {
        let target_root = PathBuf::from(&input.target_root);
        if !target_root.is_absolute() {
            return Err("Skill installation target must be an absolute path".to_owned());
        }
        let mut repository = self
            .repository
            .lock()
            .map_err(|_| "Skill repository is unavailable".to_owned())?;
        let skill = repository
            .get_skill(id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "Skill was not found".to_owned())?;
        if skill.review_status() != SkillReviewStatus::Approved {
            return Err("Skill must be approved before installation".to_owned());
        }
        let source = skill.snapshot_path().unwrap_or(skill.source().location());
        let source = PathBuf::from(source);
        let backup_root = if self.backup_root.as_os_str().is_empty() {
            target_root.join(".prompt-hub-skill-backups")
        } else {
            self.backup_root.clone()
        };
        let receipt = install_reviewed_skill(InstallRequest {
            source: &source,
            target_root: &target_root,
            backup_root: &backup_root,
            destination_name: &input.destination_name,
            expected_content_hash: skill.content_hash(),
            mode: if input.replace_after_backup {
                InstallMode::ReplaceAfterBackup
            } else {
                InstallMode::FailIfExists
            },
        })
        .map_err(|error| error.to_string())?;
        let now = OffsetDateTime::now_utc();
        repository
            .record_installation(
                id,
                &input.target_root,
                &receipt.install_path().display().to_string(),
                receipt.installed_hash(),
                receipt
                    .backup_path()
                    .map(|path| path.display().to_string())
                    .as_deref(),
                now,
            )
            .map_err(|error| error.to_string())?;
        Ok(SkillInstallationItem {
            install_path: receipt.install_path().display().to_string(),
            backup_path: receipt.backup_path().map(|path| path.display().to_string()),
            installed_hash: receipt.installed_hash().to_owned(),
        })
    }

    pub fn verify_installation(&self, id: &str) -> Result<SkillInstallationVerification, String> {
        let mut repository = self
            .repository
            .lock()
            .map_err(|_| "Skill repository is unavailable".to_owned())?;
        let installation = repository
            .installation(id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "Skill installation was not found".to_owned())?;
        let state = match scan_skill(&PathBuf::from(installation.install_path())) {
            Ok(candidate) if candidate.content_hash() == installation.installed_hash() => {
                "matching"
            }
            Ok(_) => "drifted",
            Err(_) => "unavailable",
        };
        repository
            .mark_installation_verified(id, OffsetDateTime::now_utc())
            .map_err(|error| error.to_string())?;
        Ok(SkillInstallationVerification {
            state: state.to_owned(),
        })
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
    sources: Vec<PromptSourceItem>,
    applicable_tools: Vec<String>,
    applicable_models: Vec<String>,
    rating: Option<u8>,
    favorite: bool,
    use_count: i64,
    last_used_at: Option<String>,
    imported_at: Option<String>,
    last_validated_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptSourceItem {
    kind: SourceKind,
    name: String,
    location: Option<String>,
    collected_at: String,
    raw_excerpt: Option<String>,
    import_job_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyPromptUsageInput {
    pub id: String,
    pub use_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptUsageItem {
    use_count: i64,
    last_used_at: Option<String>,
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
pub struct SkillSourceItem {
    pub kind: String,
    pub location: String,
    pub revision: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillListItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: SkillSourceItem,
    pub risks: Vec<String>,
    pub review_status: String,
    pub favorite: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillFileItem {
    pub relative_path: String,
    pub bytes: u64,
    pub sha256: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDetail {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: SkillSourceItem,
    pub risks: Vec<String>,
    pub review_status: String,
    pub review_notes: Option<String>,
    pub favorite: bool,
    pub skill_markdown: String,
    pub files: Vec<SkillFileItem>,
    pub content_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInstallationItem {
    pub install_path: String,
    pub backup_path: Option<String>,
    pub installed_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInstallationVerification {
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    imported: usize,
    skipped_duplicates: usize,
    failed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportJobSummary {
    id: String,
    source_kind: String,
    source_path: Option<String>,
    status: String,
    started_at: String,
    completed_at: Option<String>,
    imported: usize,
    skipped_duplicates: usize,
    failed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSetupInfo {
    database_path: String,
    database_available: bool,
    configuration: String,
}

impl ImportJobSummary {
    fn from_store(job: prompt_store::ImportJob) -> Result<Self, String> {
        let diagnostics: serde_json::Value =
            serde_json::from_str(job.diagnostics_json()).unwrap_or_default();
        let count = |key| {
            diagnostics
                .get(key)
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize
        };
        Ok(Self {
            id: job.id().to_owned(),
            source_kind: job.source_kind().to_owned(),
            source_path: job.source_path().map(str::to_owned),
            status: job.status().to_owned(),
            started_at: format_timestamp(job.started_at())?,
            completed_at: job.completed_at().map(format_timestamp).transpose()?,
            imported: count("imported"),
            skipped_duplicates: count("skippedDuplicates"),
            failed: count("failed"),
        })
    }
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

impl SkillSourceItem {
    fn from_source(source: &SkillSource) -> Self {
        Self {
            kind: source.kind().to_owned(),
            location: source.location().to_owned(),
            revision: source.revision().map(str::to_owned),
        }
    }
}

impl SkillListItem {
    fn from_stored(skill: &StoredSkill) -> Result<Self, String> {
        Ok(Self {
            id: skill.id().to_owned(),
            name: skill.name().to_owned(),
            description: skill.description().to_owned(),
            source: SkillSourceItem::from_source(skill.source()),
            risks: skill.risks().to_vec(),
            review_status: skill_review_status_name(skill.review_status()).to_owned(),
            favorite: skill.favorite(),
            updated_at: format_timestamp(skill.updated_at())?,
        })
    }

    fn from_summary(skill: prompt_store::SkillSummary) -> Result<Self, String> {
        Ok(Self {
            id: skill.id().to_owned(),
            name: skill.name().to_owned(),
            description: skill.description().to_owned(),
            source: SkillSourceItem::from_source(skill.source()),
            risks: skill.risks().to_vec(),
            review_status: skill_review_status_name(skill.review_status()).to_owned(),
            favorite: skill.favorite(),
            updated_at: format_timestamp(skill.updated_at())?,
        })
    }
}

impl SkillDetail {
    fn from_stored(skill: &StoredSkill) -> Result<Self, String> {
        Ok(Self {
            id: skill.id().to_owned(),
            name: skill.name().to_owned(),
            description: skill.description().to_owned(),
            source: SkillSourceItem::from_source(skill.source()),
            risks: skill.risks().to_vec(),
            review_status: skill_review_status_name(skill.review_status()).to_owned(),
            review_notes: skill.review_notes().map(str::to_owned),
            favorite: skill.favorite(),
            skill_markdown: skill.skill_markdown().to_owned(),
            files: skill
                .files()
                .iter()
                .map(|file| SkillFileItem {
                    relative_path: file.relative_path().to_owned(),
                    bytes: file.bytes(),
                    sha256: file.sha256().to_owned(),
                    kind: file.kind().to_owned(),
                })
                .collect(),
            content_hash: skill.content_hash().to_owned(),
            created_at: format_timestamp(skill.created_at())?,
            updated_at: format_timestamp(skill.updated_at())?,
        })
    }
}

impl PromptListItem {
    fn from_prompt(
        prompt: Prompt,
        favorite: bool,
        usage: PromptUsageStats,
    ) -> Result<Self, String> {
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
            sources: prompt
                .sources()
                .iter()
                .map(|source| {
                    Ok(PromptSourceItem {
                        kind: source.kind(),
                        name: source.name().to_owned(),
                        location: source.location().map(str::to_owned),
                        collected_at: format_timestamp(source.collected_at())?,
                        raw_excerpt: source.raw_excerpt().map(str::to_owned),
                        import_job_id: source.import_job_id().map(str::to_owned),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
            applicable_tools: prompt
                .compatibilities()
                .iter()
                .map(|compatibility| compatibility.tool().to_owned())
                .collect(),
            applicable_models: prompt
                .compatibilities()
                .iter()
                .filter_map(|compatibility| compatibility.model().map(str::to_owned))
                .collect(),
            rating: prompt
                .validations()
                .last()
                .and_then(|validation| validation.rating),
            favorite,
            use_count: usage.use_count(),
            last_used_at: usage.last_used_at().map(format_timestamp).transpose()?,
            imported_at: prompt.imported_at().map(format_timestamp).transpose()?,
            last_validated_at: prompt
                .last_validated_at()
                .map(format_timestamp)
                .transpose()?,
            created_at: format_timestamp(prompt.created_at())?,
            updated_at: format_timestamp(prompt.updated_at())?,
        })
    }
}

impl PromptUsageItem {
    fn from_store(stats: PromptUsageStats) -> Result<Self, String> {
        Ok(Self {
            use_count: stats.use_count(),
            last_used_at: stats.last_used_at().map(format_timestamp).transpose()?,
        })
    }
}

fn format_timestamp(timestamp: OffsetDateTime) -> Result<String, String> {
    timestamp
        .format(&Rfc3339)
        .map_err(|error| error.to_string())
}

fn source_excerpt(body: &str) -> String {
    body.chars().take(512).collect()
}

fn parse_skill_review_status(value: &str) -> Result<SkillReviewStatus, String> {
    match value {
        "pending_review" => Ok(SkillReviewStatus::PendingReview),
        "approved" => Ok(SkillReviewStatus::Approved),
        "rejected" => Ok(SkillReviewStatus::Rejected),
        "risk_pending_confirmation" => Ok(SkillReviewStatus::RiskPendingConfirmation),
        _ => Err("invalid Skill review status".to_owned()),
    }
}

fn skill_review_status_name(status: SkillReviewStatus) -> &'static str {
    match status {
        SkillReviewStatus::PendingReview => "pending_review",
        SkillReviewStatus::Approved => "approved",
        SkillReviewStatus::Rejected => "rejected",
        SkillReviewStatus::RiskPendingConfirmation => "risk_pending_confirmation",
    }
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
            let usage = repository
                .usage_stats(prompt.id())
                .map_err(|error| error.to_string())?;
            PromptListItem::from_prompt(prompt, favorite, usage)
        })
        .collect()
}

#[tauri::command]
pub fn record_prompt_use(
    service: State<'_, PromptService>,
    id: String,
) -> Result<PromptUsageItem, String> {
    let id = PromptId::from_uuid(Uuid::parse_str(&id).map_err(|_| "invalid prompt id".to_owned())?);
    PromptUsageItem::from_store(service.record_use(id, OffsetDateTime::now_utc())?)
}

#[tauri::command]
pub fn migrate_legacy_prompt_usage(
    service: State<'_, PromptService>,
    entries: Vec<LegacyPromptUsageInput>,
) -> Result<(), String> {
    let entries = entries
        .into_iter()
        .filter(|entry| entry.use_count > 0)
        .map(|entry| {
            Uuid::parse_str(&entry.id)
                .map(PromptId::from_uuid)
                .map(|id| (id, entry.use_count))
                .map_err(|_| "invalid prompt id".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    service.merge_legacy_usage(entries).map(|_| ())
}

#[tauri::command]
pub fn collect_skill_folder(
    service: State<'_, SkillService>,
    path: String,
) -> Result<SkillListItem, String> {
    service.collect_local_folder(PathBuf::from(path))
}

#[tauri::command]
pub fn collect_git_skill(
    service: State<'_, SkillService>,
    source: GitSkillCollectionInput,
) -> Result<SkillListItem, String> {
    service.collect_git_candidate(source)
}

#[tauri::command]
pub fn list_skills(service: State<'_, SkillService>) -> Result<Vec<SkillListItem>, String> {
    service.list()
}

#[tauri::command]
pub fn get_skill(
    service: State<'_, SkillService>,
    id: String,
) -> Result<Option<SkillDetail>, String> {
    service.get(&id)
}

#[tauri::command]
pub fn review_skill(
    service: State<'_, SkillService>,
    id: String,
    review: SkillReviewInput,
) -> Result<(), String> {
    service.review(&id, review)
}

#[tauri::command]
pub fn set_skill_favorite(
    service: State<'_, SkillService>,
    id: String,
    favorite: bool,
) -> Result<(), String> {
    service.set_favorite(&id, favorite)
}

#[tauri::command]
pub fn install_skill(
    service: State<'_, SkillService>,
    id: String,
    installation: SkillInstallInput,
) -> Result<SkillInstallationItem, String> {
    service.install(&id, installation)
}

#[tauri::command]
pub fn verify_skill_installation(
    service: State<'_, SkillService>,
    id: String,
) -> Result<SkillInstallationVerification, String> {
    service.verify_installation(&id)
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
    sort: Option<SearchSort>,
) -> Result<SearchPage, String> {
    let query = SearchQuery::new(text)
        .with_filters(filters.unwrap_or_default().into_store()?)
        .with_sort(sort.unwrap_or_default())
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
pub async fn generate_ai_draft(
    service: State<'_, PromptService>,
    cancellations: State<'_, AiCancellationRegistry>,
    request: AiGenerationRequestInput,
) -> Result<Prompt, String> {
    let task_id = request.task_id.clone();
    let cancellation = cancellations.register(task_id.clone())?;
    let result = service
        .create_ai_draft(request, OffsetDateTime::now_utc(), cancellation)
        .await;
    cancellations.finish(&task_id);
    result
}

#[tauri::command]
pub async fn optimize_ai_prompt(
    service: State<'_, PromptService>,
    cancellations: State<'_, AiCancellationRegistry>,
    id: PromptId,
    request: AiGenerationRequestInput,
) -> Result<Prompt, String> {
    let task_id = request.task_id.clone();
    let cancellation = cancellations.register(task_id.clone())?;
    let result = service
        .optimize_ai_prompt(id, request, OffsetDateTime::now_utc(), cancellation)
        .await;
    cancellations.finish(&task_id);
    result
}

#[tauri::command]
pub fn cancel_ai_generation(
    cancellations: State<'_, AiCancellationRegistry>,
    task_id: String,
) -> Result<(), String> {
    if cancellations.cancel(&task_id) {
        Ok(())
    } else {
        Err("AI generation task is not active".to_owned())
    }
}

#[tauri::command]
pub async fn test_ai_connection(
    service: State<'_, PromptService>,
    request: AiConnectionRequestInput,
) -> Result<AiConnectionStatus, String> {
    service.test_ai_connection(request).await?;
    Ok(AiConnectionStatus { connected: true })
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
        failed: outcome.failed,
    })
}

#[tauri::command]
pub fn import_folder_to_inbox(
    service: State<'_, PromptService>,
    path: String,
) -> Result<ImportResult, String> {
    let outcome = service.import_folder_to_inbox(PathBuf::from(path), OffsetDateTime::now_utc())?;
    Ok(ImportResult {
        imported: outcome.drafts.len(),
        skipped_duplicates: outcome.skipped_duplicates,
        failed: outcome.failed,
    })
}

#[tauri::command]
pub fn import_url_to_inbox(
    service: State<'_, PromptService>,
    url: String,
) -> Result<ImportResult, String> {
    let outcome = service.import_url_to_inbox(url, OffsetDateTime::now_utc())?;
    Ok(ImportResult {
        imported: outcome.drafts.len(),
        skipped_duplicates: outcome.skipped_duplicates,
        failed: outcome.failed,
    })
}

#[tauri::command]
pub fn recent_import_jobs(
    service: State<'_, PromptService>,
) -> Result<Vec<ImportJobSummary>, String> {
    service
        .recent_import_jobs()?
        .into_iter()
        .map(ImportJobSummary::from_store)
        .collect()
}

#[tauri::command]
pub fn get_mcp_setup(backups: State<'_, BackupService>) -> McpSetupInfo {
    backups.mcp_setup()
}

#[tauri::command]
pub fn prune_local_backups(
    backups: State<'_, BackupService>,
    retain: usize,
) -> Result<usize, String> {
    backups.prune_backups(retain)
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
pub fn permanently_delete_prompt(
    prompts: State<'_, PromptService>,
    backups: State<'_, BackupService>,
    id: PromptId,
) -> Result<BackupInfo, String> {
    prompts.permanently_delete(id, &backups)
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
pub struct DiagnosticsStatus {
    database_available: bool,
    search_index_consistent: bool,
    mcp_database_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedactedDiagnosticEvent {
    occurred_at: String,
    event: String,
    recommendation: String,
}

fn redacted_diagnostic_events(
    database_available: bool,
    search_index_consistent: bool,
    observed_at: OffsetDateTime,
) -> Vec<RedactedDiagnosticEvent> {
    let occurred_at = format_timestamp(observed_at).unwrap_or_default();
    let mut events = Vec::new();
    if !database_available {
        events.push(RedactedDiagnosticEvent {
            occurred_at: occurred_at.clone(),
            event: "database_unavailable".to_owned(),
            recommendation: "检查数据目录权限后重试。".to_owned(),
        });
    }
    if !search_index_consistent {
        events.push(RedactedDiagnosticEvent {
            occurred_at,
            event: "search_index_inconsistent".to_owned(),
            recommendation: "在诊断信息中重建搜索索引后重试。".to_owned(),
        });
    }
    events
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiCredentialStatus {
    configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConnectionStatus {
    connected: bool,
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
    prompt_count: u64,
}

impl BackupRestorePreview {
    const fn from_store(preview: prompt_store::RestorePreview) -> Self {
        Self {
            target_exists: preview.target_exists(),
            backup_schema_version: preview.backup_schema_version(),
            backup_byte_len: preview.backup_byte_len(),
            prompt_count: preview.prompt_count(),
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
pub fn get_bootstrap_status(runtime: State<'_, BootstrapRuntime>) -> BootstrapStatus {
    runtime.status()
}

#[tauri::command]
pub fn retry_database_bootstrap(
    app: tauri::AppHandle,
    runtime: State<'_, BootstrapRuntime>,
) -> Result<BootstrapStatus, String> {
    if runtime.status().state == "ready" {
        return Ok(runtime.status());
    }
    let services = bootstrap::prepare_services(&runtime).map_err(|failure| {
        runtime.mark_recovery(
            failure.code.clone(),
            failure.safe_message.clone(),
            failure.backup_name.clone(),
        );
        failure.safe_message
    })?;
    bootstrap::attach_services(&app, services);
    runtime.mark_ready();
    Ok(runtime.status())
}

#[tauri::command]
pub fn export_bootstrap_diagnostics(
    runtime: State<'_, BootstrapRuntime>,
) -> Result<String, String> {
    serde_json::to_string_pretty(&runtime.status()).map_err(|_| "无法导出诊断摘要".to_owned())
}

#[tauri::command]
pub fn get_diagnostics_status(
    prompts: State<'_, PromptService>,
    backups: State<'_, BackupService>,
) -> DiagnosticsStatus {
    let search_index_consistent = prompts
        .repository
        .lock()
        .ok()
        .and_then(|repository| repository.search_index_is_consistent().ok())
        .unwrap_or(false);
    let database_available = prompt_store::Database::open(&backups.database_path).is_ok();
    DiagnosticsStatus {
        database_available,
        search_index_consistent,
        mcp_database_available: database_available,
    }
}

#[tauri::command]
pub fn get_redacted_diagnostic_events(
    prompts: State<'_, PromptService>,
    backups: State<'_, BackupService>,
) -> Vec<RedactedDiagnosticEvent> {
    let database_available = prompt_store::Database::open(&backups.database_path).is_ok();
    let search_index_consistent = prompts
        .repository
        .lock()
        .ok()
        .and_then(|repository| repository.search_index_is_consistent().ok())
        .unwrap_or(false);
    redacted_diagnostic_events(
        database_available,
        search_index_consistent,
        OffsetDateTime::now_utc(),
    )
}

#[tauri::command]
pub fn rebuild_search_index(prompts: State<'_, PromptService>) -> Result<(), String> {
    prompts
        .repository
        .lock()
        .map_err(|_| "prompt repository is unavailable".to_owned())?
        .rebuild_search_index()
        .map_err(|error| error.to_string())
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
pub fn create_manual_backup(
    service: State<'_, BackupService>,
    directory: Option<String>,
) -> Result<BackupInfo, String> {
    service.create_manual_backup(directory.map(PathBuf::from))
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
mod ai_cancellation_tests {
    use super::*;

    #[test]
    fn cancelling_a_registered_generation_marks_it_cancelled_and_removes_it() {
        let registry = AiCancellationRegistry::default();
        let mut cancellation = registry.register("generation-1".to_owned()).unwrap();

        assert!(registry.cancel("generation-1"));
        assert!(cancellation.has_changed().unwrap());
        assert!(*cancellation.borrow_and_update());

        registry.finish("generation-1");
        assert!(!registry.cancel("generation-1"));
    }
}

#[cfg(test)]
mod diagnostics_tests {
    use super::*;

    #[test]
    fn redacted_events_explain_an_inconsistent_search_index_without_sensitive_data() {
        let events = redacted_diagnostic_events(false, false, OffsetDateTime::UNIX_EPOCH);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event, "database_unavailable");
        assert_eq!(events[1].event, "search_index_inconsistent");
        assert!(!events.iter().any(|event| event.event.contains("secret")));
    }
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

        let backup = service.create_manual_backup(None).unwrap();
        let preview = service.preview_restore(PathBuf::from(backup.path)).unwrap();

        assert!(preview.target_exists);
        assert_eq!(preview.backup_schema_version, LATEST_SCHEMA_VERSION);
        assert!(preview.backup_byte_len > 0);
    }

    #[test]
    fn url_import_failures_are_recorded_as_import_jobs() {
        let directory = tempfile::tempdir().unwrap();
        let database =
            prompt_store::Database::open(directory.path().join("prompt-hub.db")).unwrap();
        let service = PromptService::new(database.into_repository());
        let created_at = OffsetDateTime::now_utc();

        assert!(
            service
                .import_url_to_inbox("file:///not-a-web-page".to_owned(), created_at)
                .is_err()
        );

        let jobs = service.recent_import_jobs().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].source_kind(), "web_url");
        assert_eq!(jobs[0].status(), "failed");
        assert_eq!(jobs[0].source_path(), Some("file:///not-a-web-page"));
    }

    #[test]
    fn permanent_deletion_creates_a_verified_safety_backup_before_removing_the_prompt() {
        let directory = tempfile::tempdir().unwrap();
        let database_path = directory.path().join("prompt-hub.db");
        let database = prompt_store::Database::open(&database_path).unwrap();
        let mut repository = database.into_repository();
        let created_at = OffsetDateTime::now_utc();
        let content = PromptContent::new(
            "待永久清除",
            "正文",
            None,
            Some("测试".to_owned()),
            Vec::new(),
        )
        .unwrap();
        let source = PromptSource::new(SourceKind::Manual, "测试", None, created_at).unwrap();
        let mut prompt = Prompt::new_inbox(content, source, Actor::User, created_at);
        repository.save(&prompt, AuditAction::Created).unwrap();
        prompt.soft_delete(Actor::User, created_at).unwrap();
        repository.save(&prompt, AuditAction::Deleted).unwrap();
        let id = prompt.id();
        let service = PromptService::new(repository);
        let backups = BackupService::new(database_path);

        let backup = service.permanently_delete(id, &backups).unwrap();

        assert!(PathBuf::from(backup.path).exists());
        assert!(
            service
                .repository
                .lock()
                .unwrap()
                .get(id)
                .unwrap()
                .is_none()
        );
    }
}
