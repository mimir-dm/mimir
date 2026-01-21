-- CharacterInventory, CharacterProficiency, and CharacterSpell tables
-- Track character items, proficiencies, and known spells

-- Character inventory (items held by character)
CREATE TABLE character_inventory (
    id TEXT PRIMARY KEY NOT NULL,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    item_name TEXT NOT NULL,
    item_source TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    equipped INTEGER NOT NULL DEFAULT 0,
    attuned INTEGER NOT NULL DEFAULT 0,
    notes TEXT
);

CREATE INDEX idx_character_inventory_character ON character_inventory(character_id);

-- Character proficiencies (skills, saves, tools, weapons, armor, languages)
CREATE TABLE character_proficiencies (
    id TEXT PRIMARY KEY NOT NULL,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    proficiency_type TEXT NOT NULL,  -- 'skill', 'save', 'tool', 'weapon', 'armor', 'language'
    name TEXT NOT NULL,
    expertise INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_character_proficiencies_character ON character_proficiencies(character_id);
CREATE INDEX idx_character_proficiencies_type ON character_proficiencies(proficiency_type);

-- Character spells (known/prepared spells)
CREATE TABLE character_spells (
    id TEXT PRIMARY KEY NOT NULL,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    spell_name TEXT NOT NULL,
    spell_source TEXT NOT NULL,
    source_class TEXT NOT NULL,  -- which class grants this spell
    prepared INTEGER NOT NULL DEFAULT 0  -- whether spell is prepared/chosen
);

CREATE INDEX idx_character_spells_character ON character_spells(character_id);
CREATE INDEX idx_character_spells_class ON character_spells(source_class);
