CREATE TABLE prompt_favorites (
    prompt_id TEXT PRIMARY KEY REFERENCES prompts(id) ON DELETE CASCADE,
    marked_at INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_prompt_favorites_marked ON prompt_favorites(marked_at DESC);
