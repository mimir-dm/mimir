-- Create tokens table for map token placement
CREATE TABLE tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    map_id INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    token_type TEXT NOT NULL DEFAULT 'monster',  -- 'monster', 'pc', 'npc', 'trap', 'marker'
    size TEXT NOT NULL DEFAULT 'medium',  -- 'tiny', 'small', 'medium', 'large', 'huge', 'gargantuan'
    x REAL NOT NULL DEFAULT 0.0,  -- Grid position (float for sub-grid positioning)
    y REAL NOT NULL DEFAULT 0.0,
    visible_to_players INTEGER NOT NULL DEFAULT 1,
    color TEXT,  -- Fallback color if no image (hex format)
    image_path TEXT,  -- Custom token image path
    monster_id INTEGER REFERENCES catalog_monsters(id) ON DELETE SET NULL,  -- Link to catalog monster
    character_id INTEGER REFERENCES characters(id) ON DELETE SET NULL,  -- Link to PC/NPC
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for fast lookup by map
CREATE INDEX idx_tokens_map_id ON tokens(map_id);

-- Index for finding tokens linked to specific monsters/characters
CREATE INDEX idx_tokens_monster_id ON tokens(monster_id);
CREATE INDEX idx_tokens_character_id ON tokens(character_id);
