-- Token placements for monsters/NPCs on maps
-- (PCs are placed at runtime via frontend state, not persisted here)
CREATE TABLE token_placements (
    id TEXT PRIMARY KEY NOT NULL,
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,

    -- What this token represents (exactly one must be set)
    module_monster_id TEXT REFERENCES module_monsters(id) ON DELETE CASCADE,
    module_npc_id TEXT REFERENCES module_npcs(id) ON DELETE CASCADE,

    -- Position (grid coordinates)
    grid_x INTEGER NOT NULL,
    grid_y INTEGER NOT NULL,

    -- Display options
    label TEXT,           -- Optional override label
    faction_color TEXT,   -- Hex color for faction ring (e.g., "#FF0000" for enemy)
    hidden INTEGER NOT NULL DEFAULT 0,  -- Hidden from players initially (0=visible, 1=hidden)

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    -- Ensure exactly one entity type is referenced
    CHECK (
        (module_monster_id IS NOT NULL AND module_npc_id IS NULL) OR
        (module_monster_id IS NULL AND module_npc_id IS NOT NULL)
    )
);

CREATE INDEX idx_token_placements_map ON token_placements(map_id);
CREATE INDEX idx_token_placements_monster ON token_placements(module_monster_id);
CREATE INDEX idx_token_placements_npc ON token_placements(module_npc_id);

-- Trap placements on maps
CREATE TABLE map_traps (
    id TEXT PRIMARY KEY NOT NULL,
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,

    -- Position (grid coordinates)
    grid_x INTEGER NOT NULL,
    grid_y INTEGER NOT NULL,

    -- Trap info
    name TEXT NOT NULL,
    description TEXT,
    trigger_description TEXT,
    effect_description TEXT,
    dc INTEGER,  -- Detection/disarm DC

    -- State
    triggered INTEGER NOT NULL DEFAULT 0,  -- Has been triggered (0=armed, 1=triggered)
    visible INTEGER NOT NULL DEFAULT 0,    -- Visible to players (0=hidden, 1=visible)

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_map_traps_map ON map_traps(map_id);

-- Dynamic light sources (beyond UVTT static lights)
CREATE TABLE light_sources (
    id TEXT PRIMARY KEY NOT NULL,
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,

    -- Position (grid coordinates)
    grid_x INTEGER NOT NULL,
    grid_y INTEGER NOT NULL,

    -- Light properties
    name TEXT,
    bright_radius INTEGER NOT NULL,  -- Bright light radius in feet
    dim_radius INTEGER NOT NULL,     -- Dim light radius in feet
    color TEXT,                      -- Hex color (e.g., "#FFAA00" for torch)

    -- State
    active INTEGER NOT NULL DEFAULT 1,  -- Light is on (0=off, 1=on)

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_light_sources_map ON light_sources(map_id);
