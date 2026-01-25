-- Map Points of Interest (POIs)
-- Markers for locations, secrets, notes, etc. on maps

CREATE TABLE map_pois (
    id TEXT PRIMARY KEY NOT NULL,
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    grid_x INTEGER NOT NULL,
    grid_y INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT NOT NULL DEFAULT 'pin',  -- pin, star, skull, chest, door, secret, question, exclamation
    color TEXT,  -- hex color for the marker
    visible INTEGER NOT NULL DEFAULT 0,  -- 0 = hidden from players, 1 = visible
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_map_pois_map_id ON map_pois(map_id);
