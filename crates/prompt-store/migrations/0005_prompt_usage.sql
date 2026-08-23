ALTER TABLE prompts ADD COLUMN last_used_at INTEGER;

CREATE INDEX idx_prompts_last_used
    ON prompts(last_used_at DESC, created_at DESC, id ASC);
