CREATE TABLE catalog_rewards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    reward_type TEXT NOT NULL,
    description TEXT NOT NULL,
    has_prerequisites INTEGER NOT NULL DEFAULT 0,
    is_srd INTEGER NOT NULL DEFAULT 0,
    source TEXT NOT NULL,
    full_reward_json TEXT NOT NULL
);

-- Create indices for better performance
CREATE INDEX idx_catalog_rewards_name ON catalog_rewards(name);
CREATE INDEX idx_catalog_rewards_type ON catalog_rewards(reward_type);
CREATE INDEX idx_catalog_rewards_source ON catalog_rewards(source);
CREATE INDEX idx_catalog_rewards_prerequisites ON catalog_rewards(has_prerequisites);