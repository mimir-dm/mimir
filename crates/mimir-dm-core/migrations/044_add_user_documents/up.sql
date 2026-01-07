-- Add columns for user-created documents
-- file_type: 'markdown', 'png', 'jpg', 'webp', 'gif', 'svg'
-- is_user_created: distinguishes user docs from template-generated docs

ALTER TABLE documents ADD COLUMN file_type TEXT NOT NULL DEFAULT 'markdown';
ALTER TABLE documents ADD COLUMN is_user_created INTEGER NOT NULL DEFAULT 0;

-- Index for efficient user document queries
CREATE INDEX idx_documents_user_created ON documents(is_user_created);
CREATE INDEX idx_documents_file_type ON documents(file_type);
