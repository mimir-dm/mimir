-- Migration 001: Catalog Sources Table
-- Tracks imported source books and their enabled status

CREATE TABLE catalog_sources (
    code TEXT PRIMARY KEY NOT NULL,      -- Source code (e.g., "PHB", "MM", "XGE")
    name TEXT NOT NULL,                   -- Display name (e.g., "Player's Handbook")
    enabled INTEGER NOT NULL DEFAULT 1,   -- SQLite boolean: 1 = enabled, 0 = disabled
    imported_at TEXT NOT NULL             -- ISO 8601 timestamp of import
);

-- Index for filtering by enabled status
CREATE INDEX idx_catalog_sources_enabled ON catalog_sources(enabled);
