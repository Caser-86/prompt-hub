use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::commands::{
    AiCancellationRegistry, AiSettingsService, BackupService, PromptService, SkillService,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapStatus {
    pub state: String,
    pub code: Option<String>,
    pub safe_message: Option<String>,
    pub backup_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BootstrapFailure {
    pub code: String,
    pub safe_message: String,
    pub backup_name: Option<String>,
}

pub struct BootstrapRuntime {
    database_path: PathBuf,
    data_directory: PathBuf,
    status: Mutex<BootstrapStatus>,
}

impl BootstrapRuntime {
    #[must_use]
    pub fn new(data_directory: PathBuf) -> Self {
        Self {
            database_path: data_directory.join("prompt-hub.db"),
            data_directory,
            status: Mutex::new(ready_status()),
        }
    }

    #[must_use]
    pub fn for_test(database_path: PathBuf) -> Self {
        let data_directory = database_path
            .parent()
            .map_or_else(PathBuf::new, Path::to_path_buf);
        Self {
            database_path,
            data_directory,
            status: Mutex::new(ready_status()),
        }
    }

    #[must_use]
    pub fn status(&self) -> BootstrapStatus {
        self.status.lock().map_or_else(
            |_| BootstrapStatus {
                state: "recovery".to_owned(),
                code: Some("bootstrap_state_unavailable".to_owned()),
                safe_message: Some("应用启动状态不可用，请重新启动后重试。".to_owned()),
                backup_name: None,
            },
            |status| status.clone(),
        )
    }

    pub fn mark_ready(&self) {
        if let Ok(mut status) = self.status.lock() {
            *status = ready_status();
        }
    }

    pub fn mark_recovery(
        &self,
        code: impl Into<String>,
        safe_message: impl Into<String>,
        backup_name: Option<String>,
    ) {
        if let Ok(mut status) = self.status.lock() {
            *status = BootstrapStatus {
                state: "recovery".to_owned(),
                code: Some(code.into()),
                safe_message: Some(safe_message.into()),
                backup_name,
            };
        }
    }

    #[must_use]
    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    #[must_use]
    pub fn data_directory(&self) -> &Path {
        &self.data_directory
    }
}

pub struct BootstrapServices {
    pub prompt: PromptService,
    pub skill: SkillService,
    pub cancellation: AiCancellationRegistry,
    pub backups: BackupService,
    pub ai: AiSettingsService,
}

pub fn prepare_services(runtime: &BootstrapRuntime) -> Result<BootstrapServices, BootstrapFailure> {
    let data_directory = runtime.data_directory().to_path_buf();
    let database_path = runtime.database_path().to_path_buf();
    let database = prompt_store::Database::open(&database_path).map_err(|_| migration_failure())?;
    let skill_database =
        prompt_store::Database::open(&database_path).map_err(|_| migration_failure())?;
    let credentials =
        prompt_ai::SystemCredentialAdapter::new("Prompt Hub", "default").map_err(|_| {
            BootstrapFailure {
                code: "credential_store_unavailable".to_owned(),
                safe_message: "系统凭据存储不可用，应用暂时进入恢复状态。".to_owned(),
                backup_name: None,
            }
        })?;

    Ok(BootstrapServices {
        prompt: PromptService::new(database.into_repository()),
        skill: SkillService::with_storage_roots(
            skill_database.into_skill_repository(),
            data_directory.join("skill-backups"),
            data_directory.join("skill-snapshots"),
        ),
        cancellation: AiCancellationRegistry::default(),
        backups: BackupService::new(database_path),
        ai: AiSettingsService::new(credentials),
    })
}

pub fn attach_services(app: &AppHandle, services: BootstrapServices) {
    app.manage(services.prompt);
    app.manage(services.skill);
    app.manage(services.cancellation);
    app.manage(services.backups);
    app.manage(services.ai);
}

#[must_use]
pub fn migration_failure() -> BootstrapFailure {
    BootstrapFailure {
        code: "migration_failed".to_owned(),
        safe_message: "本地数据升级失败，原数据未被替换。请重试或导出诊断信息。".to_owned(),
        backup_name: None,
    }
}

fn ready_status() -> BootstrapStatus {
    BootstrapStatus {
        state: "ready".to_owned(),
        code: None,
        safe_message: None,
        backup_name: None,
    }
}
