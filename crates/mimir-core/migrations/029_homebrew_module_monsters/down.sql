-- Revert homebrew_monster_id from module_monsters
-- Remove homebrew monster references and restore NOT NULL on monster_name/monster_source

PRAGMA foreign_keys = OFF;

-- 1. Create original table structure
CREATE TABLE module_monsters_old (
    id TEXT PRIMARY KEY NOT NULL,
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    monster_name TEXT NOT NULL,
    monster_source TEXT NOT NULL,
    display_name TEXT,
    notes TEXT,
    quantity INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 2. Copy only catalog monster rows (homebrew rows have no catalog reference)
INSERT INTO module_monsters_old (id, module_id, monster_name, monster_source, display_name, notes, quantity, created_at, updated_at)
SELECT id, module_id, monster_name, monster_source, display_name, notes, quantity, created_at, updated_at
FROM module_monsters
WHERE monster_name IS NOT NULL;

-- 3. Drop new table
DROP TABLE module_monsters;

-- 4. Rename back
ALTER TABLE module_monsters_old RENAME TO module_monsters;

-- 5. Recreate indexes
CREATE INDEX idx_module_monsters_module ON module_monsters(module_id);

PRAGMA foreign_keys = ON;
