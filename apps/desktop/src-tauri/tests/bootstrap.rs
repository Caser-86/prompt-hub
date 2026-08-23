use std::path::PathBuf;

use prompt_hub_desktop_lib::bootstrap::{BootstrapRuntime, BootstrapStatus};

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
