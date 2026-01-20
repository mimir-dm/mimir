-- Remove fluff_json column from catalog_classes table
ALTER TABLE catalog_classes DROP COLUMN fluff_json;

-- Remove fluff_json column from catalog_subclasses table
ALTER TABLE catalog_subclasses DROP COLUMN fluff_json;