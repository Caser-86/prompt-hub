use std::path::PathBuf;

use prompt_hub_desktop_lib::bootstrap::{BootstrapRuntime, BootstrapStatus, prepare_services};

#[test]
fn bootstrap_runtime_exposes_safe_recovery_status() {
    let runtime = BootstrapRuntime::for_test(PathBuf::from("C:/private/prompt-hub.db"));
    runtime.mark_recovery(
        "migration_failed",
        "本地数据升级失败，原数据未被替换。",
        None,
    );

    let status: BootstrapStatus = runtime.status();
    assert_eq!(status.state, "recovery");
    assert_eq!(status.code.as_deref(), Some("migration_failed"));
    assert!(
        !status
            .safe_message
            .as_deref()
            .is_some_and(|message| message.contains("C:/private"))
    );
    assert_eq!(status.backup_name, None);
}

#[test]
fn bootstrap_runtime_can_return_to_ready_without_restarting() {
    let runtime = BootstrapRuntime::for_test(PathBuf::from("C:/private/prompt-hub.db"));
    runtime.mark_recovery(
        "migration_failed",
        "本地数据升级失败，原数据未被替换。",
        None,
    );
    runtime.mark_ready();

    assert_eq!(runtime.status().state, "ready");
    assert_eq!(runtime.status().code, None);
}

#[test]
fn unavailable_app_data_path_stays_in_recovery_and_cannot_prepare_services() {
    let runtime = BootstrapRuntime::unavailable();

    assert!(!runtime.data_directory_available());
    assert_eq!(runtime.status().state, "recovery");
    assert_eq!(
        runtime.status().code.as_deref(),
        Some("data_directory_unavailable")
    );
    assert!(runtime.database_path().as_os_str().is_empty());

    let failure = match prepare_services(&runtime) {
        Ok(_) => panic!("service preparation must not use a relative fallback database"),
        Err(failure) => failure,
    };
    assert_eq!(failure.code, "data_directory_unavailable");
    assert!(!failure.safe_message.contains(':'));
}

#[test]
fn an_empty_data_directory_is_treated_as_unavailable() {
    let runtime = BootstrapRuntime::new(PathBuf::new());

    assert!(!runtime.data_directory_available());
    assert_eq!(
        runtime.status().code.as_deref(),
        Some("data_directory_unavailable")
    );
}
