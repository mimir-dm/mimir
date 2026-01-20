CREATE TABLE catalog_optional_features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    feature_types TEXT, -- JSON array of feature types for filtering: ["EI", "MM"], etc.
    feature_type_full TEXT, -- Formatted display name: "Eldritch Invocation, Metamagic"
    prerequisite_text TEXT, -- Formatted prerequisite text
    grants_spells BOOLEAN DEFAULT FALSE, -- Whether this feature grants additional spells
    source TEXT NOT NULL,
    page INTEGER,
    full_optional_feature_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

-- Index for searching
CREATE INDEX idx_catalog_optional_features_name ON catalog_optional_features (name);
CREATE INDEX idx_catalog_optional_features_feature_types ON catalog_optional_features (feature_types);
CREATE INDEX idx_catalog_optional_features_source ON catalog_optional_features (source);
CREATE INDEX idx_catalog_optional_features_grants_spells ON catalog_optional_features (grants_spells);