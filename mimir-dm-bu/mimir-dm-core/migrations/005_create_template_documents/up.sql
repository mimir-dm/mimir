-- Template documents table with versioning support
CREATE TABLE template_documents (
    document_id TEXT NOT NULL,
    version_number INTEGER NOT NULL DEFAULT 1,
    document_content TEXT NOT NULL,
    content_hash TEXT NOT NULL, -- SHA-256 hash of document_content to detect duplicates
    document_type TEXT, -- Optional: campaign_pitch, session_plan, npc_template, etc.
    document_level TEXT CHECK(document_level IN ('campaign', 'module', 'session', 'handout')), -- Level at which this template operates
    purpose TEXT, -- Brief description of what this template is for and when to use it
    variables_schema TEXT CHECK(json_valid(variables_schema) OR variables_schema IS NULL), -- JSON schema of available variables
    default_values TEXT CHECK(json_valid(default_values) OR default_values IS NULL), -- JSON object with default values for variables
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE, -- Soft delete support
    metadata TEXT CHECK(json_valid(metadata) OR metadata IS NULL), -- Optional JSON metadata
    PRIMARY KEY (document_id, version_number)
);

-- Index for efficient latest version queries
CREATE INDEX idx_template_documents_id_version ON template_documents(document_id, version_number DESC);

-- Index for document type filtering
CREATE INDEX idx_template_documents_type ON template_documents(document_type);

-- Index for document level filtering
CREATE INDEX idx_template_documents_level ON template_documents(document_level);

-- Index for active documents
CREATE INDEX idx_template_documents_active ON template_documents(is_active);

-- Index for content hash to detect duplicates efficiently
CREATE INDEX idx_template_documents_hash ON template_documents(document_id, content_hash);

-- View for latest versions only (convenience)
CREATE VIEW latest_template_documents AS
SELECT 
    td1.document_id,
    td1.version_number,
    td1.document_content,
    td1.content_hash,
    td1.document_type,
    td1.document_level,
    td1.purpose,
    td1.variables_schema,
    td1.default_values,
    td1.created_at,
    td1.updated_at,
    td1.is_active,
    td1.metadata
FROM template_documents td1
WHERE td1.version_number = (
    SELECT MAX(td2.version_number)
    FROM template_documents td2
    WHERE td2.document_id = td1.document_id
    AND td2.is_active = TRUE
)
AND td1.is_active = TRUE;