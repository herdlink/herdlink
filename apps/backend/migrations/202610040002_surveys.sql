CREATE TABLE surveys (
    id BLOB PRIMARY KEY NOT NULL CHECK (length(id) = 16),
    creator_id BLOB NOT NULL REFERENCES users(id),
    title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 200),
    description TEXT NOT NULL DEFAULT '' CHECK (length(description) <= 3000),
    questions_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE INDEX surveys_creator_idx ON surveys(creator_id, created_at DESC);
CREATE TABLE survey_communities (
    survey_id BLOB NOT NULL REFERENCES surveys(id) ON DELETE CASCADE,
    community_id BLOB NOT NULL REFERENCES communities(id) ON DELETE CASCADE,
    PRIMARY KEY (survey_id, community_id)
) STRICT;
CREATE INDEX survey_communities_community_idx ON survey_communities(community_id, survey_id);
CREATE TABLE survey_responses (
    survey_id BLOB NOT NULL REFERENCES surveys(id) ON DELETE CASCADE,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    answers_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (survey_id, user_id)
) STRICT;
