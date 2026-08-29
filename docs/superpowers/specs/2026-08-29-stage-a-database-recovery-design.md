# Stage A Database and Recovery Safety Design

## Status and scope

This design hardens the self-use Windows product against ambiguous, partially migrated, and path-unavailable startup states. It changes no user content model and adds no schema migration. The database remains SQLite schema version 7.

## Safety invariants

1. A database with `user_version` greater than the supported version is rejected without writes.
2. A committed version-7 database must contain the complete canonical migration ledger. The optional legacy prompt-usage record is accepted only with its known checksum.
3. After any migration, every required table and critical column for schema version 7 must exist before the transaction commits.
4. Unsupported or incomplete databases fail closed and are not repaired by guessing.
5. The known legacy 0.1.2 prompt-usage schema remains upgradeable and reopenable.
6. Failure to resolve the operating-system application-data directory enters recovery state and never opens a relative database path.
7. Recovery diagnostics contain safe codes and messages, not prompt bodies or private absolute paths.
8. Reopening a complete latest-version database performs no migration-ledger write and remains compatible with a concurrent writer.

## Database validation

`prompt-store` will separate three checks:

- migration lineage: known IDs and SHA-256 values;
- ledger completeness: all canonical IDs are present once version 7 is committed;
- schema shape: required tables and critical columns exist after migration.

Legacy versions without a ledger retain the current structural detection and forward-migration flow. Version 7 never receives ledger backfill on reopen; an incomplete ledger is evidence of an interrupted or foreign history and is rejected.

## Desktop bootstrap

`BootstrapRuntime` will represent whether a safe data directory is available. `BootstrapRuntime::unavailable()` starts in recovery with code `data_directory_unavailable`. `prepare_services` checks this state before opening SQLite or initializing credentials, so retry cannot create `prompt-hub.db` relative to the process working directory.

## Verification

Behavior is introduced test-first. Focused migration and bootstrap suites run after each change, followed by formatting, Clippy, the full Rust workspace, frontend checks, and the desktop production build required by `AGENTS.md`.
