-- Create maps table for Visual Display System
CREATE TABLE maps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    campaign_id INTEGER NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    module_id INTEGER REFERENCES modules(id) ON DELETE CASCADE,  -- NULL = campaign-level map
    name TEXT NOT NULL,
    image_path TEXT NOT NULL,
    width_px INTEGER NOT NULL,
    height_px INTEGER NOT NULL,
    original_width_px INTEGER,      -- original dimensions before compression
    original_height_px INTEGER,
    grid_type TEXT NOT NULL DEFAULT 'none',  -- 'square', 'hex', 'none'
    grid_size_px INTEGER,           -- pixels per grid cell
    grid_offset_x INTEGER NOT NULL DEFAULT 0,
    grid_offset_y INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    preview_path TEXT               -- thumbnail image path
);

CREATE INDEX idx_maps_campaign ON maps(campaign_id);
CREATE INDEX idx_maps_module ON maps(module_id);
