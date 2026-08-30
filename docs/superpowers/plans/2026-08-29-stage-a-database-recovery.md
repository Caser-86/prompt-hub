# Stage A Database and Recovery Safety Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Prompt Hub fail closed for unsupported or incomplete databases and enter a safe recovery state when the Windows application-data directory is unavailable.

**Architecture:** `prompt-store` validates migration lineage, ledger completeness, and the final schema shape before accepting a connection. The Tauri bootstrap represents path resolution failure explicitly and refuses to prepare services without a safe absolute data directory.

**Tech Stack:** Rust 2024, rusqlite 0.40, Tauri 2.11, tempfile, pnpm/Vitest/Playwright.

**Spec:** `docs/superpowers/specs/2026-08-29-stage-a-database-recovery-design.md`

## Global Constraints

- `docs/prompt-hub-product-spec.md` remains the product-requirement baseline.
- Keep SQLite schema version at 7; this phase adds validation, not a data migration.
- Use failing tests before production changes.
- Never log prompt bodies, credentials, authorization headers, or private absolute paths.
- External and AI writes remain inbox-only.

---

### Task 1: Fail closed on invalid migration lineage

**Files:**
- Modify: `crates/prompt-store/tests/migrations.rs`
- Modify: `crates/prompt-store/src/migration.rs`

**Interfaces:**
- Consumes: `Database::open(path) -> Result<Database, StoreError>`
- Produces: strict validation for `LATEST_SCHEMA_VERSION`, canonical ledger IDs, and the known `legacy/0.1.2-prompt-usage` record.

- [x] **Step 1: Add failing migration tests**

Add tests that prove:

```rust
assert!(Database::open(&ledger_only_path).is_err());
assert!(Database::open(&missing_ledger_entry_path).is_err());
assert!(Database::open(&future_version_path).is_err());
drop(Database::open(&legacy_v5_path).unwrap());
assert!(Database::open(&legacy_v5_path).is_ok());
```

- [x] **Step 2: Run the focused suite and observe the intended failures**

Run: `cargo test -p prompt-store --test migrations -- --nocapture`

Expected: new invalid-lineage tests fail because the current implementation accepts incomplete/future schemas; the legacy reopen test fails because the legacy ledger ID is treated as unknown.

- [x] **Step 3: Implement minimal lineage validation**

Introduce explicit constants for the optional legacy ledger record, reject `user_version > LATEST_SCHEMA_VERSION`, validate known checksums, and require every canonical ledger ID when a committed latest-version ledger already exists.

- [x] **Step 4: Run the focused suite**

Run: `cargo test -p prompt-store --test migrations -- --nocapture`

Expected: all migration tests pass.

- [x] **Step 5: Commit**

```powershell
git add crates/prompt-store/src/migration.rs crates/prompt-store/tests/migrations.rs
git commit -m "fix(store): fail closed on invalid migration lineage"
```

### Task 2: Validate final schema shape

**Files:**
- Modify: `crates/prompt-store/tests/migrations.rs`
- Modify: `crates/prompt-store/src/migration.rs`

**Interfaces:**
- Consumes: the migrated SQLite transaction before commit.
- Produces: `validate_latest_schema(connection) -> Result<(), StoreError>`.

- [x] **Step 1: Add failing corruption tests**

Create a valid database, remove one required table in one test and recreate `skills` without `snapshot_path` in another, then assert reopening returns `StoreError::UnsupportedSchema` without recreating missing objects.

- [x] **Step 2: Run the focused tests and observe failure**

Run: `cargo test -p prompt-store --test migrations -- --nocapture`

Expected: both new tests fail because checksum validation alone currently accepts the database.

- [x] **Step 3: Implement required table and column checks**

Validate all version-7 application tables and critical columns using `sqlite_master` and `pragma_table_info`. Run the validator inside the migration transaction after applying/backfilling migrations and before commit.

- [x] **Step 4: Run migration and backup suites**

Run: `cargo test -p prompt-store --test migrations --test backup`

Expected: all tests pass and corrupt databases remain unchanged.

- [x] **Step 5: Commit**

```powershell
git add crates/prompt-store/src/migration.rs crates/prompt-store/tests/migrations.rs
git commit -m "fix(store): verify schema shape before opening"
```

### Task 3: Fail safely when AppData resolution is unavailable

**Files:**
- Modify: `apps/desktop/src-tauri/tests/bootstrap.rs`
- Modify: `apps/desktop/src-tauri/src/bootstrap.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `BootstrapRuntime::unavailable() -> BootstrapRuntime` and `BootstrapRuntime::data_directory_available() -> bool`.
- Consumes: `prepare_services(&BootstrapRuntime)` and existing recovery commands.

- [x] **Step 1: Add failing bootstrap tests**

Add tests asserting:

```rust
let runtime = BootstrapRuntime::unavailable();
assert_eq!(runtime.status().code.as_deref(), Some("data_directory_unavailable"));
assert!(!runtime.data_directory_available());
assert_eq!(prepare_services(&runtime).unwrap_err().code, "data_directory_unavailable");
```

- [x] **Step 2: Run the bootstrap suite and observe compilation/test failure**

Run: `cargo test -p prompt-hub-desktop --test bootstrap -- --nocapture`

Expected: failure because the unavailable constructor and guard do not exist.

- [x] **Step 3: Implement the explicit unavailable state**

Add an availability flag, a safe recovery status, a guard at the start of `prepare_services`, and replace `app_data_dir().unwrap_or_default()` with an explicit `match` that creates either a normal or unavailable runtime.

- [x] **Step 4: Run bootstrap and desktop command suites**

Run: `cargo test -p prompt-hub-desktop --test bootstrap --test commands`

Expected: all tests pass without creating a relative `prompt-hub.db`.

- [x] **Step 5: Commit**

```powershell
git add apps/desktop/src-tauri/src/bootstrap.rs apps/desktop/src-tauri/src/lib.rs apps/desktop/src-tauri/tests/bootstrap.rs
git commit -m "fix(desktop): recover when app data path is unavailable"
```

### Task 4: Verify Stage A as a release-quality self-use slice

**Files:**
- Modify: `docs/security-review.md`
- Create: `docs/release-evidence/0.1.10/stage-a.md`

**Interfaces:**
- Consumes: fresh command output and exact Git commit.
- Produces: a durable evidence record without secrets or prompt bodies.

- [x] **Step 1: Run focused validation**

```powershell
cargo test -p prompt-store --test migrations --test backup
cargo test -p prompt-hub-desktop --test bootstrap --test commands
```

- [x] **Step 2: Run the required project gate**

```powershell
pnpm install --frozen-lockfile
pnpm lint
pnpm typecheck
pnpm test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --filter @prompt-hub/desktop build
```

- [x] **Step 3: Review the diff**

Run: `git diff --check` and inspect `git diff --stat` plus the exact migration/bootstrap changes.

- [x] **Step 4: Record evidence and limitations**

Document command results, schema version, covered corruption scenarios, and the remaining real installed-app smoke test. Do not include prompt bodies, private database paths, credentials, or authorization headers.

- [x] **Step 5: Commit**

```powershell
git add docs/security-review.md docs/release-evidence/0.1.10/stage-a.md docs/superpowers/specs/2026-08-29-stage-a-database-recovery-design.md docs/superpowers/plans/2026-08-29-stage-a-database-recovery.md
git commit -m "docs: record stage A recovery evidence"
```

## Self-review

- The plan covers every Stage A invariant: invalid lineage, incomplete structure, future versions, legacy v5 reopening, backup preservation, and unavailable AppData.
- It intentionally does not change schema version or prompt metadata; those belong to Stage B.
- All new behavior begins with an observed failing test.
- Every task has a focused verification command and a reviewable commit boundary.
