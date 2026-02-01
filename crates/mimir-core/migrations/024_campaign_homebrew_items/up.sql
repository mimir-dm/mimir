-- Campaign Homebrew Items table
-- Stores custom items created by DMs within a campaign, optionally cloned from catalog items.

CREATE TABLE campaign_homebrew_items (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    item_type TEXT,
    rarity TEXT,
    data TEXT NOT NULL,
    cloned_from_name TEXT,
    cloned_from_source TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(campaign_id, name)
);

CREATE INDEX idx_campaign_homebrew_items_campaign ON campaign_homebrew_items(campaign_id);
