-- Create catalog_objects table for storing D&D 5e object data
CREATE TABLE catalog_objects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- Core searchable fields matching ObjectTable columns
    name TEXT NOT NULL,
    object_type TEXT,           -- Formatted type: "Siege Weapon", "Generic", etc.
    size TEXT,                  -- Formatted size: "Large", "Medium/Large", etc.
    ac TEXT,                    -- Formatted AC: number or special text
    hp TEXT,                    -- Formatted HP: number or special text
    is_srd INTEGER NOT NULL DEFAULT 0,
    source TEXT NOT NULL,       -- Source book
    
    -- Full object data for modal display
    full_object_json TEXT NOT NULL,
    
    -- Timestamp
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    -- Unique constraint to prevent duplicates
    UNIQUE(name, source)
);

-- Indexes for performance
CREATE INDEX idx_catalog_objects_name ON catalog_objects(name);
CREATE INDEX idx_catalog_objects_type ON catalog_objects(object_type);
CREATE INDEX idx_catalog_objects_size ON catalog_objects(size);
CREATE INDEX idx_catalog_objects_source ON catalog_objects(source);
CREATE INDEX idx_catalog_objects_srd ON catalog_objects(is_srd);
CREATE INDEX idx_catalog_objects_search ON catalog_objects(name, source);