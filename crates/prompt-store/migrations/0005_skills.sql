CREATE TABLE skills (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    tool_kind TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    source_location TEXT NOT NULL,
    source_revision TEXT,
    content_hash TEXT NOT NULL,
    skill_markdown TEXT NOT NULL,
    risk_flags TEXT NOT NULL,
    review_status TEXT NOT NULL,
    review_notes TEXT,
    reviewed_at INTEGER,
    favorite INTEGER NOT NULL DEFAULT 0 CHECK(favorite IN (0, 1)),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(source_location, content_hash)
) STRICT;

CREATE TABLE skill_files (
    skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    relative_path TEXT NOT NULL,
    bytes INTEGER NOT NULL CHECK(bytes >= 0),
    sha256 TEXT NOT NULL,
    kind TEXT NOT NULL,
    PRIMARY KEY(skill_id, relative_path)
) STRICT;

CREATE TABLE skill_installations (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL UNIQUE REFERENCES skills(id) ON DELETE CASCADE,
    target_root TEXT NOT NULL,
    install_path TEXT NOT NULL,
    installed_hash TEXT NOT NULL,
    backup_path TEXT,
    installed_at INTEGER NOT NULL,
    last_verified_at INTEGER
) STRICT;

CREATE INDEX skill_list_order ON skills(favorite DESC, updated_at DESC, id ASC);
CREATE INDEX skill_review_status ON skills(review_status, updated_at DESC);
