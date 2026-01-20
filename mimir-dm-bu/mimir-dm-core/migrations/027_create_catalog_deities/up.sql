-- Create catalog_deities table for database-backed deity catalog
CREATE TABLE catalog_deities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    title TEXT,
    pantheon TEXT,
    alignment TEXT,
    domains TEXT, -- JSON array of domains as comma-separated string for filtering
    symbol TEXT,
    source TEXT NOT NULL,
    page INTEGER,
    full_deity_json TEXT NOT NULL, -- Complete deity data
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

-- Indexes for common queries
CREATE INDEX idx_catalog_deities_name ON catalog_deities(name);
CREATE INDEX idx_catalog_deities_source ON catalog_deities(source);
CREATE INDEX idx_catalog_deities_pantheon ON catalog_deities(pantheon);
CREATE INDEX idx_catalog_deities_alignment ON catalog_deities(alignment);