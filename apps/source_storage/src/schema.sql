CREATE TABLE IF NOT EXISTS records (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36),
    storage_uri TEXT NOT NULL,
    file_format TEXT NOT NULL CHECK (file_format IN ('pdf', 'xml', 'json', 'html')),
    content_hash TEXT NOT NULL CHECK (
        length(content_hash) = 64 AND content_hash NOT GLOB '*[^0-9a-f]*'
    ),
    lastfetched_at TEXT NOT NULL,
    category TEXT NOT NULL CHECK (category IN (
        'research', 'preprint', 'clinical_study', 'funding', 'reference',
        'model_resource', 'organization', 'experience', 'news', 'guideline'
    )),
    subtype TEXT NOT NULL,
    is_peer_reviewed INTEGER NOT NULL CHECK (is_peer_reviewed IN (0, 1)),
    source_name TEXT NOT NULL,
    external_id TEXT NOT NULL,
    doi TEXT,
    source_url TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT,
    language TEXT NOT NULL,
    authors TEXT CHECK (authors IS NULL OR
        CASE WHEN json_valid(authors) THEN json_type(authors) = 'array' ELSE 0 END),
    published_at TEXT,
    source_updated_at TEXT,
    record_status TEXT NOT NULL CHECK (record_status IN (
        'active', 'retracted', 'withdrawn', 'terminated'
    )),
    metadata TEXT NOT NULL CHECK (
        CASE WHEN json_valid(metadata) THEN json_type(metadata) = 'object' ELSE 0 END
    )
) STRICT;

-- Duplicates are discoverable, not forbidden: multiple sources can describe one paper.
CREATE INDEX IF NOT EXISTS records_content_hash_idx ON records(content_hash);
CREATE INDEX IF NOT EXISTS records_doi_idx ON records(doi);
CREATE INDEX IF NOT EXISTS records_source_idx ON records(source_name, external_id);
CREATE INDEX IF NOT EXISTS records_category_idx ON records(category);
