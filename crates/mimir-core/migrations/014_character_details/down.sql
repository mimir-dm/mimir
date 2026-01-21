-- Drop character detail tables
DROP INDEX IF EXISTS idx_character_spells_class;
DROP INDEX IF EXISTS idx_character_spells_character;
DROP TABLE IF EXISTS character_spells;

DROP INDEX IF EXISTS idx_character_proficiencies_type;
DROP INDEX IF EXISTS idx_character_proficiencies_character;
DROP TABLE IF EXISTS character_proficiencies;

DROP INDEX IF EXISTS idx_character_inventory_character;
DROP TABLE IF EXISTS character_inventory;
