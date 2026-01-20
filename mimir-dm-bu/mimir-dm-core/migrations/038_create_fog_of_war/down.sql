-- Drop fog of war tables
DROP INDEX IF EXISTS idx_fog_revealed_areas_map_id;
DROP TABLE IF EXISTS fog_revealed_areas;

-- Note: SQLite doesn't support DROP COLUMN, so fog_enabled stays
-- It will be ignored if not used
