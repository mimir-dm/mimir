-- Sessions table for the Three-Board System
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY,
    campaign_id INTEGER NOT NULL,
    module_id INTEGER,
    session_number INTEGER NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('next_week', 'prep_needed', 'in_prep', 'ready', 'complete')),
    scheduled_date TEXT,  -- ISO date format YYYY-MM-DD
    prep_started_at TEXT,
    prep_completed_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE,
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE SET NULL
);

CREATE INDEX idx_sessions_campaign ON sessions(campaign_id);
CREATE INDEX idx_sessions_module ON sessions(module_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_scheduled ON sessions(scheduled_date);
CREATE UNIQUE INDEX idx_sessions_campaign_number ON sessions(campaign_id, session_number);