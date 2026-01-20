-- Drop is_srd columns and related indexes from catalog tables

-- Drop indexes first (if they exist)
DROP INDEX IF EXISTS idx_catalog_backgrounds_is_srd;
DROP INDEX IF EXISTS idx_catalog_feats_is_srd;
DROP INDEX IF EXISTS idx_catalog_objects_srd;

-- Drop is_srd columns from all catalog tables
ALTER TABLE catalog_conditions DROP COLUMN is_srd;
ALTER TABLE catalog_languages DROP COLUMN is_srd;
ALTER TABLE catalog_rewards DROP COLUMN is_srd;
ALTER TABLE catalog_backgrounds DROP COLUMN is_srd;
ALTER TABLE catalog_feats DROP COLUMN is_srd;
ALTER TABLE catalog_objects DROP COLUMN is_srd;