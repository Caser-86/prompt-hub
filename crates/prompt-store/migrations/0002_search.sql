CREATE VIRTUAL TABLE prompt_fts USING fts5(
    prompt_id UNINDEXED,
    title,
    body,
    description,
    tags,
    variables,
    tokenize = 'trigram'
);

INSERT INTO prompt_fts(prompt_id, title, body, description, tags, variables)
SELECT
    p.id,
    v.title,
    v.body,
    COALESCE(v.description, ''),
    COALESCE((
        SELECT group_concat(t.name, ' ')
        FROM prompt_version_tags pvt
        JOIN tags t ON t.id = pvt.tag_id
        WHERE pvt.prompt_id = p.id AND pvt.version_number = p.current_version
    ), ''),
    COALESCE((
        SELECT group_concat(pvv.name, ' ')
        FROM prompt_version_variables pvv
        WHERE pvv.prompt_id = p.id AND pvv.version_number = p.current_version
    ), '')
FROM prompts p
JOIN prompt_versions v
  ON v.prompt_id = p.id AND v.version_number = p.current_version;

