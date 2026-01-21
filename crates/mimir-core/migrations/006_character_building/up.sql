-- Migration 006: Character Building Tables
-- Classes, subclasses, races, backgrounds, and feats for character creation

-- Character classes (Wizard, Fighter, etc.)
CREATE TABLE classes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,                  -- Full 5etools JSON blob
    fluff TEXT,                          -- Lore/flavor text and image paths
    UNIQUE(name, source)
);
CREATE INDEX idx_classes_name ON classes(name);
CREATE INDEX idx_classes_source ON classes(source);

-- Subclasses (School of Evocation, Champion, etc.)
CREATE TABLE subclasses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    class_name TEXT NOT NULL,            -- Parent class name
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,                  -- Full 5etools JSON blob
    fluff TEXT,                          -- Lore/flavor text and image paths
    UNIQUE(name, class_name, source)
);
CREATE INDEX idx_subclasses_name ON subclasses(name);
CREATE INDEX idx_subclasses_class ON subclasses(class_name);
CREATE INDEX idx_subclasses_source ON subclasses(source);

-- Races and lineages (Elf, Dwarf, etc.)
CREATE TABLE races (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,                  -- Full 5etools JSON blob
    fluff TEXT,                          -- Lore/flavor text and image paths
    UNIQUE(name, source)
);
CREATE INDEX idx_races_name ON races(name);
CREATE INDEX idx_races_source ON races(source);

-- Character backgrounds (Acolyte, Criminal, etc.)
CREATE TABLE backgrounds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,                  -- Full 5etools JSON blob
    fluff TEXT,                          -- Lore/flavor text and image paths
    UNIQUE(name, source)
);
CREATE INDEX idx_backgrounds_name ON backgrounds(name);
CREATE INDEX idx_backgrounds_source ON backgrounds(source);

-- Character feats (Alert, Sharpshooter, etc.)
CREATE TABLE feats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,                  -- Full 5etools JSON blob
    fluff TEXT,                          -- Lore/flavor text and image paths
    UNIQUE(name, source)
);
CREATE INDEX idx_feats_name ON feats(name);
CREATE INDEX idx_feats_source ON feats(source);
