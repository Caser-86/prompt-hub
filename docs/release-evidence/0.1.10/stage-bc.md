# Stage B/C verification evidence

Date: 2026-08-29
Worktree: `codex/legacy-v5-schema-recovery`
Scope: durable prompt metadata/use history and offline workflow closure.

## Implemented boundaries

- Schema version 8 is a forward migration (`20260829_01_prompt_metadata_and_usage`) adding source evidence, import/validation timestamps, and the `prompt_usage` table. Legacy v5 `prompts.last_used_at` is preserved and copied into usage history with an explicitly synthetic count of 1 because the old schema has no count.
- Imported file/URL prompts keep their import job ID and bounded source excerpt in source metadata. Prompt copying uses only the rendered body. Metadata is available only through the explicit “导出 Markdown（含来源）” action.
- Library ordering is persisted-data driven: default, recently used, recently added, recently updated, and most used. Old browser counts are merged once into SQLite; stale IDs are ignored and counts never decrease.

## Commands run

| Command | Result |
| --- | --- |
| `pnpm install --frozen-lockfile` | PASS — lockfile and workspace dependencies are current |
| `cargo test -p prompt-domain` | PASS — 10 tests |
| `cargo test -p prompt-store --test migrations --test repository` | PASS — 32 tests (21 migration, 11 repository) |
| `cargo test -p prompt-hub-desktop --test prompt_workflow` | PASS — 13 tests |
| `pnpm lint` | PASS |
| `pnpm typecheck` | PASS |
| `pnpm test` | PASS — contracts 22 tests, desktop 61 tests |
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — ignored baseline test remains explicitly ignored |
| `pnpm --filter @prompt-hub/desktop build` | PASS — Vite production build |
| `pnpm exec playwright test tests/e2e/offline-library.spec.ts --reporter=line` | PASS — 1 test |

The desktop Vitest suite still prints React `act(...)` warnings from the existing `App` tests; they are warnings only and do not indicate failed assertions. No test or build command failed in this gate run.

## Real workflow coverage

`apps/desktop/src-tauri/tests/prompt_workflow.rs` covers real SQLite service paths for manual creation, supported Markdown/TXT/JSON/CSV imports, invalid JSON, duplicate detection, inbox-only imports, validation metadata, use recording, legacy usage merge, restart persistence, search, version history, publish/edit/restore, and archive/delete/recover. Existing backup tests cover verified backup/restore and failed-restore safety. The Playwright path covers Tauri bridge create/history/search plus native copy status and export-menu visibility.

## Remaining manual-only check

The evidence above does not claim a signed Windows installer or native clipboard test; those belong to the release packaging stage. MCP and Skill security suites remain green in `cargo test --workspace` and were not broadened by Stage B/C.
