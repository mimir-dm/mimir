-- Drop document FTS and triggers
DROP TRIGGER IF EXISTS documents_fts_update;
DROP TRIGGER IF EXISTS documents_fts_delete;
DROP TRIGGER IF EXISTS documents_fts_insert;
DROP TABLE IF EXISTS documents_fts;

-- Drop documents table
DROP INDEX IF EXISTS idx_documents_type;
DROP INDEX IF EXISTS idx_documents_module;
DROP INDEX IF EXISTS idx_documents_campaign;
DROP TABLE IF EXISTS documents;
