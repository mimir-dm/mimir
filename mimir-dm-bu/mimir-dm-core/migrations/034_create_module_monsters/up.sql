-- Create module_monsters table for associating monsters with modules
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

-- Index for efficient lookups by module
CREATE INDEX idx_module_monsters_module_id ON module_monsters(module_id);

-- Index for looking up by encounter tag within a module
CREATE INDEX idx_module_monsters_encounter ON module_monsters(module_id, encounter_tag);
