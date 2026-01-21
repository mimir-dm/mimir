-- Migration 004 rollback: Remove spells table

DROP INDEX IF EXISTS idx_spells_concentration;
DROP INDEX IF EXISTS idx_spells_ritual;
DROP INDEX IF EXISTS idx_spells_school;
DROP INDEX IF EXISTS idx_spells_level;
DROP INDEX IF EXISTS idx_spells_source;
DROP INDEX IF EXISTS idx_spells_name;
DROP TABLE IF EXISTS spells;
