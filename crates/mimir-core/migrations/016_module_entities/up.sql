-- Module monsters and NPCs
-- These are module-scoped entities that can be placed on maps

-- Module monsters: catalog monster instances with optional customizations
CREATE TABLE module_monsters (
    id TEXT PRIMARY KEY NOT NULL,
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,

    -- Reference to catalog (name + source for lookup)
    monster_name TEXT NOT NULL,
    monster_source TEXT NOT NULL,

    -- Customizations (NULL = use catalog value)
    display_name TEXT,  -- Override name (e.g., "Goblin Chief" instead of "Goblin")
    notes TEXT,         -- DM notes for this instance

    -- Quantity for encounters
    quantity INTEGER NOT NULL DEFAULT 1,

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_module_monsters_module ON module_monsters(module_id);

-- Module NPCs: custom characters created by the DM
CREATE TABLE module_npcs (
    id TEXT PRIMARY KEY NOT NULL,
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,

    -- Core info
    name TEXT NOT NULL,
    role TEXT,  -- e.g., "Quest Giver", "Merchant", "Villain"
    description TEXT,

    -- Appearance
    appearance TEXT,

    -- Personality & motivation
    personality TEXT,
    motivation TEXT,
    secrets TEXT,  -- DM-only info

    -- Stats (optional - not all NPCs need stat blocks)
    stat_block TEXT,  -- JSON blob if needed

    -- Token image (optional)
    token_asset_id TEXT REFERENCES campaign_assets(id),

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_module_npcs_module ON module_npcs(module_id);
