-- Migration 002: Monsters Table
-- Stores monsters with indexed columns for filtering and full JSON data

CREATE TABLE monsters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    cr TEXT,                              -- Challenge rating as string (e.g., "1/4", "1", "10")
    creature_type TEXT,                   -- Extracted from data.type.type
    size TEXT,                            -- Extracted from data.size[0]
    token_image_path TEXT,                -- Path to token image file (nullable)
    data TEXT NOT NULL,                   -- Full 5etools JSON blob
    UNIQUE(name, source)
);

-- Indexes for common query patterns
CREATE INDEX idx_monsters_name ON monsters(name);
CREATE INDEX idx_monsters_source ON monsters(source);
CREATE INDEX idx_monsters_cr ON monsters(cr);
CREATE INDEX idx_monsters_creature_type ON monsters(creature_type);
CREATE INDEX idx_monsters_size ON monsters(size);
