-- Modules table for the Three-Board System
CREATE TABLE modules (
    id INTEGER PRIMARY KEY,
    campaign_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    module_number INTEGER NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('planning', 'development', 'ready', 'active', 'completed')),
    expected_sessions INTEGER NOT NULL DEFAULT 4,
    actual_sessions INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    completed_at TEXT,
    FOREIGN KEY (campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE
);

CREATE INDEX idx_modules_campaign ON modules(campaign_id);
CREATE INDEX idx_modules_status ON modules(status);
CREATE UNIQUE INDEX idx_modules_campaign_number ON modules(campaign_id, module_number);