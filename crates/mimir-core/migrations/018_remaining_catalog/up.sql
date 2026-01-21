-- Migration 018: Remaining Catalog Entity Tables
-- Optional features, psionics, rewards, variant rules, tables

-- Optional features (Eldritch Invocations, Metamagic Options, Fighting Styles, etc.)
CREATE TABLE optional_features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    feature_type TEXT,                    -- EI (Eldritch Invocation), MM (Metamagic), etc.
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_optional_features_name ON optional_features(name);
CREATE INDEX idx_optional_features_source ON optional_features(source);
CREATE INDEX idx_optional_features_type ON optional_features(feature_type);

-- Psionics (psionic disciplines and talents)
CREATE TABLE psionics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    psionic_type TEXT,                    -- D (Discipline), T (Talent)
    psionic_order TEXT,                   -- Avatar, Awakened, Immortal, Nomad, Wu Jen
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_psionics_name ON psionics(name);
CREATE INDEX idx_psionics_source ON psionics(source);
CREATE INDEX idx_psionics_type ON psionics(psionic_type);
CREATE INDEX idx_psionics_order ON psionics(psionic_order);

-- Rewards (blessings, boons, charms, supernatural gifts)
CREATE TABLE rewards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    reward_type TEXT,                     -- blessing, boon, charm, gift
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_rewards_name ON rewards(name);
CREATE INDEX idx_rewards_source ON rewards(source);
CREATE INDEX idx_rewards_type ON rewards(reward_type);

-- Variant rules (optional/alternate game rules)
CREATE TABLE variant_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    rule_type TEXT,                       -- O (Optional), V (Variant), etc.
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_variant_rules_name ON variant_rules(name);
CREATE INDEX idx_variant_rules_source ON variant_rules(source);

-- Tables (random tables, encounter tables, etc.)
CREATE TABLE catalog_tables (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_catalog_tables_name ON catalog_tables(name);
CREATE INDEX idx_catalog_tables_source ON catalog_tables(source);
