-- Add fluff_json column to store class fluff data
ALTER TABLE catalog_classes ADD COLUMN fluff_json TEXT;

-- Add fluff_json column to store subclass fluff data  
ALTER TABLE catalog_subclasses ADD COLUMN fluff_json TEXT;