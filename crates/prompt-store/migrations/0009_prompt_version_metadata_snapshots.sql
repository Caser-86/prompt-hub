ALTER TABLE prompt_versions ADD COLUMN metadata_json TEXT NOT NULL DEFAULT '{}';

-- Pre-v9 rows did not retain metadata separately. Reuse the persisted prompt
-- entity as a safe compatibility snapshot so opening and restoring old data
-- never silently clears the current source, validation, or tool records.
UPDATE prompt_versions
SET metadata_json = (
    SELECT entity_json FROM prompts WHERE prompts.id = prompt_versions.prompt_id
)
WHERE metadata_json = '{}';
