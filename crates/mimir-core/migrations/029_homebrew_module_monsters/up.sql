-- Add homebrew_monster_id to module_monsters
-- SQLite doesn't support ALTER COLUMN to make columns nullable, so we rebuild the table.

PRAGMA foreign_keys = OFF;

-- 1. Create new table with homebrew support
CREATE TABLE module_monsters_new (
    id TEXT PRIMARY KEY NOT NULL,
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,

    -- Reference to catalog (name + source for lookup) — nullable for homebrew monsters
    monster_name TEXT,
    monster_source TEXT,

    -- Reference to homebrew monster — alternative to catalog reference
    homebrew_monster_id TEXT REFERENCES campaign_homebrew_monsters(id) ON DELETE CASCADE,

    -- Customizations (NULL = use catalog/homebrew value)
    display_name TEXT,
    notes TEXT,

    -- Quantity for encounters
    quantity INTEGER NOT NULL DEFAULT 1,

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    -- Exactly one of catalog (monster_name) or homebrew (homebrew_monster_id) must be set
    CHECK (
        (monster_name IS NOT NULL AND homebrew_monster_id IS NULL) OR
        (monster_name IS NULL AND homebrew_monster_id IS NOT NULL)
    )
);

-- 2. Copy existing data (all existing rows are catalog monsters)
INSERT INTO module_monsters_new (id, module_id, monster_name, monster_source, display_name, notes, quantity, created_at, updated_at)
SELECT id, module_id, monster_name, monster_source, display_name, notes, quantity, created_at, updated_at
FROM module_monsters;

-- 3. Drop old table
DROP TABLE module_monsters;

-- 4. Rename new table
ALTER TABLE module_monsters_new RENAME TO module_monsters;

-- 5. Recreate indexes
CREATE INDEX idx_module_monsters_module ON module_monsters(module_id);
CREATE INDEX idx_module_monsters_homebrew ON module_monsters(homebrew_monster_id);

PRAGMA foreign_keys = ON;
