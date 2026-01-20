-- Create light sources table for vision and lighting system
-- Tracks light sources placed on maps (torches, lanterns, spells, etc.)
CREATE TABLE light_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    map_id INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    token_id INTEGER REFERENCES tokens(id) ON DELETE CASCADE,  -- Optional: light attached to token
    name TEXT NOT NULL,
    light_type TEXT NOT NULL DEFAULT 'torch',  -- 'torch', 'lantern', 'candle', 'spell', 'custom'
    -- Position in pixel coordinates (ignored if attached to token)
    x REAL NOT NULL DEFAULT 0.0,
    y REAL NOT NULL DEFAULT 0.0,
    -- Radii in feet (converted to pixels based on grid size)
    bright_radius_ft REAL NOT NULL DEFAULT 20.0,
    dim_radius_ft REAL NOT NULL DEFAULT 40.0,
    -- Optional color tint (hex format, e.g., '#ffaa00' for warm torch light)
    color TEXT,
    -- Whether this light is currently active
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for fast lookup by map
CREATE INDEX idx_light_sources_map_id ON light_sources(map_id);

-- Index for finding lights attached to specific tokens
CREATE INDEX idx_light_sources_token_id ON light_sources(token_id);

-- Add vision fields to tokens table
-- vision_type: 'normal', 'darkvision', 'blindsight', 'tremorsense', 'truesight', 'devils_sight'
ALTER TABLE tokens ADD COLUMN vision_type TEXT NOT NULL DEFAULT 'normal';
-- vision_range_ft: range in feet (e.g., 60 for standard darkvision)
ALTER TABLE tokens ADD COLUMN vision_range_ft REAL;

-- Add ambient light level to maps (default lighting condition)
-- 'bright', 'dim', 'darkness'
ALTER TABLE maps ADD COLUMN ambient_light TEXT NOT NULL DEFAULT 'bright';
