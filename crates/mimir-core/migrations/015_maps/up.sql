-- Map table for storing map metadata and initial play state
-- UVTT files remain the source of truth for grid, walls, and lighting geometry
-- Maps can belong to a campaign (world maps) or a module (dungeon maps)

CREATE TABLE maps (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    module_id TEXT REFERENCES modules(id) ON DELETE CASCADE,

    -- Display info
    name TEXT NOT NULL,
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,

    -- UVTT asset reference (blob storage)
    uvtt_asset_id TEXT NOT NULL REFERENCES campaign_assets(id),

    -- Initial play state (not live state)
    -- bright = full visibility, dim = partial, dark = no visibility
    lighting_mode TEXT NOT NULL DEFAULT 'bright' CHECK (lighting_mode IN ('bright', 'dim', 'dark')),

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_maps_campaign ON maps(campaign_id);
CREATE INDEX idx_maps_module ON maps(module_id);
CREATE INDEX idx_maps_sort_order ON maps(campaign_id, sort_order);
