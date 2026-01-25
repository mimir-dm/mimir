-- Add fog of war support to maps

-- Add fog_enabled column to maps table
ALTER TABLE maps ADD COLUMN fog_enabled INTEGER NOT NULL DEFAULT 0;

-- Revealed areas table for fog of war
-- Stores rectangular regions that have been revealed by the DM
CREATE TABLE fog_revealed_areas (
    id TEXT PRIMARY KEY NOT NULL,
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,

    -- Revealed area bounds (in grid coordinates)
    x REAL NOT NULL,
    y REAL NOT NULL,
    width REAL NOT NULL,
    height REAL NOT NULL,

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_fog_revealed_areas_map ON fog_revealed_areas(map_id);
