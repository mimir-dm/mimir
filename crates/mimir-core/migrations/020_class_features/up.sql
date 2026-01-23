-- Class Features (Spellcasting, Extra Attack, etc.)
-- These are the detailed feature entries with descriptions
CREATE TABLE class_features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    class_name TEXT NOT NULL,
    class_source TEXT NOT NULL,
    level INTEGER NOT NULL,
    data TEXT NOT NULL,                  -- Full 5etools JSON blob with entries
    UNIQUE(name, source, class_name, class_source)
);
CREATE INDEX idx_class_features_name ON class_features(name);
CREATE INDEX idx_class_features_source ON class_features(source);
CREATE INDEX idx_class_features_class ON class_features(class_name, class_source);
CREATE INDEX idx_class_features_level ON class_features(level);

-- Subclass Features (Channel Divinity: Turn Undead, etc.)
CREATE TABLE subclass_features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    class_name TEXT NOT NULL,
    class_source TEXT NOT NULL,
    subclass_name TEXT NOT NULL,
    subclass_source TEXT NOT NULL,
    level INTEGER NOT NULL,
    data TEXT NOT NULL,                  -- Full 5etools JSON blob with entries
    UNIQUE(name, source, class_name, class_source, subclass_name, subclass_source)
);
CREATE INDEX idx_subclass_features_name ON subclass_features(name);
CREATE INDEX idx_subclass_features_source ON subclass_features(source);
CREATE INDEX idx_subclass_features_class ON subclass_features(class_name, class_source);
CREATE INDEX idx_subclass_features_subclass ON subclass_features(subclass_name, subclass_source);
CREATE INDEX idx_subclass_features_level ON subclass_features(level);
