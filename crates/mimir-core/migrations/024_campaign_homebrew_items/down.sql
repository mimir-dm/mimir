-- Reverse migration: drop campaign_homebrew_items table

DROP INDEX IF EXISTS idx_campaign_homebrew_items_campaign;
DROP TABLE IF EXISTS campaign_homebrew_items;
