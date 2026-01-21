-- Migration 004: Spells Table
-- Stores spells with indexed columns for filtering by level, school, ritual, concentration

CREATE TABLE spells (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    level INTEGER NOT NULL,              -- 0 = cantrip, 1-9 = spell level
    school TEXT,                         -- School code (A, C, D, E, V, I, N, T)
    ritual INTEGER NOT NULL DEFAULT 0,   -- Boolean: 1 = ritual spell
    concentration INTEGER NOT NULL DEFAULT 0, -- Boolean: 1 = requires concentration
    data TEXT NOT NULL,                  -- Full 5etools JSON blob
    fluff TEXT,                          -- Lore/flavor text and image paths from fluff files
    UNIQUE(name, source)
);

-- Indexes for common query patterns
CREATE INDEX idx_spells_name ON spells(name);
CREATE INDEX idx_spells_source ON spells(source);
CREATE INDEX idx_spells_level ON spells(level);
CREATE INDEX idx_spells_school ON spells(school);
CREATE INDEX idx_spells_ritual ON spells(ritual);
CREATE INDEX idx_spells_concentration ON spells(concentration);
