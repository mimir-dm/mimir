-- Create catalog classes table
CREATE TABLE catalog_classes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    hit_dice TEXT,                    -- Formatted as "1d8", "1d10", etc.
    primary_ability TEXT,              -- Computed: Strength, Dexterity, etc.
    proficiency TEXT,                  -- Saving throws formatted: "STR, CON"
    spellcasting_ability TEXT,         -- "int", "wis", "cha" 
    subclass_title TEXT,               -- "Archetype", "Domain", etc.
    caster_progression TEXT,           -- "full", "1/2", "1/3", "pact"
    source TEXT NOT NULL,
    page INTEGER,
    full_class_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

-- Create catalog subclasses table
CREATE TABLE catalog_subclasses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    short_name TEXT,
    class_name TEXT NOT NULL,
    class_source TEXT NOT NULL,
    source TEXT NOT NULL,
    page INTEGER,
    caster_progression TEXT,           -- Subclass-specific casting progression
    spellcasting_ability TEXT,         -- Subclass-specific spellcasting ability
    full_subclass_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, class_name, source)
);

-- Create catalog class features table
CREATE TABLE catalog_class_features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    class_name TEXT NOT NULL,
    class_source TEXT NOT NULL,
    level INTEGER NOT NULL,
    source TEXT NOT NULL,
    page INTEGER,
    full_feature_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create catalog subclass features table
CREATE TABLE catalog_subclass_features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    class_name TEXT NOT NULL,
    class_source TEXT NOT NULL,
    subclass_short_name TEXT,
    subclass_source TEXT NOT NULL,
    level INTEGER NOT NULL,
    source TEXT NOT NULL,
    page INTEGER,
    full_feature_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for common search patterns
CREATE INDEX idx_catalog_classes_name ON catalog_classes(name);
CREATE INDEX idx_catalog_classes_source ON catalog_classes(source);
CREATE INDEX idx_catalog_classes_spellcasting ON catalog_classes(spellcasting_ability);
CREATE INDEX idx_catalog_classes_primary_ability ON catalog_classes(primary_ability);

CREATE INDEX idx_catalog_subclasses_class ON catalog_subclasses(class_name, class_source);
CREATE INDEX idx_catalog_subclasses_source ON catalog_subclasses(source);
CREATE INDEX idx_catalog_subclasses_name ON catalog_subclasses(name);

CREATE INDEX idx_catalog_class_features_class ON catalog_class_features(class_name, class_source);
CREATE INDEX idx_catalog_class_features_level ON catalog_class_features(level);
CREATE INDEX idx_catalog_class_features_source ON catalog_class_features(source);

CREATE INDEX idx_catalog_subclass_features_class ON catalog_subclass_features(class_name, class_source);
CREATE INDEX idx_catalog_subclass_features_subclass ON catalog_subclass_features(subclass_short_name, subclass_source);
CREATE INDEX idx_catalog_subclass_features_level ON catalog_subclass_features(level);
CREATE INDEX idx_catalog_subclass_features_source ON catalog_subclass_features(source);