-- Drop character_sources table
DROP INDEX IF EXISTS idx_character_sources_character;
DROP TABLE IF EXISTS character_sources;

-- Drop characters table
DROP INDEX IF EXISTS idx_characters_is_npc;
DROP INDEX IF EXISTS idx_characters_campaign;
DROP TABLE IF EXISTS characters;
