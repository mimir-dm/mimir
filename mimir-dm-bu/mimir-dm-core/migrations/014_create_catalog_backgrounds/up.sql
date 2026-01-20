CREATE TABLE catalog_backgrounds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    skills TEXT NOT NULL,
    languages TEXT NOT NULL,
    tools TEXT NOT NULL,
    feature TEXT NOT NULL,
    is_srd INTEGER NOT NULL DEFAULT 0,
    source TEXT NOT NULL,
    full_background_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indices for performance
CREATE INDEX idx_catalog_backgrounds_name ON catalog_backgrounds(name);
CREATE INDEX idx_catalog_backgrounds_source ON catalog_backgrounds(source);
CREATE INDEX idx_catalog_backgrounds_skills ON catalog_backgrounds(skills);
CREATE INDEX idx_catalog_backgrounds_is_srd ON catalog_backgrounds(is_srd);