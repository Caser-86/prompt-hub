pub mod bootstrap;
pub mod commands;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let runtime = match app.path().app_data_dir() {
                Ok(data_directory) => bootstrap::BootstrapRuntime::new(data_directory),
                Err(_) => bootstrap::BootstrapRuntime::unavailable(),
            };
            app.manage(runtime);
            let runtime = app.state::<bootstrap::BootstrapRuntime>();
            match bootstrap::prepare_services(&runtime) {
                Ok(services) => bootstrap::attach_services(app.handle(), services),
                Err(failure) => {
                    runtime.mark_recovery(failure.code, failure.safe_message, failure.backup_name)
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_bootstrap_status,
            commands::retry_database_bootstrap,
            commands::export_bootstrap_diagnostics,
            commands::get_application_status,
            commands::get_diagnostics_status,
            commands::get_redacted_diagnostic_events,
            commands::rebuild_search_index,
            commands::get_ai_credential_status,
            commands::save_ai_credential,
            commands::create_manual_backup,
            commands::preview_backup_restore,
            commands::restore_backup,
            commands::list_prompts,
            commands::collect_skill_folder,
            commands::collect_git_skill,
            commands::list_skills,
            commands::get_skill,
            commands::review_skill,
            commands::set_skill_favorite,
            commands::install_skill,
            commands::verify_skill_installation,
            commands::prompt_history,
            commands::restore_prompt_version,
            commands::search_prompts,
            commands::create_manual_prompt_draft,
            commands::generate_ai_draft,
            commands::optimize_ai_prompt,
            commands::cancel_ai_generation,
            commands::test_ai_connection,
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
        .unwrap_or_else(|error| eprintln!("Prompt Hub desktop event loop stopped: {error}"));
}
