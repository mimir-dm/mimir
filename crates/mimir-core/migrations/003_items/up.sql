-- Migration 003: Items Table
-- Stores items (weapons, armor, magic items, mundane equipment) with indexed columns

CREATE TABLE items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    item_type TEXT,                       -- Item type code (e.g., "R" for rod, "A" for armor)
    rarity TEXT,                          -- Rarity (common, uncommon, rare, very rare, legendary)
    data TEXT NOT NULL,                   -- Full 5etools JSON blob
    fluff TEXT,                           -- Lore/flavor text and image paths from fluff files
    UNIQUE(name, source)
);

-- Indexes for common query patterns
CREATE INDEX idx_items_name ON items(name);
CREATE INDEX idx_items_source ON items(source);
CREATE INDEX idx_items_type ON items(item_type);
CREATE INDEX idx_items_rarity ON items(rarity);
