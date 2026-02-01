-- Campaign Homebrew Spells table
-- Stores custom spells created by DMs within a campaign, optionally cloned from catalog spells.

CREATE TABLE campaign_homebrew_spells (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    level INTEGER,
    school TEXT,
    data TEXT NOT NULL,
    cloned_from_name TEXT,
    cloned_from_source TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(campaign_id, name)
);

CREATE INDEX idx_campaign_homebrew_spells_campaign ON campaign_homebrew_spells(campaign_id);
