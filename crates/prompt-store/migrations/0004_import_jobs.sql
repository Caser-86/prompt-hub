ALTER TABLE import_jobs ADD COLUMN source_path TEXT;
ALTER TABLE import_jobs ADD COLUMN source_fingerprint TEXT;

CREATE TABLE import_job_items (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES import_jobs(id) ON DELETE CASCADE,
    source_path TEXT NOT NULL,
    body_fingerprint TEXT,
    title TEXT,
    outcome TEXT NOT NULL,
    warnings_json TEXT NOT NULL DEFAULT '[]',
    error_message TEXT,
    prompt_id TEXT REFERENCES prompts(id) ON DELETE SET NULL,
    recorded_at INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_import_jobs_started_at ON import_jobs(started_at DESC);
CREATE INDEX idx_import_job_items_job_id ON import_job_items(job_id, recorded_at ASC);
CREATE INDEX idx_import_job_items_fingerprint ON import_job_items(body_fingerprint);
