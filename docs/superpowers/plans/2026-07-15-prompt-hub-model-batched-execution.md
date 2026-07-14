# Prompt Hub Model-Batched Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and release a production-grade, local-first Prompt Hub desktop application with searchable prompt assets, safe imports, AI drafting, backup/recovery, and local Codex MCP integration.

**Architecture:** Use a pnpm workspace for a React/TypeScript desktop UI and a Cargo workspace for reusable Rust domain, storage, import, AI, and MCP components. SQLite remains the source of truth; the Tauri desktop process and STDIO MCP binary reuse the same core crates and stable schemas.

**Tech Stack:** Tauri 2, React, TypeScript, Vite, Rust stable, SQLite/FTS5, Vitest, Playwright, Cargo test, pnpm 11.

## Global Constraints

- The product specification at `docs/prompt-hub-product-spec.md` is the sole source of product requirements.
- Windows is the required first release platform; macOS and Linux build paths must remain viable.
- Core prompt management must work offline and without an AI provider key.
- AI and MCP writes may create inbox drafts only; they must never modify published prompts directly.
- Secrets must use OS credential storage and must never enter SQLite, logs, fixtures, or repository files.
- Every schema change requires a forward migration, migration test, and pre-migration backup behavior.
- No task is complete until its tests and the relevant integration checks pass.

---

## Model schedule

This schedule intentionally uses three continuous model sessions and only two switches.

| Session | Model and effort | Purpose | Exit condition |
| --- | --- | --- | --- |
| A | GPT-5.6 Sol · High | Architecture, contracts, migrations, search, security boundaries and hard foundations | Core crates and contracts pass unit/integration tests |
| B | GPT-5.6 Terra · Medium | Continuous product construction, including all medium- and low-complexity work | Full desktop workflows and MCP work locally |
| C | GPT-5.6 Sol · Extra High | Cross-module hardening, destructive-path testing, performance, packaging and release review | Release candidate passes every production gate |

Do not switch to Luna for the low-complexity batch inside Session B. Keeping Terra Medium active costs more tokens than Luna but avoids another switch and preserves implementation context.

Two high-strength sessions are necessary: foundations must be decided before construction, while security and release verification require the completed system. Combining them would either delay all development or audit code that does not yet exist.

## Planned repository structure

```text
apps/desktop/                    React/Vite UI and Tauri host
apps/desktop/src/features/       Feature-owned UI, state and tests
apps/desktop/src-tauri/          Tauri commands, app lifecycle and packaging
crates/prompt-domain/            Domain types, validation and service interfaces
crates/prompt-store/             SQLite migrations, repositories, FTS and backups
crates/prompt-import/            File, folder and URL ingestion
crates/prompt-ai/                Provider adapter and draft generation
crates/prompt-mcp/               STDIO MCP binary and JSON schemas
packages/contracts/              Generated/checked TypeScript IPC contracts
tests/fixtures/                  Sanitized import, migration and search fixtures
tests/e2e/                       Desktop and MCP end-to-end tests
docs/                            Product, architecture, operations and release docs
```

## Session A — Sol High: foundations and irreversible decisions

Keep Sol High selected for this entire session.

### Task A1: Development baseline and workspace

**Files:**
- Create: `Cargo.toml`, `rust-toolchain.toml`, `package.json`, `pnpm-workspace.yaml`
- Create: `apps/desktop/package.json`, `apps/desktop/src-tauri/Cargo.toml`
- Create: `.github/workflows/ci.yml`, `AGENTS.md`

**Produces:** A pinned, buildable workspace with shared lint, format and test commands.

- [ ] Install Rust stable with the Windows MSVC target because the current machine has Node 20.15.0 and pnpm 11.7.0 but no `rustc` or `cargo` on `PATH`.
- [ ] Initialize Git before generating project files so every later task has reviewable commits.
- [ ] Scaffold the pnpm and Cargo workspaces and pin the required Node, pnpm and Rust channels.
- [ ] Add root commands: `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm test:e2e`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [ ] Add CI that runs the same commands on Windows and uploads no prompt database or secret-containing artifact.
- [ ] Verify a clean desktop development launch and commit the baseline separately.

### Task A2: Domain contracts and lifecycle invariants

**Files:**
- Create: `crates/prompt-domain/src/model.rs`
- Create: `crates/prompt-domain/src/service.rs`
- Test: `crates/prompt-domain/tests/domain_rules.rs`

**Produces:** Stable Rust types for `Prompt`, `PromptVersion`, `PromptSource`, `PromptVariable`, `Compatibility`, `ValidationRecord`, `AuditEvent`, `PromptStatus`, and `EffectivenessStatus`.

- [ ] Write failing tests for required publication fields, lifecycle transitions, inbox-only external writes, version restoration and compatibility/effectiveness rules.
- [ ] Define UUID identifiers, UTC timestamps, explicit enums and validation errors; do not expose database row structs as public domain types.
- [ ] Implement lifecycle methods so restoring an old revision creates a new version and published records cannot be overwritten by AI or MCP callers.
- [ ] Run domain tests and serialize representative values to lock the wire naming convention.
- [ ] Commit the domain contract before database work begins.

### Task A3: SQLite schema, migrations and repository boundary

**Files:**
- Create: `crates/prompt-store/migrations/0001_initial.sql`
- Create: `crates/prompt-store/src/repository.rs`
- Test: `crates/prompt-store/tests/migrations.rs`, `crates/prompt-store/tests/repository.rs`

**Produces:** `PromptRepository` implementation with transactional versioning, source provenance, audit records and soft deletion.

- [ ] Write migration tests for a new database, repeated startup, interrupted migration recovery and upgrade backup creation.
- [ ] Create normalized tables for prompts, versions, sources, variables, categories, tags, tool/model compatibility, validation records, audit events and import jobs.
- [ ] Enable foreign keys, WAL mode and a bounded busy timeout; centralize connection creation so desktop and MCP use identical settings.
- [ ] Implement transaction-scoped create, publish, edit, archive, restore and soft-delete operations against the domain interfaces.
- [ ] Prove rollback behavior by injecting a failure between version and audit writes.
- [ ] Commit schema and repository as one independently testable unit.

### Task A4: Search contract and Chinese retrieval baseline

**Files:**
- Create: `crates/prompt-store/migrations/0002_search.sql`
- Create: `crates/prompt-store/src/search.rs`
- Test: `crates/prompt-store/tests/search.rs`, `tests/fixtures/search-zh.json`

**Produces:** `search_prompts(SearchQuery) -> SearchPage` with stable filtering, highlighting and ranking behavior.

- [ ] Create a sanitized Chinese fixture set covering titles, bodies, tags, variables, invalid prompts and model/tool compatibility.
- [ ] Write failing tests for Chinese substring queries, one- and two-character fallback, quoted phrases, combined filters, deterministic pagination and invalid-content demotion.
- [ ] Implement the FTS5 trigram index and explicit short-query fallback without interpolating user input into SQL.
- [ ] Define deterministic ranking as text relevance followed by effectiveness, rating, last verification time and update time.
- [ ] Add index rebuild and consistency-check operations for diagnostics and recovery.
- [ ] Record baseline query timings and commit the search contract.

### Task A5: Security and external-boundary contracts

**Files:**
- Create: `crates/prompt-import/src/url_policy.rs`
- Create: `crates/prompt-ai/src/credentials.rs`
- Create: `crates/prompt-mcp/schemas/*.json`
- Test: `crates/prompt-import/tests/url_policy.rs`, `crates/prompt-mcp/tests/schema_contract.rs`

**Produces:** Enforceable URL, credential, logging and MCP schema boundaries before external integrations are implemented.

- [ ] Write URL-policy tests blocking file URLs, loopback, private/link-local/reserved IP ranges, DNS rebinding and unsafe redirect targets.
- [ ] Define fixed request timeout, response-size and redirect limits in a single URL policy type.
- [ ] Define a credential-store interface backed by Windows Credential Manager through an audited library; prohibit plaintext fallback.
- [ ] Define versioned JSON Schemas for `search_prompts`, `get_prompt`, `render_prompt`, `save_prompt_draft` and structured errors.
- [ ] Mark MCP search/get/render as read-only and draft creation as a write operation; the write schema must contain no published-status field.
- [ ] Add a redaction utility contract for URLs, headers, secrets and prompt bodies before logging.
- [ ] Run contract and security-boundary tests and commit the foundation checkpoint.

### Session A checkpoint

Run:

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm lint
pnpm typecheck
pnpm test
```

Proceed only when all commands exit successfully and the database/search contracts match `docs/prompt-hub-product-spec.md`.

## Session B — Terra Medium: continuous product construction

Switch once to Terra Medium and keep it selected through all tasks in this session, including the final low-complexity batch.

### Task B1: Desktop shell and IPC boundary

**Files:**
- Create: `apps/desktop/src/app/*`, `apps/desktop/src/components/*`
- Create: `apps/desktop/src-tauri/src/commands.rs`
- Create: `packages/contracts/src/index.ts`
- Test: `apps/desktop/src/app/*.test.tsx`, `apps/desktop/src-tauri/tests/commands.rs`

**Produces:** Accessible application shell, typed command boundary, routing, global error handling and loading/empty states.

- [ ] Build the application frame, navigation, command palette, notification region and error boundary.
- [ ] Generate or validate TypeScript contracts from the Rust DTO schema so frontend and backend names cannot drift.
- [ ] Expose only service-level Tauri commands; prevent frontend code from issuing SQL or reading secret storage directly.
- [ ] Test keyboard navigation, focus restoration and command error presentation.

### Task B2: Prompt library, editor and versions

**Files:**
- Create: `apps/desktop/src/features/library/*`
- Create: `apps/desktop/src/features/editor/*`
- Create: `apps/desktop/src/features/history/*`
- Test: colocated `*.test.tsx` files and `apps/desktop/src-tauri/tests/prompt_workflow.rs`

**Produces:** Complete create, edit, publish, archive, soft-delete, restore, favorite and batch-edit workflows.

- [ ] Implement metadata editing for sources, timestamps, tools/models, effectiveness, rating and validation notes.
- [ ] Implement typed prompt variables with required/default values and a safe render preview.
- [ ] Show version diffs and make historical restoration create a new version.
- [ ] Add confirmation and recovery UI for destructive actions.
- [ ] Test the complete prompt lifecycle through the Tauri command boundary.

### Task B3: Search and discovery UI

**Files:**
- Create: `apps/desktop/src/features/search/*`
- Test: `apps/desktop/src/features/search/*.test.tsx`

**Produces:** Fast top-level search, advanced filters, result snippets, deterministic pagination and saved local view state.

- [ ] Implement debounced search without dropping the last user query or displaying stale responses.
- [ ] Add filters for category, tag, source, time, tool, model, effectiveness, rating, favorite and lifecycle.
- [ ] Display matched snippets and the metadata required by the product specification.
- [ ] Provide accessible empty, loading, index-rebuilding and error states.

### Task B4: Inbox and file/folder import

**Files:**
- Create: `crates/prompt-import/src/file.rs`, `crates/prompt-import/src/folder.rs`
- Create: `apps/desktop/src/features/inbox/*`, `apps/desktop/src/features/import/*`
- Test: `crates/prompt-import/tests/file_import.rs`, `apps/desktop/src/features/inbox/*.test.tsx`

**Produces:** Markdown, TXT, JSON and CSV import with fingerprints, duplicate detection, review and retry.

- [ ] Parse each format into a common `ImportCandidate` without publishing it.
- [ ] Persist import job progress, file fingerprints, warnings and per-item errors.
- [ ] Implement exact duplicate detection using normalized-body hashes; show similar candidates without automatic merging.
- [ ] Build inbox review, metadata completion, bulk publish, retry and skip workflows.
- [ ] Test malformed files, partial batch failure, repeated scans and Unicode paths.

### Task B5: Safe URL import

**Files:**
- Create: `crates/prompt-import/src/url.rs`, `crates/prompt-import/src/extract.rs`
- Create: `apps/desktop/src/features/import/UrlImport.tsx`
- Test: `crates/prompt-import/tests/url_import.rs`

**Produces:** URL extraction that obeys the Session A policy and creates traceable inbox candidates.

- [ ] Resolve and revalidate every redirect target before connecting.
- [ ] Stream responses under the configured size limit and accept only supported textual content types.
- [ ] Extract readable text without executing scripts, loading page subresources or trusting page instructions.
- [ ] Preserve canonical URL, retrieval time, title and extraction warnings as source evidence.
- [ ] Test timeout, redirect loops, oversized responses, unsupported content and SSRF cases.

### Task B6: AI generation and optimization

**Files:**
- Create: `crates/prompt-ai/src/provider.rs`, `crates/prompt-ai/src/draft.rs`
- Create: `apps/desktop/src/features/ai/*`, `apps/desktop/src/features/settings/AiSettings.tsx`
- Test: `crates/prompt-ai/tests/draft.rs`, `apps/desktop/src/features/ai/*.test.tsx`

**Produces:** Configurable OpenAI-compatible connection, generation, optimization and diff review that only creates inbox drafts.

- [ ] Store provider credentials exclusively through the credential interface and expose only masked settings to the UI.
- [ ] Implement connection testing, timeout, cancellation, rate-limit and malformed-response handling.
- [ ] Record model, generation time and source summary while excluding secrets and unnecessary sensitive input.
- [ ] Preserve user edits across provider failures and send all successful results to the inbox.
- [ ] Test that neither generation nor optimization can mutate a published record.

### Task B7: MCP server and Codex setup

**Files:**
- Create: `crates/prompt-mcp/src/main.rs`, `crates/prompt-mcp/src/tools.rs`
- Create: `apps/desktop/src/features/settings/McpSettings.tsx`
- Test: `crates/prompt-mcp/tests/tools.rs`, `tests/e2e/mcp.spec.ts`

**Produces:** Installable STDIO MCP binary implementing the four approved tools and a diagnostics/setup UI.

- [ ] Implement every MCP input/output directly from the versioned schemas created in Session A.
- [ ] Ensure search returns summaries, get returns complete records, render validates variables, and save always creates an inbox draft.
- [ ] Handle database migration, lock and unavailable states with stable error codes.
- [ ] Add generated Codex configuration, enable/disable instructions and a local health check.
- [ ] Test tool discovery and calls through a real STDIO client process.

### Task B8: Backup, restore, diagnostics and onboarding

**Files:**
- Create: `crates/prompt-store/src/backup.rs`
- Create: `apps/desktop/src/features/settings/BackupSettings.tsx`
- Create: `apps/desktop/src/features/diagnostics/*`, `apps/desktop/src/features/onboarding/*`
- Test: `crates/prompt-store/tests/backup.rs`, `tests/e2e/onboarding.spec.ts`

**Produces:** Integrity-checked backups, safe restore preview, redacted diagnostics and first-run guidance.

- [ ] Create migration, restore and permanent-delete safety backups with timestamps and integrity metadata.
- [ ] Implement retention settings, integrity verification, restore preview and pre-replacement safety copy.
- [ ] Display application/database versions, search index state, import state, MCP state and redacted logs.
- [ ] Explain data location, local privacy, backup setup, MCP installation and AI credential storage during onboarding.
- [ ] Test corrupt backups, insufficient disk space, interrupted restoration and successful round trips.

### Task B9: Low-complexity completion batch

**Files:**
- Create: `docs/user-guide.md`, `docs/import-formats.md`, `docs/mcp-setup.md`, `docs/privacy.md`
- Create: `tests/fixtures/import/*`, `tests/fixtures/mcp/*`
- Modify: UI copy and accessible labels throughout `apps/desktop/src/features/`

**Produces:** User-facing documentation, sanitized fixtures, consistent copy, keyboard labels and release-ready empty states.

- [ ] Write user guidance from verified application behavior only.
- [ ] Add valid and invalid fixtures for every import format and MCP schema.
- [ ] Normalize Chinese UI terminology for inbox, publication, validation and compatibility states.
- [ ] Audit labels, tab order, focus behavior, contrast and screen-reader names.
- [ ] Remove development-only data, placeholder copy and unreferenced assets.

### Session B checkpoint

Run:

```powershell
pnpm lint
pnpm typecheck
pnpm test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm test:e2e
pnpm --filter desktop tauri dev
```

Proceed only when a user can complete the offline library workflow and a real STDIO client can call all four MCP tools.

## Session C — Sol Extra High: hardening and production release

Switch once to Sol Extra High and keep it selected until the release decision.

### Task C1: Cross-module correctness and threat review

**Files:**
- Modify only files implicated by demonstrated failures
- Create: `docs/security-review.md`
- Test: `tests/security/*`, relevant crate and UI regression tests

**Produces:** Evidence-backed review of trust boundaries, destructive operations and concurrency behavior.

- [ ] Trace untrusted data from file, URL, model and MCP inputs through storage, rendering, logs and exports.
- [ ] Verify SSRF defenses after DNS resolution and on every redirect; test local, private, link-local and reserved ranges.
- [ ] Verify prompt content cannot execute scripts or become shell/application commands.
- [ ] Verify credentials and prompt bodies are absent from logs, crash diagnostics and repository fixtures.
- [ ] Stress simultaneous desktop/MCP reads and writes, backup operations and index rebuilds.
- [ ] Fix only reproduced issues, add regression tests and record evidence in the security review.

### Task C2: Recovery, migration and failure-path verification

**Files:**
- Create: `tests/e2e/recovery.spec.ts`
- Create: `docs/recovery-runbook.md`
- Modify: migration/backup code only for reproduced defects

**Produces:** Verified recovery from corrupt data, interrupted migration, failed import and interrupted restore.

- [ ] Execute clean install, upgrade, failed upgrade and repeated-start migration scenarios.
- [ ] Corrupt copies of test databases and backups, then verify diagnostics and safe recovery behavior.
- [ ] Interrupt import, backup and restore operations at controlled points and verify atomicity.
- [ ] Restore a complete library into a clean application and compare record/version/source counts and content hashes.
- [ ] Document commands and user-visible recovery steps from verified behavior.

### Task C3: Performance and end-to-end release acceptance

**Files:**
- Create: `tests/performance/search.rs`, `tests/e2e/release.spec.ts`
- Create: `docs/release-evidence/README.md`

**Produces:** Reproducible evidence for search performance and every product acceptance criterion.

- [ ] Generate sanitized libraries at 1,000, 10,000 and 50,000 prompts with Chinese and mixed-language content.
- [ ] Measure cold/warm search, combined filters, index rebuild, startup and backup duration on the release machine.
- [ ] Run complete user journeys: install, onboard, create, import, review, search, render, version, export, backup, restore and MCP access.
- [ ] Record exact commands, versions, durations and pass/fail results without claiming unsupported hardware-wide guarantees.
- [ ] Investigate and fix regressions before packaging.

### Task C4: Windows packaging and release gate

**Files:**
- Modify: `apps/desktop/src-tauri/tauri.conf.json`
- Create: `.github/workflows/release.yml`, `docs/release-checklist.md`, `CHANGELOG.md`
- Test: packaged application smoke tests

**Produces:** A versioned Windows installer with documented update, backup, signing and rollback behavior.

- [ ] Build the production application and MCP sidecar from a clean checkout.
- [ ] Verify the installer, uninstall behavior, data preservation, first launch and MCP executable discovery on a clean Windows user profile.
- [ ] Configure code signing only when a valid user-controlled certificate is available; do not fabricate a signed-release result.
- [ ] Configure user-confirmed updates with automatic pre-update backup and recovery instructions.
- [ ] Publish checksums, changelog, database compatibility notes and release evidence.
- [ ] Block release if signing requirements, tests, security review, recovery tests or installer smoke tests are incomplete.

### Final release commands

```powershell
pnpm install --frozen-lockfile
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm lint
pnpm typecheck
pnpm test
pnpm test:e2e
pnpm --filter desktop tauri build
```

All commands must exit successfully. Then install the generated package in a clean Windows profile and run `tests/e2e/release.spec.ts` against the packaged application before declaring the release complete.

## External prerequisites and honest blockers

- Rust stable and the Windows MSVC build prerequisites must be installed before Session A implementation.
- A real code-signing certificate is required to claim a signed public Windows release; without it, development and internal installers can be tested but the public release gate remains blocked.
- AI provider integration tests require a user-supplied test credential; core offline acceptance must not depend on that credential.
- Automatic update publishing requires a user-controlled release host and signing configuration; local update behavior can be tested with a controlled fixture server.

## Execution rule

At each task boundary, review the diff, run the listed tests and commit only the task's files. Do not move to the next model session while its checkpoint is failing. If a requirement cannot be verified, report it as incomplete with evidence instead of marking it complete.
