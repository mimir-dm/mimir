-- Re-add is_srd columns and indexes to catalog tables

-- Add is_srd columns back
ALTER TABLE catalog_conditions ADD COLUMN is_srd INTEGER NOT NULL DEFAULT 0;
ALTER TABLE catalog_languages ADD COLUMN is_srd INTEGER NOT NULL DEFAULT 0;
ALTER TABLE catalog_rewards ADD COLUMN is_srd INTEGER NOT NULL DEFAULT 0;
ALTER TABLE catalog_backgrounds ADD COLUMN is_srd INTEGER NOT NULL DEFAULT 0;
ALTER TABLE catalog_feats ADD COLUMN is_srd INTEGER NOT NULL DEFAULT 0;
ALTER TABLE catalog_objects ADD COLUMN is_srd INTEGER NOT NULL DEFAULT 0;

-- Re-create indexes
CREATE INDEX idx_catalog_backgrounds_is_srd ON catalog_backgrounds(is_srd);
CREATE INDEX idx_catalog_feats_is_srd ON catalog_feats(is_srd);
CREATE INDEX idx_catalog_objects_srd ON catalog_objects(is_srd);