-- Drop light sources table and indexes
DROP INDEX IF EXISTS idx_light_sources_token_id;
DROP INDEX IF EXISTS idx_light_sources_map_id;
DROP TABLE IF EXISTS light_sources;

-- Note: SQLite doesn't support DROP COLUMN
-- vision_type, vision_range_ft on tokens will be ignored if not used
-- ambient_light on maps will be ignored if not used
