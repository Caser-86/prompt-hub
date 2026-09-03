# Formal Product Gap Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the remaining formal-product gaps identified in the 0.1.10 audit without weakening offline, migration, MCP, or Skill safety boundaries.

**Architecture:** Keep the existing local SQLite/Tauri service boundary. Add narrowly scoped command-client capabilities for prompt lookup and atomic metadata/content revision, make search state request-safe, and expose recovery actions only through verified backup APIs. Preserve inbox-only writes for imports, AI, and MCP.

**Tech Stack:** React + TypeScript, Tauri/Rust, SQLite/FTS5, Vitest, Playwright, Cargo tests.

**Spec:** `docs/prompt-hub-product-spec.md`

## Global Constraints

- Core prompt management remains offline-capable and independent of model credentials.
- External and AI writes may create inbox drafts only; never overwrite published prompts.
- Every schema change requires a forward migration and migration test.
- Every migration, restore, and permanent deletion must create a timestamped safety backup first.
- Never store or log credentials, raw authorization headers, or unredacted prompt bodies.

---

### Task 1: Make advanced search functional and reusable

**Files:**
- Modify: `apps/desktop/src/features/search/PromptSearch.tsx`
- Modify: `packages/contracts/src/index.ts`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Test: `apps/desktop/src/features/search/PromptSearch.test.tsx`
- Test: `apps/desktop/src-tauri/tests/prompt_workflow.rs`

- [x] Write failing tests for filter-only search, stale filter responses, and selecting a search hit.
- [x] Run the focused tests and confirm the new assertions fail for the current implementation.
- [x] Add a validated `getPrompt` desktop command and make search results open the corresponding detail view.
- [x] Permit empty text when at least one filter is active, increment request generation for every query/filter change, and render source/model metadata with localized dates.
- [x] Run focused frontend and Rust tests, then the full frontend suite.

### Task 2: Add complete prompt revision and atomic metadata editing

**Files:**
- Modify: `packages/contracts/src/index.ts`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src/app/AppShell.tsx`
- Modify: `apps/desktop/src/features/editor/PromptEditor.tsx`
- Modify: `apps/desktop/src/features/editor/PromptMetadataEditor.tsx`
- Test: `apps/desktop/src/features/editor/PromptEditor.test.tsx`
- Test: `apps/desktop/src/features/editor/PromptMetadataEditor.test.tsx`
- Test: `apps/desktop/src-tauri/tests/prompt_workflow.rs`

- [x] Write failing tests for editing an existing prompt, preserving untouched metadata, tag editing, variable removal/duplicate validation, and atomic metadata failure.
- [x] Run focused tests and confirm they fail for the current UI/client.
- [x] Reuse the existing revision domain operation through a typed desktop command; initialize forms from current values and return the updated list item.
- [x] Add one service-level transaction for compatibility and validation updates, or an explicit rollback path, and refresh the selected prompt after success.
- [x] Run focused and full tests.

### Task 3: Make migration repair and recovery self-contained

**Files:**
- Modify: `crates/prompt-store/src/migration.rs`
- Modify: `apps/desktop/src-tauri/src/bootstrap.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `packages/contracts/src/index.ts`
- Modify: `apps/desktop/src/app/AppShell.tsx`
- Modify: `apps/desktop/src/features/recovery/RecoveryScreen.tsx`
- Test: `crates/prompt-store/tests/migrations.rs`
- Test: `apps/desktop/src/features/recovery/RecoveryScreen.test.tsx`

- [x] Write a failing migration test proving a latest-schema ledger repair creates a pre-repair backup.
- [x] Write a failing recovery test proving a verified backup can be selected and restored from the recovery screen.
- [x] Run the focused tests and confirm failure.
- [x] Create a backup before any known latest-schema repair and expose only verified backup preview/restore operations to recovery.
- [x] Preserve original data on failed replacement and show the pre-replacement backup path.
- [x] Run all migration, recovery, and Rust tests.

### Task 4: Acceptance, scale, and failure-state hardening

**Files:**
- Modify: `apps/desktop/src/features/library/PromptLibrary.tsx`
- Modify: `apps/desktop/src/components/CommandPalette.tsx`
- Modify: `apps/desktop/src/features/skills/SkillLibrary.tsx`
- Modify: `apps/desktop/src/features/ai/AiDraftGenerator.tsx`
- Add/modify: `tests/e2e/real-desktop-smoke.spec.ts`
- Modify: `.github/workflows/ci.yml`
- Modify: `docs/release-evidence/0.1.10/packaging.md`

- [x] Add failing tests for visible favorite/archive errors and disabled concurrent AI submission. A real installed-package boundary remains separately documented because the current test harness uses the Tauri command boundary mock.
- [x] Replace silent promise failures with user-visible retryable states and disable duplicate AI, restore, lifecycle, and library-load submissions.
- [x] Add batched summary retrieval before the library exceeds the measured dataset range; pagination remains a follow-up if the local dataset grows beyond the measured range.
- [x] Run the available browser smoke and all required verification commands; a fresh installed-package lifecycle smoke and external Codex discovery remain release-evidence follow-ups.
