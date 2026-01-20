-- Rollback Migration 046: DB-Only Document Storage
-- Note: This will lose any content stored in the documents.content column

-- Recreate character_versions table with file_path column
CREATE TABLE character_versions_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL,
    version_number INTEGER NOT NULL,
    file_path TEXT NOT NULL DEFAULT '',
    character_data TEXT NOT NULL,
    snapshot_reason TEXT,
    level INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE,
    UNIQUE(character_id, version_number)
);

-- Copy data back (file_path will be empty string since we don't have the original)
INSERT INTO character_versions_new (id, character_id, version_number, file_path, character_data, snapshot_reason, level, created_at)
SELECT id, character_id, version_number, '', character_data, snapshot_reason, level, created_at
FROM character_versions;

-- Drop table and rename
DROP TABLE character_versions;
ALTER TABLE character_versions_new RENAME TO character_versions;

-- Recreate indexes
CREATE INDEX idx_character_versions_character ON character_versions(character_id);
CREATE INDEX idx_character_versions_level ON character_versions(level);

-- Remove content column from documents
-- SQLite doesn't support DROP COLUMN easily, so we recreate the table
CREATE TABLE documents_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    campaign_id INTEGER NOT NULL,
    module_id INTEGER NULL,
    session_id INTEGER NULL,
    template_id TEXT NOT NULL,
    document_type TEXT NOT NULL,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT NULL,
    file_type TEXT NOT NULL DEFAULT 'markdown',
    is_user_created INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE,
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

INSERT INTO documents_new (id, campaign_id, module_id, session_id, template_id, document_type, title, file_path, created_at, updated_at, completed_at, file_type, is_user_created)
SELECT id, campaign_id, module_id, session_id, template_id, document_type, title, file_path, created_at, updated_at, completed_at, file_type, is_user_created
FROM documents;

DROP TABLE documents;
ALTER TABLE documents_new RENAME TO documents;

-- Recreate indexes for documents
CREATE INDEX idx_documents_campaign ON documents(campaign_id);
CREATE INDEX idx_documents_module ON documents(module_id);
CREATE INDEX idx_documents_session ON documents(session_id);
CREATE INDEX idx_documents_user_created ON documents(is_user_created);
CREATE INDEX idx_documents_file_type ON documents(file_type);
