CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL
) STRICT;

CREATE TABLE prompts (
    id TEXT PRIMARY KEY,
    status TEXT NOT NULL,
    effectiveness TEXT NOT NULL,
    current_version INTEGER NOT NULL CHECK (current_version > 0),
    entity_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER
) STRICT;

CREATE TABLE prompt_versions (
    prompt_id TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL CHECK (version_number > 0),
    version_id TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    description TEXT,
    content_json TEXT NOT NULL,
    actor TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (prompt_id, version_number)
) STRICT;

CREATE TABLE categories (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
) STRICT;

CREATE TABLE prompt_version_categories (
    prompt_id TEXT NOT NULL,
    version_number INTEGER NOT NULL,
    category_id INTEGER NOT NULL REFERENCES categories(id),
    PRIMARY KEY (prompt_id, version_number),
    FOREIGN KEY (prompt_id, version_number)
        REFERENCES prompt_versions(prompt_id, version_number) ON DELETE CASCADE
) STRICT;

CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
) STRICT;

CREATE TABLE prompt_version_tags (
    prompt_id TEXT NOT NULL,
    version_number INTEGER NOT NULL,
    tag_id INTEGER NOT NULL REFERENCES tags(id),
    PRIMARY KEY (prompt_id, version_number, tag_id),
    FOREIGN KEY (prompt_id, version_number)
        REFERENCES prompt_versions(prompt_id, version_number) ON DELETE CASCADE
) STRICT;

CREATE TABLE prompt_version_variables (
    prompt_id TEXT NOT NULL,
    version_number INTEGER NOT NULL,
    name TEXT NOT NULL,
    definition_json TEXT NOT NULL,
    PRIMARY KEY (prompt_id, version_number, name),
    FOREIGN KEY (prompt_id, version_number)
        REFERENCES prompt_versions(prompt_id, version_number) ON DELETE CASCADE
) STRICT;

CREATE TABLE prompt_sources (
    id TEXT PRIMARY KEY,
    prompt_id TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    location TEXT,
    collected_at INTEGER NOT NULL
) STRICT;

CREATE TABLE compatibilities (
    id INTEGER PRIMARY KEY,
    prompt_id TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
    tool TEXT NOT NULL,
    model TEXT,
    status TEXT NOT NULL,
    notes TEXT,
    confirmed_at INTEGER
) STRICT;

CREATE TABLE validation_records (
    id INTEGER PRIMARY KEY,
    prompt_id TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    rating INTEGER CHECK (rating BETWEEN 1 AND 5),
    notes TEXT,
    validated_at INTEGER NOT NULL
) STRICT;

CREATE TABLE audit_events (
    id TEXT PRIMARY KEY,
    prompt_id TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
    action TEXT NOT NULL,
    actor TEXT NOT NULL,
    occurred_at INTEGER NOT NULL
) STRICT;

CREATE TABLE import_jobs (
    id TEXT PRIMARY KEY,
    source_kind TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    diagnostics_json TEXT NOT NULL DEFAULT '{}'
) STRICT;

CREATE INDEX idx_prompts_status_updated ON prompts(status, updated_at DESC);
CREATE INDEX idx_prompt_sources_prompt ON prompt_sources(prompt_id);
CREATE INDEX idx_audit_events_prompt_time ON audit_events(prompt_id, occurred_at DESC);

