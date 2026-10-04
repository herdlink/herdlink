CREATE TABLE graph_chats (
    id BLOB PRIMARY KEY NOT NULL CHECK (length(id) = 16),
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    messages_json TEXT NOT NULL DEFAULT '[]',
    input_json TEXT NOT NULL DEFAULT '[]',
    graph_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE INDEX graph_chats_user_updated ON graph_chats(user_id, updated_at DESC);
