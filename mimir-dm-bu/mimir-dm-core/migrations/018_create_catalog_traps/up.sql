-- Create catalog_traps table for database-backed trap and hazard storage
CREATE TABLE catalog_traps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    category TEXT NOT NULL, -- "Trap" or "Hazard"
    trap_type TEXT, -- Formatted trap type (Mechanical, Magical, etc.)
    source TEXT NOT NULL,
    full_trap_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

-- Indexes for better search performance
CREATE INDEX idx_catalog_traps_name ON catalog_traps(name);
CREATE INDEX idx_catalog_traps_category ON catalog_traps(category);
CREATE INDEX idx_catalog_traps_trap_type ON catalog_traps(trap_type);
CREATE INDEX idx_catalog_traps_source ON catalog_traps(source);