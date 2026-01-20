-- Drop catalog_deities table and indexes
DROP INDEX IF EXISTS idx_catalog_deities_alignment;
DROP INDEX IF EXISTS idx_catalog_deities_pantheon;
DROP INDEX IF EXISTS idx_catalog_deities_source;
DROP INDEX IF EXISTS idx_catalog_deities_name;
DROP TABLE IF EXISTS catalog_deities;