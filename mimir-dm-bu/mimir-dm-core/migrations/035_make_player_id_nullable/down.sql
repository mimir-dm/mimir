-- Revert player_id to NOT NULL
-- Note: This will fail if any NPCs exist with NULL player_id

-- Create new table with NOT NULL player_id
CREATE TABLE characters_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    campaign_id INTEGER,
    player_id INTEGER NOT NULL,
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

-- Copy existing data (will fail if any player_id is NULL)
INSERT INTO characters_new (
    id, campaign_id, player_id, character_name, is_npc,
    current_level, current_version, directory_path,
    created_at, last_updated_at, class, race
)
SELECT
    id, campaign_id, player_id, character_name, is_npc,
    current_level, current_version, directory_path,
    created_at, last_updated_at, class, race
FROM characters
WHERE player_id IS NOT NULL;

-- Drop old table
DROP TABLE characters;

-- Rename new table
ALTER TABLE characters_new RENAME TO characters;

-- Recreate indexes
CREATE INDEX idx_characters_campaign ON characters(campaign_id);
CREATE INDEX idx_characters_player ON characters(player_id);
CREATE INDEX idx_characters_name ON characters(character_name);
CREATE INDEX idx_characters_is_npc ON characters(is_npc);
