-- Migration 005: Join Tables
-- Many-to-many relationships between spells/items and classes/subclasses

-- Spells available to each class
CREATE TABLE spell_classes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    spell_id INTEGER NOT NULL REFERENCES spells(id) ON DELETE CASCADE,
    class_name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    UNIQUE(spell_id, class_name, source)
);
CREATE INDEX idx_spell_classes_spell ON spell_classes(spell_id);
CREATE INDEX idx_spell_classes_class ON spell_classes(class_name);
CREATE INDEX idx_spell_classes_source ON spell_classes(source);

-- Subclass-specific spell lists (e.g., Arcane Trickster, Eldritch Knight)
CREATE TABLE spell_subclasses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    spell_id INTEGER NOT NULL REFERENCES spells(id) ON DELETE CASCADE,
    subclass_name TEXT NOT NULL,
    class_name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    UNIQUE(spell_id, subclass_name, class_name, source)
);
CREATE INDEX idx_spell_subclasses_spell ON spell_subclasses(spell_id);
CREATE INDEX idx_spell_subclasses_subclass ON spell_subclasses(subclass_name, class_name);
CREATE INDEX idx_spell_subclasses_source ON spell_subclasses(source);

-- Items requiring attunement by specific class
CREATE TABLE item_attunement_classes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    class_name TEXT NOT NULL,
    UNIQUE(item_id, class_name)
);
CREATE INDEX idx_item_attunement_item ON item_attunement_classes(item_id);
CREATE INDEX idx_item_attunement_class ON item_attunement_classes(class_name);
