-- SQLite doesn't support DROP COLUMN directly, so we need to recreate the table
-- This creates a new table without the customization columns

CREATE TABLE module_monsters_backup AS SELECT
    id, module_id, monster_name, monster_source, quantity, encounter_tag, created_at, updated_at
FROM module_monsters;

DROP TABLE module_monsters;

CREATE TABLE module_monsters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id INTEGER NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    monster_name TEXT NOT NULL,
    monster_source TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    encounter_tag TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO module_monsters (id, module_id, monster_name, monster_source, quantity, encounter_tag, created_at, updated_at)
SELECT id, module_id, monster_name, monster_source, quantity, encounter_tag, created_at, updated_at
FROM module_monsters_backup;

DROP TABLE module_monsters_backup;

CREATE INDEX idx_module_monsters_module_id ON module_monsters(module_id);
CREATE INDEX idx_module_monsters_encounter ON module_monsters(module_id, encounter_tag);
