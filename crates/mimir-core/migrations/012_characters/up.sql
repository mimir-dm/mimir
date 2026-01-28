-- Characters table for player characters and NPCs
-- Characters can optionally belong to a campaign

CREATE TABLE characters (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT REFERENCES campaigns(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    is_npc INTEGER NOT NULL DEFAULT 0,
    player_name TEXT,

    -- Race and background (catalog references)
    race_name TEXT,
    race_source TEXT,
    background_name TEXT,
    background_source TEXT,

    -- Ability scores (standard D&D 5e range 1-30, default 10)
    strength INTEGER NOT NULL DEFAULT 10,
    dexterity INTEGER NOT NULL DEFAULT 10,
    constitution INTEGER NOT NULL DEFAULT 10,
    intelligence INTEGER NOT NULL DEFAULT 10,
    wisdom INTEGER NOT NULL DEFAULT 10,
    charisma INTEGER NOT NULL DEFAULT 10,

    -- Currency (copper, silver, electrum, gold, platinum)
    cp INTEGER NOT NULL DEFAULT 0,
    sp INTEGER NOT NULL DEFAULT 0,
    ep INTEGER NOT NULL DEFAULT 0,
    gp INTEGER NOT NULL DEFAULT 0,
    pp INTEGER NOT NULL DEFAULT 0,

    -- Roleplay elements (for PCs)
    traits TEXT,
    ideals TEXT,
    bonds TEXT,
    flaws TEXT,

    -- NPC-specific fields
    role TEXT,
    location TEXT,
    faction TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_characters_campaign ON characters(campaign_id);
CREATE INDEX idx_characters_is_npc ON characters(is_npc);

-- Character sources: which source books a character can use
-- If empty, inherits from campaign sources
CREATE TABLE character_sources (
    id TEXT PRIMARY KEY NOT NULL,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    source_code TEXT NOT NULL REFERENCES catalog_sources(code),
    UNIQUE(character_id, source_code)
);

CREATE INDEX idx_character_sources_character ON character_sources(character_id);
