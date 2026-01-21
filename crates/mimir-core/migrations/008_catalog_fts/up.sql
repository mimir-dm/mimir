-- Migration 008: Unified Full-Text Search Table
-- FTS5 virtual table for searching across all catalog entities

-- Unified full-text search table
-- entity_id is marked UNINDEXED since we only use it for lookups, not searching
CREATE VIRTUAL TABLE catalog_fts USING fts5(
    entity_type,    -- 'monster', 'spell', 'item', etc.
    entity_id UNINDEXED,  -- References the entity's primary key
    content_type,   -- 'rules' | 'fluff'
    name,           -- Entity name (always indexed)
    text_content,   -- Flattened entries text
    tokenize='porter unicode61'
);
