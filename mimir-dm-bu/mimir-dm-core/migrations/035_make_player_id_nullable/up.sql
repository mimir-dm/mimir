-- Make player_id nullable for NPCs (SQLite requires table recreation)
-- NPCs don't belong to a specific player

-- Create new table with nullable player_id
CREATE TABLE characters_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    campaign_id INTEGER,
    player_id INTEGER,  -- Now nullable for NPCs
    character_name TEXT NOT NULL,
    is_npc INTEGER NOT NULL DEFAULT 0,
    current_level INTEGER NOT NULL DEFAULT 1,
    current_version INTEGER NOT NULL DEFAULT 1,
    directory_path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    class TEXT,
    race TEXT,
    FOREIGN KEY (campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE
);

-- Copy existing data
INSERT INTO characters_new (
    id, campaign_id, player_id, character_name, is_npc,
    current_level, current_version, directory_path,
    created_at, last_updated_at, class, race
)
SELECT
    id, campaign_id, player_id, character_name, is_npc,
    current_level, current_version, directory_path,
    created_at, last_updated_at, class, race
FROM characters;

-- Drop old table
DROP TABLE characters;

-- Rename new table
ALTER TABLE characters_new RENAME TO characters;

-- Recreate indexes
CREATE INDEX idx_characters_campaign ON characters(campaign_id);
CREATE INDEX idx_characters_player ON characters(player_id);
CREATE INDEX idx_characters_name ON characters(character_name);
CREATE INDEX idx_characters_is_npc ON characters(is_npc);
