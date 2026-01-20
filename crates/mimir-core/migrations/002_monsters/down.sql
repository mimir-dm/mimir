-- Migration 002 rollback: Remove monsters table

DROP INDEX IF EXISTS idx_monsters_size;
DROP INDEX IF EXISTS idx_monsters_creature_type;
DROP INDEX IF EXISTS idx_monsters_cr;
DROP INDEX IF EXISTS idx_monsters_source;
DROP INDEX IF EXISTS idx_monsters_name;
DROP TABLE IF EXISTS monsters;
