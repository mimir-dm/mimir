-- Drop catalog_traps table and indexes
DROP INDEX IF EXISTS idx_catalog_traps_source;
DROP INDEX IF EXISTS idx_catalog_traps_trap_type;
DROP INDEX IF EXISTS idx_catalog_traps_category;
DROP INDEX IF EXISTS idx_catalog_traps_name;
DROP TABLE catalog_traps;