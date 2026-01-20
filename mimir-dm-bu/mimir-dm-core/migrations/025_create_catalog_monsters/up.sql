-- Create catalog monsters table
CREATE TABLE catalog_monsters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    size TEXT,                      -- Single size for search (S, M, L, etc.)
    creature_type TEXT,             -- Simplified creature type (string only)
    alignment TEXT,                 -- Simplified alignment (first one)
    cr TEXT,                        -- Challenge rating as string (1/4, 1/2, 1, 2, etc.)
    cr_numeric REAL,                -- Challenge rating as numeric for sorting (0.25, 0.5, 1.0, etc.)
    hp INTEGER,                     -- Hit points (average value)
    ac INTEGER,                     -- Armor class (single value)
    source TEXT NOT NULL,
    page INTEGER,
    full_monster_json TEXT NOT NULL,  -- Complete JSON for detailed view
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

-- Create indexes for common queries
CREATE INDEX idx_monsters_size ON catalog_monsters(size);
CREATE INDEX idx_monsters_type ON catalog_monsters(creature_type);  
CREATE INDEX idx_monsters_cr_numeric ON catalog_monsters(cr_numeric);
CREATE INDEX idx_monsters_source ON catalog_monsters(source);
CREATE INDEX idx_monsters_name ON catalog_monsters(name);