-- Player links (MIMIR-T-0716, ADR MIMIR-A-0010): one link per player
-- character. Only a SHA-256 hash of the link token is stored.

CREATE TABLE player_links (
    character_id TEXT PRIMARY KEY NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    -- hex SHA-256 of the link token
    token_hash TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
