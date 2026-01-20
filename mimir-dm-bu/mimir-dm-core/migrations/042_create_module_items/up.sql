-- Module Items: Catalog references for items/treasure in modules
-- Items are catalog entries referenced by name/source (like monsters)
CREATE TABLE module_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id INTEGER NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    location TEXT,              -- Where found: boss_chamber, hidden_cache, reward, etc.
    name TEXT NOT NULL,         -- Item name as it appears in catalog
    source TEXT NOT NULL,       -- Source book: DMG, PHB, or "campaign" for custom
    quantity INTEGER NOT NULL DEFAULT 1,
    notes TEXT,                 -- DM notes about this item's role in the module
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Index for fast module lookups
CREATE INDEX idx_module_items_module_id ON module_items(module_id);

-- Index for item name searches
CREATE INDEX idx_module_items_name ON module_items(name);
