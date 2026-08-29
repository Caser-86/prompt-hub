use std::fs;

use prompt_skill::{
    InstallMode, InstallRequest, SkillInstallError, install_skill, rollback_installation,
    scan_skill,
};
use tempfile::tempdir;

fn source_with_body(body: &str) -> tempfile::TempDir {
    let source = tempdir().unwrap();
    fs::write(source.path().join("SKILL.md"), body).unwrap();
    fs::create_dir(source.path().join("scripts")).unwrap();
    fs::write(source.path().join("scripts/run.cmd"), "exit 77").unwrap();
    source
}

#[test]
fn installation_copies_reviewed_files_without_running_scripts_and_refuses_conflicts() {
    let source = source_with_body("# Safe copy\n");
    let target = tempdir().unwrap();
    let backups = tempdir().unwrap();
    let hash = scan_skill(source.path()).unwrap().content_hash().to_owned();
    let request = InstallRequest {
        source: source.path(),
        target_root: target.path(),
        backup_root: backups.path(),
        destination_name: "safe-copy",
        expected_content_hash: &hash,
        mode: InstallMode::FailIfExists,
    };
    let receipt = install_skill(request.clone()).unwrap();
    assert!(receipt.install_path().join("SKILL.md").is_file());
    assert!(receipt.install_path().join("scripts/run.cmd").is_file());
    assert!(matches!(
        install_skill(request),
        Err(SkillInstallError::DestinationExists)
    ));
}

#[test]
fn installation_detects_source_drift_before_copying() {
    let source = source_with_body("# Before\n");
    let target = tempdir().unwrap();
    let backups = tempdir().unwrap();
    let hash = scan_skill(source.path()).unwrap().content_hash().to_owned();
    fs::write(source.path().join("SKILL.md"), "# Changed\n").unwrap();
    let error = install_skill(InstallRequest {
        source: source.path(),
        target_root: target.path(),
        backup_root: backups.path(),
        destination_name: "changed",
        expected_content_hash: &hash,
        mode: InstallMode::FailIfExists,
    })
    .unwrap_err();
    assert!(matches!(error, SkillInstallError::SourceChanged));
    assert!(!target.path().join("changed").exists());
}

#[test]
fn replacement_moves_previous_install_to_backup_first() {
    let source = source_with_body("# New\n");
    let target = tempdir().unwrap();
    let backups = tempdir().unwrap();
    let existing = target.path().join("same");
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("SKILL.md"), "# Old\n").unwrap();
    let hash = scan_skill(source.path()).unwrap().content_hash().to_owned();
    let receipt = install_skill(InstallRequest {
        source: source.path(),
        target_root: target.path(),
        backup_root: backups.path(),
        destination_name: "same",
        expected_content_hash: &hash,
        mode: InstallMode::ReplaceAfterBackup,
    })
    .unwrap();
    assert_eq!(
        fs::read_to_string(target.path().join("same/SKILL.md")).unwrap(),
        "# New\n"
    );
    assert_eq!(
        fs::read_to_string(receipt.backup_path().unwrap().join("SKILL.md")).unwrap(),
        "# Old\n"
    );
}

#[test]
fn rollback_removes_the_new_install_and_restores_the_previous_backup() {
    let source = source_with_body("# New\n");
    let target = tempdir().unwrap();
    let backups = tempdir().unwrap();
    let existing = target.path().join("same");
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("SKILL.md"), "# Old\n").unwrap();
    let hash = scan_skill(source.path()).unwrap().content_hash().to_owned();
    let receipt = install_skill(InstallRequest {
        source: source.path(),
        target_root: target.path(),
        backup_root: backups.path(),
        destination_name: "same",
        expected_content_hash: &hash,
        mode: InstallMode::ReplaceAfterBackup,
    })
    .unwrap();

    rollback_installation(&receipt).unwrap();

    assert_eq!(
        fs::read_to_string(target.path().join("same/SKILL.md")).unwrap(),
        "# Old\n"
    );
    assert!(!receipt.backup_path().unwrap().exists());
}

#[test]
fn rollback_refuses_to_delete_an_install_that_changed_after_copying() {
    let source = source_with_body("# New\n");
    let target = tempdir().unwrap();
    let backups = tempdir().unwrap();
    let hash = scan_skill(source.path()).unwrap().content_hash().to_owned();
    let receipt = install_skill(InstallRequest {
        source: source.path(),
        target_root: target.path(),
        backup_root: backups.path(),
        destination_name: "new",
        expected_content_hash: &hash,
        mode: InstallMode::FailIfExists,
    })
    .unwrap();
    fs::write(receipt.install_path().join("SKILL.md"), "# User change\n").unwrap();

    assert!(matches!(
        rollback_installation(&receipt),
        Err(SkillInstallError::RollbackUnsafe)
    ));
    assert!(receipt.install_path().exists());
}
