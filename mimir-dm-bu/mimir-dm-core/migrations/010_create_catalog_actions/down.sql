-- Drop catalog_actions table and its indexes
DROP INDEX IF EXISTS idx_catalog_actions_time_type;
DROP INDEX IF EXISTS idx_catalog_actions_source;
DROP INDEX IF EXISTS idx_catalog_actions_name;
DROP TABLE IF EXISTS catalog_actions;