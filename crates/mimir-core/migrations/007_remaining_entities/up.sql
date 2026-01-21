-- Migration 007: Remaining Catalog Entity Tables
-- Conditions, diseases, senses, skills, actions, languages, vehicles, objects, traps, hazards, cults, deities

-- Game conditions (blinded, charmed, frightened, etc.)
CREATE TABLE conditions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    fluff TEXT,                          -- Lore/flavor text and image paths
    UNIQUE(name, source)
);
CREATE INDEX idx_conditions_name ON conditions(name);
CREATE INDEX idx_conditions_source ON conditions(source);

-- Diseases
CREATE TABLE diseases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    fluff TEXT,                          -- Lore/flavor text and image paths
    UNIQUE(name, source)
);
CREATE INDEX idx_diseases_name ON diseases(name);
CREATE INDEX idx_diseases_source ON diseases(source);

-- Senses (darkvision, tremorsense, blindsight, etc.)
CREATE TABLE senses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_senses_name ON senses(name);
CREATE INDEX idx_senses_source ON senses(source);

-- Skills (Athletics, Perception, Stealth, etc.)
CREATE TABLE skills (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    ability TEXT,                        -- Associated ability (STR, DEX, etc.)
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_skills_name ON skills(name);
CREATE INDEX idx_skills_source ON skills(source);

-- Actions (Dash, Dodge, Help, Hide, etc.)
CREATE TABLE actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_actions_name ON actions(name);
CREATE INDEX idx_actions_source ON actions(source);

-- Languages (Common, Elvish, Dwarvish, etc.)
CREATE TABLE languages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    language_type TEXT,                  -- Standard, Exotic, Secret
    data TEXT NOT NULL,
    fluff TEXT,                          -- Lore/flavor text and image paths
    UNIQUE(name, source)
);
CREATE INDEX idx_languages_name ON languages(name);
CREATE INDEX idx_languages_source ON languages(source);
CREATE INDEX idx_languages_type ON languages(language_type);

-- Vehicles (ships, airships, infernal war machines, etc.)
CREATE TABLE vehicles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    vehicle_type TEXT,                   -- ship, infernal, creature, object, spelljammer
    data TEXT NOT NULL,
    fluff TEXT,                          -- Lore/flavor text and image paths
    UNIQUE(name, source)
);
CREATE INDEX idx_vehicles_name ON vehicles(name);
CREATE INDEX idx_vehicles_source ON vehicles(source);
CREATE INDEX idx_vehicles_type ON vehicles(vehicle_type);

-- Objects (doors, chests, siege weapons, etc.)
CREATE TABLE objects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    object_type TEXT,                    -- Generic, Siege Weapon, etc.
    data TEXT NOT NULL,
    fluff TEXT,                          -- Lore/flavor text and image paths
    UNIQUE(name, source)
);
CREATE INDEX idx_objects_name ON objects(name);
CREATE INDEX idx_objects_source ON objects(source);

-- Traps
CREATE TABLE traps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    trap_tier TEXT,                      -- Simple, Complex
    data TEXT NOT NULL,
    fluff TEXT,                          -- Lore/flavor text and image paths
    UNIQUE(name, source)
);
CREATE INDEX idx_traps_name ON traps(name);
CREATE INDEX idx_traps_source ON traps(source);

-- Hazards (environmental dangers)
CREATE TABLE hazards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    fluff TEXT,                          -- Lore/flavor text and image paths
    UNIQUE(name, source)
);
CREATE INDEX idx_hazards_name ON hazards(name);
CREATE INDEX idx_hazards_source ON hazards(source);

-- Cults and supernatural gifts/boons
CREATE TABLE cults (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_cults_name ON cults(name);
CREATE INDEX idx_cults_source ON cults(source);

-- Deities (gods by pantheon)
CREATE TABLE deities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    pantheon TEXT,                       -- Greek, Norse, Forgotten Realms, etc.
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_deities_name ON deities(name);
CREATE INDEX idx_deities_source ON deities(source);
CREATE INDEX idx_deities_pantheon ON deities(pantheon);
