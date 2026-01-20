-- Create catalog_spells table for storing D&D 5e spell data
CREATE TABLE catalog_spells (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- Core searchable fields
    name TEXT NOT NULL,
    level INTEGER NOT NULL,
    school TEXT NOT NULL,        -- Full name: "Abjuration", not "A"
    cast_time TEXT NOT NULL,     -- Formatted: "1 action", "1 bonus action", etc.
    range TEXT NOT NULL,         -- Formatted: "60 feet", "Touch", "Self", etc.
    components TEXT NOT NULL,    -- Simple: "V, S", "V, S, M", etc.
    tags TEXT NOT NULL,          -- JSON array: ["Concentration", "Ritual", "SRD"]
    source TEXT NOT NULL,        -- Book ID: "PHB", "DMG", etc.
    
    -- Full spell data for modal display
    full_spell_json TEXT NOT NULL,
    
    -- Unique constraint to prevent duplicates
    UNIQUE(name, source)
);

-- Indexes for performance
CREATE INDEX idx_catalog_spells_name ON catalog_spells(name);
CREATE INDEX idx_catalog_spells_level ON catalog_spells(level);
CREATE INDEX idx_catalog_spells_school ON catalog_spells(school);
CREATE INDEX idx_catalog_spells_source ON catalog_spells(source);
CREATE INDEX idx_catalog_spells_tags ON catalog_spells(tags);
CREATE INDEX idx_catalog_spells_search ON catalog_spells(name, level, school, source);