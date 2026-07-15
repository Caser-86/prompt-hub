pub mod commands;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            let database_path = data_directory.join("prompt-hub.db");
            let database = prompt_store::Database::open(&database_path)?;
            app.manage(commands::PromptService::new(database.into_repository()));
            app.manage(commands::BackupService::new(database_path));
            let credentials = prompt_ai::SystemCredentialAdapter::new("Prompt Hub", "default")?;
            app.manage(commands::AiSettingsService::new(credentials));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_application_status,
            commands::get_ai_credential_status,
            commands::save_ai_credential,
            commands::create_manual_backup,
            commands::preview_backup_restore,
            commands::restore_backup,
            commands::list_prompts,
            commands::prompt_history,
            commands::restore_prompt_version,
            commands::search_prompts,
            commands::create_manual_prompt_draft,
            commands::generate_ai_draft,
            commands::import_file_to_inbox,
            commands::import_folder_to_inbox,
            commands::import_url_to_inbox,
            commands::recent_import_jobs,
            commands::get_mcp_setup,
            commands::prune_local_backups,
            commands::publish_prompt,
            commands::revise_prompt,
            commands::archive_prompt,
            commands::batch_archive_prompts,
            commands::soft_delete_prompt,
            commands::permanently_delete_prompt,
            commands::recover_prompt,
            commands::set_prompt_favorite,
            commands::record_prompt_compatibility,
            commands::record_prompt_validation
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Prompt Hub desktop application");
}
