CREATE TABLE catalog_conditions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    item_type TEXT NOT NULL,
    description TEXT NOT NULL,
    is_srd INTEGER NOT NULL,
    source TEXT NOT NULL,
    full_condition_json TEXT NOT NULL,
    UNIQUE(name, source)
);