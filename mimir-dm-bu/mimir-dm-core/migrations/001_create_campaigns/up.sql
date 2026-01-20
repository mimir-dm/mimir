-- Campaigns table for the Three-Board System
CREATE TABLE campaigns (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('concept', 'session_zero', 'integration', 'active', 'concluding', 'completed')),
    directory_path TEXT NOT NULL, -- Filesystem path to campaign directory
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    session_zero_date TEXT,  -- ISO date format YYYY-MM-DD
    first_session_date TEXT, -- ISO date format YYYY-MM-DD
    last_activity_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_campaigns_status ON campaigns(status);
CREATE INDEX idx_campaigns_last_activity ON campaigns(last_activity_at);