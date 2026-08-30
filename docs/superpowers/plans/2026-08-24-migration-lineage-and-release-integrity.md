# Migration Lineage and Release Integrity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every supported Prompt Hub database upgrade deterministic and recoverable, and prevent a feature-branch build from being shipped as a release package.

**Architecture:** Keep the existing numeric migrations as immutable legacy input, but add a durable `migration_ledger` with globally unique IDs and checksums for all future migrations. Database bootstrap will return a structured recovery state instead of panicking; the desktop UI will expose retry and redacted diagnostics. Release builds will pass a version/tag/ancestry gate and embed build provenance.

**Tech Stack:** Rust 2024, rusqlite, Tauri 2, React/TypeScript, Vitest, GitHub Actions, Node.js release script.

**Spec:** `docs/superpowers/specs/2026-08-24-migration-lineage-and-release-integrity-design.md`

## Global Constraints

- Existing `schema_migrations` and `PRAGMA user_version` remain historical diagnostics only; new migrations use immutable IDs and checksums.
- Upgrade is forward-only and creates/verifies a backup before changing an existing database.
- Unknown schemas, checksum conflicts, and failed transactions enter recovery without exposing prompt bodies, credentials, authorization headers, or unredacted paths.
- All behavior changes use TDD: failing focused test, minimal implementation, focused test, related suite.
- Formal release builds require a clean tree, a matching annotated `v<version>` tag, and tag ancestry from `origin/main`; candidate builds are explicitly non-public.

---

### Task 1: Add the migration ledger and immutable migration manifest

**Files:**
- Create: `crates/prompt-store/migrations/0007_migration_ledger.sql`
- Modify: `crates/prompt-store/src/migration.rs`
- Modify: `crates/prompt-store/src/lib.rs`
- Modify: `crates/prompt-store/Cargo.toml`
- Modify: `Cargo.toml`
- Test: `crates/prompt-store/tests/migrations.rs`

**Interfaces:**
- Produce `MigrationDefinition { id: &'static str, checksum_sha256: &'static str, sql: &'static str }` and `MigrationError` variants for unknown schema and checksum mismatch.
- Produce `Database::migration_ledger() -> Result<Vec<MigrationLedgerEntry>, StoreError>` for diagnostics/tests.

- [ ] **Step 1: Write failing tests** for a fresh database ledger, idempotent reopen, checksum mismatch, and the legacy v5 prompt-usage fixture. Assert prompt data and `last_used_at` survive.
- [ ] **Step 2: Run the focused suite and observe failure** with `cargo test -p prompt-store --test migrations migration_ledger -- --nocapture`.
- [ ] **Step 3: Add the ledger SQL and SHA-256 dependency.** The table must be `STRICT`, use `migration_id` as the primary key, and constrain provenance to `canonical` or `legacy_recovery`.
- [ ] **Step 4: Implement the manifest and backfill.** On a database without the ledger, recognize only the three documented historical fingerprints; insert immutable legacy records in one transaction, then run canonical migrations. On a known ledger ID, compare the stored checksum before executing any SQL.
- [ ] **Step 5: Preserve the current v5 compatibility path** as an explicit legacy fingerprint, then record it as `legacy/0.1.2-prompt-usage`; no branch-specific numeric version may skip a canonical SQL definition.
- [ ] **Step 6: Run focused and related suites** with `cargo test -p prompt-store --test migrations` and `cargo test -p prompt-store`.
- [ ] **Step 7: Commit** with `git commit -m "feat(store): add migration lineage ledger"`.

### Task 2: Harden backups, rollback, and recovery errors

**Files:**
- Modify: `crates/prompt-store/src/migration.rs`
- Modify: `crates/prompt-store/src/backup.rs`
- Modify: `crates/prompt-store/src/repository.rs`
- Modify: `crates/prompt-store/src/lib.rs`
- Test: `crates/prompt-store/tests/migrations.rs`
- Test: `crates/prompt-store/tests/backup.rs`

**Interfaces:**
- Produce `StoreError::RecoveryRequired { code: String, safe_message: String }` with no raw SQL/path/prompt content.
- Produce `Database::open` atomic behavior: a failed upgrade leaves the original database unchanged and returns a structured error.

- [ ] **Step 1: Write failing tests** for unknown structure, checksum conflict, injected migration failure, and restoring a pre-current backup followed by reopening.
- [ ] **Step 2: Run the tests** and capture the current partial-upgrade or missing-column failure.
- [ ] **Step 3: Implement read-only schema fingerprinting** and reject unsupported combinations before the write transaction; use a verified copy in the same data directory for the pre-migration backup.
- [ ] **Step 4: Make backup restore reopen through `Database::open`** (or an equivalent migration gate) before replacing the live repository, so restoring v4 cannot remove `last_used_at` from a running service.
- [ ] **Step 5: Run focused migration/backup tests**, then the full `prompt-store` suite.
- [ ] **Step 6: Commit** with `git commit -m "fix(store): make migration recovery atomic"`.

### Task 3: Add non-panicking Tauri bootstrap state

**Files:**
- Create: `apps/desktop/src-tauri/src/bootstrap.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `packages/contracts/src/index.ts`
- Test: `apps/desktop/src-tauri/tests/bootstrap.rs`

**Interfaces:**
- Produce `BootstrapState::{Ready, Recovery}` and serializable `BootstrapStatus { state, code, safe_message, backup_name }`.
- Produce commands `get_bootstrap_status`, `retry_database_bootstrap`, and `export_bootstrap_diagnostics`.

- [ ] **Step 1: Write failing Rust tests** that an injected store error returns `Recovery` and does not panic, and that retry after fixing the fixture returns `Ready`.
- [ ] **Step 2: Run `cargo test -p prompt-hub-desktop --test bootstrap`** and observe the missing bootstrap API.
- [ ] **Step 3: Move database/service construction into `bootstrap_application`** and store a mutex-protected bootstrap state. Register Tauri commands before attempting the database; remove setup `.expect` for database failures.
- [ ] **Step 4: Gate business commands** with `application_recovery_required` while in recovery; diagnostics must contain only version, error code, backup filename, and migration manifest summary.
- [ ] **Step 5: Add the TypeScript contract and run `pnpm test --filter @prompt-hub/contracts` plus the Rust desktop tests.
- [ ] **Step 6: Commit** with `git commit -m "feat(desktop): add recoverable bootstrap state"`.

### Task 4: Implement and test the recovery screen

**Files:**
- Create: `apps/desktop/src/features/recovery/RecoveryScreen.tsx`
- Create: `apps/desktop/src/features/recovery/recovery.css`
- Modify: `apps/desktop/src/app/AppShell.tsx`
- Modify: `apps/desktop/src/app/tauri.ts`
- Test: `apps/desktop/src/features/recovery/RecoveryScreen.test.tsx`

**Interfaces:**
- Consume `get_bootstrap_status`, `retry_database_bootstrap`, and `export_bootstrap_diagnostics`.
- Produce a recovery view with safe error text, backup availability/name, retry, diagnostics export, and a local recovery-help link.

- [ ] **Step 1: Write failing Vitest tests** for recovery rendering, retry success to the library view, and disabled business navigation while recovery is active.
- [ ] **Step 2: Run the focused Vitest test** and observe the absent view.
- [ ] **Step 3: Implement the recovery route and safe copy**; never render raw error strings containing paths or database content.
- [ ] **Step 4: Run the focused test, desktop tests, lint, and typecheck.
- [ ] **Step 5: Commit** with `git commit -m "feat(desktop): show database recovery screen"`.

### Task 5: Add release provenance verification

**Files:**
- Create: `scripts/verify-release.mjs`
- Create: `scripts/verify-release.test.mjs`
- Modify: `package.json`
- Modify: `.github/workflows/release.yml`
- Modify: `docs/release-checklist.md`
- Modify: `apps/desktop/src-tauri/tauri.conf.json`
- Test: `scripts/verify-release.test.mjs`

**Interfaces:**
- Produce CLI `node scripts/verify-release.mjs --channel=<candidate|release> --json-out=<path>`.
- Produce `build-info.json` fields `version`, `channel`, `gitCommit`, `migrationManifestSha256`, `builtAt`.

- [ ] **Step 1: Write failing Node tests** for matching versions, dirty tree, missing exact tag, non-main ancestry, candidate acceptance, and release rejection.
- [ ] **Step 2: Run `pnpm test scripts/verify-release.test.mjs`** and observe the missing script.
- [ ] **Step 3: Implement deterministic Git/JSON checks** using `child_process.execFileSync`; fail closed when `origin/main` or the exact tag is unavailable. Hash the sorted migration manifest, never source prompt data.
- [ ] **Step 4: Wire `release.yml`** to fetch full history and run the release gate before build/upload; name candidate artifacts with the commit SHA.
- [ ] **Step 5: Update the release checklist** with the fixture matrix and clean-profile acceptance evidence.
- [ ] **Step 6: Run the Node tests and a local candidate verification.
- [ ] **Step 7: Commit** with `git commit -m "ci: gate releases by provenance"`.

### Task 6: Add the upgrade matrix and clean-profile verification

**Files:**
- Create: `crates/prompt-store/tests/fixtures/legacy_v4.sql`
- Create: `crates/prompt-store/tests/fixtures/legacy_v5_prompt_usage.sql`
- Create: `crates/prompt-store/tests/fixtures/skill_v6.sql`
- Modify: `crates/prompt-store/tests/migrations.rs`
- Create: `tests/e2e/recovery.spec.ts`
- Modify: `playwright.config.ts`
- Modify: `docs/release-checklist.md`

**Interfaces:**
- Consume the migration ledger and recovery APIs from Tasks 1–4.
- Produce repeatable fixture tests and a Windows clean-profile smoke test.

- [ ] **Step 1: Write failing fixture tests** for all supported schemas, checksum conflict, unknown schema, rollback, and data preservation.
- [ ] **Step 2: Run the fixture suite** and record the first failing case.
- [ ] **Step 3: Move the inline fixtures into named SQL files** and assert prompt title/body hashes, favorites, versions, and usage timestamps after upgrade.
- [ ] **Step 4: Add the Playwright recovery smoke path** with a temporary isolated app-data directory; assert the app remains open and shows recovery actions.
- [ ] **Step 5: Run the complete verification matrix** from `AGENTS.md`.
- [ ] **Step 6: Commit** with `git commit -m "test: cover legacy database upgrade matrix"`.

### Task 7: Review, package, and install the protected build

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `README.md`
- Modify: `docs/release-checklist.md`

- [ ] **Step 1: Run the full required commands**: `pnpm install --frozen-lockfile`, `pnpm lint`, `pnpm typecheck`, `pnpm test`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `pnpm --filter @prompt-hub/desktop build`.
- [ ] **Step 2: Build the Windows installer** from the verified worktree; record exact installer paths and SHA-256 values.
- [ ] **Step 3: Install into an isolated Windows profile**, launch it, verify the library page and recovery smoke path, and confirm the installed executable version.
- [ ] **Step 4: Perform a reviewer pass** with `rg` for `.expect(` around bootstrap, raw prompt logging, and numeric migration skip logic; fix only confirmed issues and rerun affected tests.
- [ ] **Step 5: Update changelog/readme and commit** with `git commit -m "release: document migration recovery safeguards"`.
