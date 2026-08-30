use std::fs;

use prompt_skill::{scan_skill, snapshot_local_skill};
use tempfile::tempdir;

#[test]
fn local_snapshot_remains_installable_after_source_is_removed() {
    let source = tempdir().unwrap();
    let snapshot_parent = tempdir().unwrap();
    fs::write(source.path().join("SKILL.md"), "# Durable skill\n").unwrap();
    fs::write(source.path().join("notes.txt"), "reviewed content").unwrap();
    let before = scan_skill(source.path()).unwrap();
    let snapshot = snapshot_parent.path().join("snapshot");

    let copied = snapshot_local_skill(source.path(), &snapshot).unwrap();
    fs::remove_dir_all(source.path()).unwrap();

    assert_eq!(copied.content_hash(), before.content_hash());
    assert_eq!(
        scan_skill(&snapshot).unwrap().content_hash(),
        before.content_hash()
    );
}
