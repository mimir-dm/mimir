-- Module NPCs: Links characters (with is_npc=true) to modules
-- NPCs are campaign characters, not catalog items like monsters
CREATE TABLE module_npcs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id INTEGER NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    role TEXT,                  -- quest_giver, antagonist, ally, informant, etc.
    encounter_tag TEXT,         -- Groups NPCs by scene (tavern_scene, boss_fight, etc.)
    notes TEXT,                 -- DM notes about this NPC's role in the module
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Index for fast module lookups
CREATE INDEX idx_module_npcs_module_id ON module_npcs(module_id);

-- Index for fast character lookups (find all modules an NPC appears in)
CREATE INDEX idx_module_npcs_character_id ON module_npcs(character_id);

-- Unique constraint: same character can't appear twice in the same module
CREATE UNIQUE INDEX idx_module_npcs_unique ON module_npcs(module_id, character_id);
