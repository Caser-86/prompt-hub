use serde::Serialize;

use prompt_store::LATEST_SCHEMA_VERSION;

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
