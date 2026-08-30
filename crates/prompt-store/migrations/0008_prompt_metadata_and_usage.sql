ALTER TABLE prompt_sources ADD COLUMN raw_excerpt TEXT;
ALTER TABLE prompt_sources ADD COLUMN import_job_id TEXT;

ALTER TABLE prompts ADD COLUMN imported_at INTEGER;
ALTER TABLE prompts ADD COLUMN last_validated_at INTEGER;

CREATE TABLE prompt_usage (
    prompt_id TEXT PRIMARY KEY REFERENCES prompts(id) ON DELETE CASCADE,
    use_count INTEGER NOT NULL CHECK(use_count >= 0),
    last_used_at INTEGER
) STRICT;

CREATE INDEX idx_prompt_usage_last_used_at ON prompt_usage(last_used_at DESC);
