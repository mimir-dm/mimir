-- Migration 001 rollback: Remove catalog_sources table

DROP INDEX IF EXISTS idx_catalog_sources_enabled;
DROP TABLE IF EXISTS catalog_sources;
