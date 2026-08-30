CREATE TABLE migration_ledger (
    migration_id TEXT PRIMARY KEY,
    checksum_sha256 TEXT NOT NULL,
    applied_at INTEGER NOT NULL,
    provenance TEXT NOT NULL CHECK(provenance IN ('canonical', 'legacy_recovery'))
) STRICT;
