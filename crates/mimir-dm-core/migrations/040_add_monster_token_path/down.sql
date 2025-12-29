-- Remove token_image_path column from catalog_monsters
-- SQLite doesn't support DROP COLUMN directly, so we recreate the table
CREATE TABLE catalog_monsters_backup AS SELECT
    id, name, size, creature_type, alignment, cr, cr_numeric,
    hp, ac, source, page, full_monster_json, fluff_json, created_at
FROM catalog_monsters;

DROP TABLE catalog_monsters;

CREATE TABLE catalog_monsters (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    size TEXT,
    creature_type TEXT,
    alignment TEXT,
    cr TEXT,
    cr_numeric REAL,
    hp INTEGER,
    ac INTEGER,
    source TEXT NOT NULL,
    page INTEGER,
    full_monster_json TEXT NOT NULL,
    fluff_json TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

INSERT INTO catalog_monsters SELECT * FROM catalog_monsters_backup;
DROP TABLE catalog_monsters_backup;
