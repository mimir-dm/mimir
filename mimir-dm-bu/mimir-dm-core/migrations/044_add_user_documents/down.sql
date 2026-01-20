-- SQLite doesn't support DROP COLUMN, so this migration is not reversible
-- To rollback, the database would need to be recreated

DROP INDEX IF EXISTS idx_documents_file_type;
DROP INDEX IF EXISTS idx_documents_user_created;

-- Note: Columns file_type and is_user_created cannot be dropped in SQLite
-- The columns will remain but be ignored by older code versions
