pub mod commands;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            let database = prompt_store::Database::open(data_directory.join("prompt-hub.db"))?;
            app.manage(commands::PromptService::new(database.into_repository()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_application_status,
            commands::list_prompts,
            commands::create_manual_prompt_draft,
            commands::publish_prompt,
            commands::revise_prompt,
            commands::archive_prompt,
            commands::soft_delete_prompt,
            commands::recover_prompt
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Prompt Hub desktop application");
}
