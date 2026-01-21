-- Campaign Assets: User-uploaded images (maps, props, puzzles, etc.)
-- Files stored in app data directory, DB tracks metadata and path
-- Assets can belong to a campaign OR a module (exactly one must be set)

CREATE TABLE campaign_assets (
    id TEXT PRIMARY KEY NOT NULL,

    -- Parent (exactly one must be set)
    campaign_id TEXT REFERENCES campaigns(id) ON DELETE CASCADE,
    module_id TEXT REFERENCES modules(id) ON DELETE CASCADE,

    -- File metadata
    filename TEXT NOT NULL,           -- Original filename
    mime_type TEXT NOT NULL,          -- e.g., "image/png", "image/svg+xml"
    blob_path TEXT NOT NULL,          -- Relative path in app data dir
    file_size INTEGER,                -- Size in bytes

    -- Timestamps
    uploaded_at TEXT NOT NULL DEFAULT (datetime('now')),

    -- Ensure exactly one parent is set
    CHECK (
        (campaign_id IS NOT NULL AND module_id IS NULL) OR
        (campaign_id IS NULL AND module_id IS NOT NULL)
    )
);

CREATE INDEX idx_campaign_assets_campaign ON campaign_assets(campaign_id);
CREATE INDEX idx_campaign_assets_module ON campaign_assets(module_id);
