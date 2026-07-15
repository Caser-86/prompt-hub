# Permanent Prompt Deletion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Permanently remove only soft-deleted prompt assets after a verified pre-delete backup and explicit title confirmation.

**Architecture:** The repository owns the transactional authorization and relational cleanup. The desktop service creates the `PermanentDelete` backup before invoking it; Tauri exposes one ID-only command. The React lifecycle panel renders the maintenance confirmation only for deleted prompts and refreshes the library after a successful removal.

**Tech Stack:** Rust, rusqlite, Tauri 2, React, TypeScript, Vitest, Cargo test.

## Global Constraints

- `docs/prompt-hub-product-spec.md` is the product baseline.
- AI and MCP writes may create inbox drafts only and cannot invoke permanent deletion.
- Every schema change needs a forward migration and migration test.
- Never log credentials, authorization headers, or prompt bodies.
- Use test-driven development and fresh verification output.

---

### Task 1: Transactional repository deletion

**Files:**
- Modify: `crates/prompt-store/src/repository.rs`
- Modify: `crates/prompt-store/tests/repository.rs`

**Interfaces:**
- Produces: `PromptRepository::permanently_delete(PromptId) -> Result<(), StoreError>`.
- Rejects a prompt not currently stored with `status = 'deleted'`.
- Preserves `import_job_items` rows by relying on existing `ON DELETE SET NULL` for `prompt_id`.

- [ ] **Step 1: Write failing repository tests**

```rust
#[test]
fn permanently_deletes_only_soft_deleted_prompt_and_related_records() {
    let mut repository = test_repository();
    let prompt = saved_deleted_prompt(&mut repository);
    repository.permanently_delete(prompt.id()).unwrap();
    assert!(repository.get(prompt.id()).unwrap().is_none());
    assert_eq!(search_row_count(&repository, prompt.id()), 0);
    assert_eq!(import_job_item_prompt_id(&repository, prompt.id()), None);
}

#[test]
fn permanent_delete_rejects_non_deleted_prompts_without_mutating_them() {
    let mut repository = test_repository();
    let prompt = saved_inbox_prompt(&mut repository);
    assert!(repository.permanently_delete(prompt.id()).is_err());
    assert!(repository.get(prompt.id()).unwrap().is_some());
}
```

- [ ] **Step 2: Run the focused test and observe failure**

Run: `cargo test -p prompt-store --test repository permanently_delete`

Expected: fail because `permanently_delete` does not exist.

- [ ] **Step 3: Implement the guarded transaction**

```rust
pub fn permanently_delete(&mut self, id: PromptId) -> Result<(), StoreError> {
    let prompt_id = id.value().to_string();
    let transaction = self.connection.transaction()?;
    let status: Option<String> = transaction.query_row(
        "SELECT status FROM prompts WHERE id = ?1", [&prompt_id], |row| row.get(0),
    ).optional()?;
    if status.as_deref() != Some("deleted") {
        return Err(StoreError::Domain("only soft-deleted prompts can be permanently removed".to_owned()));
    }
    transaction.execute("DELETE FROM search_content WHERE prompt_id = ?1", [&prompt_id])?;
    transaction.execute("DELETE FROM prompt_versions WHERE prompt_id = ?1", [&prompt_id])?;
    transaction.execute("DELETE FROM prompt_sources WHERE prompt_id = ?1", [&prompt_id])?;
    transaction.execute("DELETE FROM prompt_compatibilities WHERE prompt_id = ?1", [&prompt_id])?;
    transaction.execute("DELETE FROM prompt_validations WHERE prompt_id = ?1", [&prompt_id])?;
    transaction.execute("DELETE FROM prompt_favorites WHERE prompt_id = ?1", [&prompt_id])?;
    transaction.execute("DELETE FROM audit_events WHERE prompt_id = ?1", [&prompt_id])?;
    transaction.execute("DELETE FROM prompts WHERE id = ?1", [&prompt_id])?;
    transaction.commit()?;
    Ok(())
}
```

- [ ] **Step 4: Run focused tests**

Run: `cargo test -p prompt-store --test repository permanently_delete`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add crates/prompt-store/src/repository.rs crates/prompt-store/tests/repository.rs
git commit -m "feat(store): permanently remove soft-deleted prompts"
```

### Task 2: Backup-before-delete desktop service and command

**Files:**
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/tests/commands.rs`
- Modify: `packages/contracts/src/index.ts`
- Modify: `packages/contracts/src/index.test.ts`

**Interfaces:**
- Produces `PromptService::permanently_delete(id, &BackupService) -> Result<BackupInfo, String>`.
- Exposes Tauri command `permanently_delete_prompt` with `{ id }`, returning validated `BackupInfo`.
- Always calls `create_backup(database_path, BackupDestination::PermanentDelete)` before repository deletion.

- [ ] **Step 1: Write failing command/service tests**

```rust
#[test]
fn permanent_delete_creates_verified_safety_backup_before_removing_prompt() {
    let (service, backups, prompt) = deleted_prompt_service_fixture();
    let backup = service.permanently_delete(prompt.id(), &backups).unwrap();
    assert!(Path::new(&backup.path).exists());
    assert!(service.get(prompt.id()).unwrap().is_none());
}
```

```ts
it("invokes the permanent deletion command with only the prompt id", async () => {
  await desktopCommands.permanentlyDeletePrompt("prompt-1");
  expect(invoke).toHaveBeenCalledWith("permanently_delete_prompt", { id: "prompt-1" });
});
```

- [ ] **Step 2: Run focused tests and observe failure**

Run: `cargo test -p prompt-hub-desktop permanent_delete; pnpm --filter @prompt-hub/contracts test -- --runInBand`

Expected: command/client methods are missing.

- [ ] **Step 3: Implement the service, Tauri handler and contract client**

```rust
pub fn permanently_delete(&self, id: PromptId, backups: &BackupService) -> Result<BackupInfo, String> {
    let backup = BackupInfo::from_store(create_backup(
        &backups.database_path, BackupDestination::PermanentDelete,
    ).map_err(|error| error.to_string())?)?;
    self.repository.lock().map_err(|_| "prompt repository is unavailable".to_owned())?
        .permanently_delete(id).map_err(|error| error.to_string())?;
    Ok(backup)
}
```

Register `commands::permanently_delete_prompt` in `generate_handler!` and add `permanentlyDeletePrompt(id)` to `desktopCommands`.

- [ ] **Step 4: Run focused tests**

Run: `cargo test -p prompt-hub-desktop permanent_delete; pnpm --filter @prompt-hub/contracts test`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add apps/desktop/src-tauri packages/contracts
git commit -m "feat(desktop): back up before permanent prompt deletion"
```

### Task 3: Explicit maintenance UI

**Files:**
- Modify: `apps/desktop/src/features/library/PromptLifecycleActions.tsx`
- Modify: `apps/desktop/src/features/library/PromptLifecycleActions.test.tsx`
- Modify: `apps/desktop/src/app/AppShell.tsx`
- Modify: `apps/desktop/src/App.test.tsx`

**Interfaces:**
- Extend props with `promptTitle: string`, `permanentlyDelete: () => Promise<{ path: string }>` and `onPermanentlyDeleted: () => void`.
- Render permanent cleanup only while status is `deleted`.

- [ ] **Step 1: Write failing UI tests**

```tsx
it("requires the complete title before permanent cleanup", async () => {
  render(<PromptLifecycleActions initialStatus="deleted" promptTitle="代码审查" {...handlers} />);
  fireEvent.click(screen.getByRole("button", { name: "永久清除提示词" }));
  expect(screen.getByRole("button", { name: "确认永久清除" })).toBeDisabled();
  fireEvent.change(screen.getByLabelText("输入提示词标题以确认"), { target: { value: "代码审查" } });
  fireEvent.click(screen.getByRole("button", { name: "确认永久清除" }));
  await waitFor(() => expect(handlers.permanentlyDelete).toHaveBeenCalledOnce());
});
```

- [ ] **Step 2: Run focused test and observe failure**

Run: `pnpm --filter @prompt-hub/desktop test -- PromptLifecycleActions.test.tsx`

Expected: missing maintenance action and confirmation input.

- [ ] **Step 3: Implement confirmation and refresh wiring**

Render an `alertdialog` that explains the action cannot be undone and shows no prompt body. Require `confirmationTitle === promptTitle`; after successful promise resolution, show the returned backup path, call `onPermanentlyDeleted`, then close the prompt details view from `AppShell` and increment `libraryKey`.

- [ ] **Step 4: Run focused UI tests**

Run: `pnpm --filter @prompt-hub/desktop test -- PromptLifecycleActions.test.tsx App.test.tsx`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add apps/desktop/src/features/library apps/desktop/src/app apps/desktop/src/App.test.tsx
git commit -m "feat(library): add confirmed permanent cleanup"
```

### Task 4: Full verification and user documentation

**Files:**
- Modify: `docs/user-guide.md`
- Modify: `docs/privacy.md`

- [ ] **Step 1: Document verified maintenance behavior**

Add that only soft-deleted items can be permanently cleared, a title confirmation and verified local safety backup are required, and the operation removes the local prompt record and version history. Do not claim cloud recovery.

- [ ] **Step 2: Run the Session B verification suite**

Run:

```powershell
pnpm lint
pnpm typecheck
pnpm test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm test:e2e
pnpm --filter @prompt-hub/desktop build
```

Expected: all commands exit 0.

- [ ] **Step 3: Commit**

```powershell
git add docs/user-guide.md docs/privacy.md
git commit -m "docs: explain permanent cleanup safeguards"
```

## Self-Review

- Spec coverage: repository authorization and relational cleanup, mandatory backup, command boundary, UI confirmation, docs and all required checks are covered.
- No placeholders: all behavior, interfaces and commands are concrete.
- Type consistency: the service returns `BackupInfo`, the Tauri client validates the same existing contract shape, and the UI consumes its `path` field.
