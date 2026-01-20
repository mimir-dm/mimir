-- Create fog of war revealed areas table
-- Tracks rectangular regions that have been revealed on a map
CREATE TABLE fog_revealed_areas (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    map_id INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    -- Rectangle bounds in pixel coordinates
    x REAL NOT NULL,
    y REAL NOT NULL,
    width REAL NOT NULL,
    height REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for fast lookup by map
CREATE INDEX idx_fog_revealed_areas_map_id ON fog_revealed_areas(map_id);

-- Add fog_enabled flag to maps table
ALTER TABLE maps ADD COLUMN fog_enabled INTEGER NOT NULL DEFAULT 0;
