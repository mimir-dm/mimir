CREATE TABLE catalog_cults (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    category TEXT NOT NULL, -- "cult" or "boon"
    cult_type TEXT, -- Diabolical, Demonic, Elder Evil, etc.
    source TEXT NOT NULL,
    page INTEGER,
    full_cult_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

-- Create indexes for better search performance
CREATE INDEX idx_catalog_cults_category ON catalog_cults(category);
CREATE INDEX idx_catalog_cults_type ON catalog_cults(cult_type);
CREATE INDEX idx_catalog_cults_source ON catalog_cults(source);