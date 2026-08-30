# Prompt Metadata and Offline Workflow Design

## Purpose

Complete the remaining local-first prompt-library work needed for a dependable personal desktop product: durable provenance and use history, predictable library ordering, clean copy/export boundaries, and tested offline workflows. This design implements the existing product baseline; it does not add cloud sync, automatic publishing, or automatic Skill execution.

## Scope and decisions

1. Add one immutable database migration, `0008_prompt_metadata_and_usage.sql`. It adds source evidence columns (`raw_excerpt`, `import_job_id`), prompt timestamps (`imported_at`, `last_validated_at`), and a separate `prompt_usage` table (`prompt_id`, `use_count`, `last_used_at`). The migration ID is new and is registered in the migration ledger; no existing numeric migration is reused.
2. Keep prompt body and provenance separate. Imported prompt sources receive an excerpt and import-job ID as metadata. Copy always sends only the rendered body. Markdown export defaults to the body and may explicitly include a labelled metadata appendix.
3. Keep use statistics relational rather than serializing them into the prompt aggregate. A repository API atomically records a use and returns `PromptUsageStats`; library list rows contain the current count and timestamp through a read-only join.
4. Import the legacy browser count map once. The desktop command merges each positive count into SQLite using `MAX(existing, legacy)` and never invents a historical last-used time. The client marks the one-time transfer complete only after the command succeeds, then stops writing the old localStorage counter.
5. Sorting is explicit: `default`, `recently_used`, `recently_added`, `recently_updated`, and `most_used`. The default ranks favourites first, then records with actual use history, then newer updates; each selected mode has a deterministic UUID tie-breaker. All list sorting uses server-delivered persisted statistics, not browser-local state.
6. The desktop test boundary is layered: Rust tests exercise the real SQLite repository and command services in temporary directories; browser tests exercise the visible library, copy/export, filtering, and legacy-transfer contract through the Tauri bridge mock. A manual Windows smoke checklist remains necessary for installer and native clipboard integration.

## Data and failure handling

`raw_excerpt` is optional and limited by the domain constructor to avoid accidental full-document duplication. `import_job_id` is optional for manual, MCP, and AI sources and required whenever a file, folder, or URL importer provides a job. `imported_at` is populated for importer-created prompts and left `NULL` for legacy/manual records. Recording validation updates `last_validated_at` atomically with the versioned prompt save.

Usage recording is an UPSERT in one SQLite transaction. Unknown IDs produce `NotFound`; zero/negative legacy counts are ignored. The usage table cascades on prompt deletion. Backup/restore and existing migration recovery protect it through the Phase A ledger/backup gate.

## Acceptance criteria

- A v7 database upgrades to v8 without losing prompts, sources, favourites, versions, or legacy `last_used_at` data.
- File and URL imports preserve their job ID, collection/import time, source location, and optional excerpt independently of the prompt body.
- Validations expose a last-validation time; library rows expose durable count and last-used time.
- Copy excludes provenance. Export includes no metadata by default and offers an explicit metadata-inclusive format.
- Favourites and all four requested orderings work after restart; legacy localStorage counts are copied once into SQLite.
- Offline create, TXT/MD/JSON/CSV import, duplicate/invalid handling, inbox publish, search, variable render, copy/export, favourite, restart, backup/restore, delete/recover are covered by real service tests or browser tests at the appropriate boundary.

## Out of scope

- Cloud sync, team sharing, telemetry, installer signing, browser extension collection, and Skill installation changes.
- Reconstructing an exact historical timestamp from the old browser count map; it did not store timestamps. New use events persist exact timestamps.
