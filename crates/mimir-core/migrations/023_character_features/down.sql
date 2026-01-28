-- Reverse migration: drop character_features table

DROP INDEX IF EXISTS idx_character_features_type;
DROP INDEX IF EXISTS idx_character_features_character;
DROP TABLE IF EXISTS character_features;
