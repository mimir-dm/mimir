CREATE TABLE catalog_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    item_type TEXT, -- M, R, A, LA, MA, HA, etc.
    type_name TEXT, -- "Melee Weapon", "Light Armor", etc.
    rarity TEXT, -- "common", "uncommon", "rare", etc.
    value REAL, -- Value in copper pieces (can be fractional)
    weight REAL, -- Weight in pounds
    ac INTEGER, -- Armor class
    damage TEXT, -- Combined damage string
    requires_attunement TEXT, -- Attunement requirements
    source TEXT NOT NULL,
    page INTEGER,
    full_item_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

-- Indexes for searching and filtering
CREATE INDEX idx_catalog_items_name ON catalog_items (name);
CREATE INDEX idx_catalog_items_item_type ON catalog_items (item_type);
CREATE INDEX idx_catalog_items_rarity ON catalog_items (rarity);
CREATE INDEX idx_catalog_items_value ON catalog_items (value);
CREATE INDEX idx_catalog_items_source ON catalog_items (source);