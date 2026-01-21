-- Migration 003 rollback: Remove items table

DROP INDEX IF EXISTS idx_items_rarity;
DROP INDEX IF EXISTS idx_items_type;
DROP INDEX IF EXISTS idx_items_source;
DROP INDEX IF EXISTS idx_items_name;
DROP TABLE IF EXISTS items;
