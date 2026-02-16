-- SQLite doesn't support DROP COLUMN before 3.35.0, so we recreate the table
CREATE TABLE documents_backup AS SELECT id, campaign_id, module_id, title, content, doc_type, created_at, updated_at FROM documents;
DROP TABLE documents;
CREATE TABLE documents (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    module_id TEXT REFERENCES modules(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    doc_type TEXT NOT NULL DEFAULT 'note',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO documents SELECT * FROM documents_backup;
DROP TABLE documents_backup;
CREATE INDEX idx_documents_campaign ON documents(campaign_id);
CREATE INDEX idx_documents_module ON documents(module_id);
CREATE INDEX idx_documents_type ON documents(doc_type);
