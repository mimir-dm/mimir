-- Campaign System: Core Hierarchy
-- Campaign, CampaignSource (allowed books), Module (adventure chapters)

-- Campaigns table: top-level container for all campaign data
CREATE TABLE campaigns (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    archived_at TEXT,  -- ISO8601 timestamp, NULL = active
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Campaign sources: which source books are allowed in this campaign
CREATE TABLE campaign_sources (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    source_code TEXT NOT NULL REFERENCES catalog_sources(code),
    UNIQUE(campaign_id, source_code)
);

CREATE INDEX idx_campaign_sources_campaign ON campaign_sources(campaign_id);

-- Modules: organizational containers for adventure chapters
CREATE TABLE modules (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    module_number INTEGER NOT NULL,  -- Ordering within campaign
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(campaign_id, module_number)
);

CREATE INDEX idx_modules_campaign ON modules(campaign_id);
