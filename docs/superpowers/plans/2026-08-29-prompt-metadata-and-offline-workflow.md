# Prompt Metadata and Offline Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make prompt provenance, use history, library ordering, copy/export, and the core offline workflow durable and verified for the local desktop product.

**Architecture:** Add a canonical v8 SQLite migration and repository APIs for usage statistics while retaining the prompt aggregate for versioned content. The Tauri command layer presents metadata and atomic mutations through stable contracts; React uses only those persisted fields and migrates the old localStorage counter once. Tests cover migrations, service flows, and visible client behavior.

**Tech Stack:** Rust 2024, rusqlite, Tauri 2, React, TypeScript, Vitest, Playwright.

**Spec:** `docs/superpowers/specs/2026-08-29-prompt-metadata-and-offline-workflow-design.md`

## Global Constraints

- Add only forward, immutable migration `0008_prompt_metadata_and_usage.sql`; register it in the Phase A ledger manifest and test it.
- Prompt bodies must never be logged; provenance is separate from body and copy defaults to body only.
- External and AI writes remain inbox-only; core workflows remain offline-capable.
- Every behavior change follows TDD: focused failing test, minimal implementation, focused passing test.
- Use real SQLite repository/service tests for persistence, migrations, backup/restore, and importer behavior.

---

### Task 1: Persist v8 provenance, import, validation, and usage fields

**Files:**
- Create: `crates/prompt-store/migrations/0008_prompt_metadata_and_usage.sql`
- Modify: `crates/prompt-domain/src/model.rs`
- Modify: `crates/prompt-store/src/migration.rs`
- Modify: `crates/prompt-store/src/repository.rs`
- Test: `crates/prompt-domain/tests/domain_rules.rs`
- Test: `crates/prompt-store/tests/migrations.rs`
- Test: `crates/prompt-store/tests/repository.rs`

**Interfaces:**
- Produce `PromptSource::with_provenance(..., raw_excerpt, import_job_id)` and source accessors.
- Produce `PromptRepository::{record_use,usage_stats,merge_legacy_usage}` returning `PromptUsageStats { use_count, last_used_at }`.
- Produce schema version 8 with source evidence and prompt time columns.

- [ ] Write tests that reject an oversized source excerpt, migrate a v7 fixture to v8, atomically increment usage, and retain the maximum imported legacy count.
- [ ] Run `cargo test -p prompt-store --test migrations --test repository` and observe missing v8 columns/APIs.
- [ ] Add the v8 SQL, manifest entry, source model fields, timestamp persistence, and usage repository implementation.
- [ ] Run the focused tests, then `cargo test -p prompt-domain -p prompt-store`.
- [ ] Commit `feat(store): persist prompt provenance and usage`.

### Task 2: Expose metadata and use persistence through desktop commands

**Files:**
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/tests/prompt_workflow.rs`
- Modify: `packages/contracts/src/index.ts`
- Test: `packages/contracts/src/index.test.ts`

**Interfaces:**
- Extend `PromptListItem` source evidence with `rawExcerpt`, `importJobId`, `importedAt`, `lastValidatedAt`, `useCount`, and `lastUsedAt`.
- Produce `recordPromptUse(id)` and `migrateLegacyPromptUsage(entries)` Tauri commands and command-client methods.

- [ ] Write failing command/service tests for imported source provenance, validation timestamp, use recording, and one-time legacy usage merge.
- [ ] Run `cargo test -p prompt-hub-desktop --test prompt_workflow` and contract tests to observe missing API fields.
- [ ] Add the minimal command/service/contract mapping, preserving all MCP restrictions.
- [ ] Run focused Rust and TypeScript tests.
- [ ] Commit `feat(desktop): expose durable prompt usage metadata`.

### Task 3: Replace browser-only usage sorting and separate copy/export metadata

**Files:**
- Modify: `apps/desktop/src/features/library/libraryView.ts`
- Modify: `apps/desktop/src/features/library/PromptLibrary.tsx`
- Modify: `apps/desktop/src/features/library/promptUsage.ts`
- Modify: `apps/desktop/src/features/library/PromptContentActions.tsx`
- Modify: `apps/desktop/src/app/AppShell.tsx`
- Test: `apps/desktop/src/features/library/libraryView.test.ts`
- Test: `apps/desktop/src/features/library/PromptLibrary.test.tsx`
- Test: `apps/desktop/src/features/library/PromptContentActions.test.tsx`

**Interfaces:**
- Produce `PromptLibrarySort = "default" | "recently_used" | "recently_added" | "recently_updated" | "most_used"`.
- `filterAndSortPrompts(prompts, filter, sort)` consumes `useCount` and `lastUsedAt` from the contract.
- `PromptContentActions` accepts `metadataExport` separately from `body`; default export is body-only.

- [ ] Write failing UI tests for each sort order, persisted use callback, one-time legacy transfer, copy body-only, and explicit metadata export.
- [ ] Run the focused Vitest tests and observe the old localStorage-only behavior.
- [ ] Implement the minimal UI/client changes and remove writes to the legacy counter after successful migration.
- [ ] Run focused tests, `pnpm lint`, and `pnpm typecheck`.
- [ ] Commit `feat(ui): use persisted library usage and explicit metadata export`.

### Task 4: Verify the offline workflow at real persistence boundaries

**Files:**
- Modify: `apps/desktop/src-tauri/tests/prompt_workflow.rs`
- Modify: `tests/e2e/offline-library.spec.ts`
- Create: `docs/release-evidence/0.1.10/stage-bc.md`

**Interfaces:**
- Real desktop service fixture covers create, TXT/MD/JSON/CSV import, invalid input, duplicate input, inbox publish, search, validation, use, favourite, backup/restore, delete/recover, and restart.
- Browser test covers library sorting/filtering, copy/export boundary, and bridge command invocation.

- [ ] Write focused failing workflow cases for the missing acceptance criteria and run the relevant test target.
- [ ] Add only the fixture helpers and implementation corrections needed to make the real path pass.
- [ ] Run `cargo test -p prompt-hub-desktop --test prompt_workflow`, `pnpm test`, and `pnpm exec playwright test tests/e2e/offline-library.spec.ts`.
- [ ] Record commands and factual outcomes in Stage B/C evidence.
- [ ] Commit `test: verify offline prompt workflow`.

### Task 5: Full review and verification

**Files:**
- Modify: `docs/release-evidence/0.1.10/stage-bc.md`

- [ ] Inspect diffs for migration-ID reuse, provenance placed in body, browser-only usage writes, raw prompt logging, and MCP mutations outside the inbox.
- [ ] Run the AGENTS.md verification commands exactly, including desktop build.
- [ ] Record all pass/fail evidence without claiming unrun installer smoke tests.
- [ ] Commit `docs: record stage b and c verification`.
