-- Rollback fog of war support

DROP INDEX IF EXISTS idx_fog_revealed_areas_map;
DROP TABLE IF EXISTS fog_revealed_areas;

-- SQLite doesn't support DROP COLUMN, so we need to recreate the table
-- For now, we'll just leave the column (it's non-destructive)
-- ALTER TABLE maps DROP COLUMN fog_enabled;
