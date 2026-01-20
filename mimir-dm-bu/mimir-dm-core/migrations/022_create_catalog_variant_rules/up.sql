CREATE TABLE catalog_variant_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    rule_type TEXT, -- "V", "O", "Action Options", etc.
    source TEXT NOT NULL,
    page INTEGER,
    full_variant_rule_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

-- Index for searching
CREATE INDEX idx_catalog_variant_rules_name ON catalog_variant_rules (name);
CREATE INDEX idx_catalog_variant_rules_type ON catalog_variant_rules (rule_type);
CREATE INDEX idx_catalog_variant_rules_source ON catalog_variant_rules (source);