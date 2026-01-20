-- Create documents table to track campaign documents
CREATE TABLE documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    campaign_id INTEGER NOT NULL,
    module_id INTEGER NULL,  -- Optional: only for module-specific documents
    session_id INTEGER NULL, -- Optional: only for session-specific documents
    template_id TEXT NOT NULL,
    document_type TEXT NOT NULL,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT NULL,
    
    FOREIGN KEY (campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE,
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- Create index for quick lookups
CREATE INDEX idx_documents_campaign ON documents(campaign_id);
CREATE INDEX idx_documents_module ON documents(module_id);
CREATE INDEX idx_documents_session ON documents(session_id);