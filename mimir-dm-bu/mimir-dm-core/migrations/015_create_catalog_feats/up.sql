-- Create catalog_feats table for storing D&D 5e feat data
CREATE TABLE catalog_feats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- Core searchable fields
    name TEXT NOT NULL,
    prerequisites TEXT,         -- Formatted prerequisites string, NULL if none
    brief TEXT,                 -- Brief description or first entry excerpt
    is_srd INTEGER NOT NULL DEFAULT 0,
    source TEXT NOT NULL,       -- Book ID: "PHB", "DMG", etc.
    
    -- Full feat data for modal display
    full_feat_json TEXT NOT NULL,
    
    -- Timestamp
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    -- Unique constraint to prevent duplicates
    UNIQUE(name, source)
);

-- Indexes for performance
CREATE INDEX idx_catalog_feats_name ON catalog_feats(name);
CREATE INDEX idx_catalog_feats_prerequisites ON catalog_feats(prerequisites);
CREATE INDEX idx_catalog_feats_source ON catalog_feats(source);
CREATE INDEX idx_catalog_feats_is_srd ON catalog_feats(is_srd);
CREATE INDEX idx_catalog_feats_search ON catalog_feats(name, source);