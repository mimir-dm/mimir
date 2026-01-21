-- Drop campaign system tables in reverse order (respecting foreign keys)
DROP INDEX IF EXISTS idx_modules_campaign;
DROP TABLE IF EXISTS modules;
DROP INDEX IF EXISTS idx_campaign_sources_campaign;
DROP TABLE IF EXISTS campaign_sources;
DROP TABLE IF EXISTS campaigns;
