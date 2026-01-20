-- Create catalog_races table for storing D&D 5e race data
CREATE TABLE catalog_races (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- Core searchable fields matching RaceTable columns exactly
    name TEXT NOT NULL,
    size TEXT,                  -- Size category (centered display)
    speed INTEGER,              -- Walking speed in feet (will display with "ft." suffix, centered)
    ability_bonuses TEXT,       -- Formatted ability score bonuses
    traits_count INTEGER NOT NULL DEFAULT 0,  -- Count of traits (centered display)
    source TEXT NOT NULL,       -- Source book
    
    -- Full race data for modal display
    full_race_json TEXT NOT NULL,
    
    -- Timestamp
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    -- Unique constraint to prevent duplicates
    UNIQUE(name, source)
);

-- Indexes for performance
CREATE INDEX idx_catalog_races_name ON catalog_races(name);
CREATE INDEX idx_catalog_races_size ON catalog_races(size);
CREATE INDEX idx_catalog_races_speed ON catalog_races(speed);
CREATE INDEX idx_catalog_races_source ON catalog_races(source);
CREATE INDEX idx_catalog_races_search ON catalog_races(name, source);