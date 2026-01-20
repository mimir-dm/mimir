CREATE TABLE catalog_tables (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    caption TEXT,
    category TEXT NOT NULL,
    source TEXT NOT NULL,
    page INTEGER,
    columns_count INTEGER NOT NULL DEFAULT 0,
    rows_count INTEGER NOT NULL DEFAULT 0,
    full_table_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

-- Index for searching
CREATE INDEX idx_catalog_tables_name ON catalog_tables (name);
CREATE INDEX idx_catalog_tables_category ON catalog_tables (category);
CREATE INDEX idx_catalog_tables_source ON catalog_tables (source);
