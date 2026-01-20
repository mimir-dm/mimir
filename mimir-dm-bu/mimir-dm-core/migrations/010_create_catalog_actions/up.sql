-- Create catalog_actions table for storing action data
CREATE TABLE catalog_actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    time_type TEXT NOT NULL,  -- Store action time (Action, Bonus Action, etc.)
    description TEXT NOT NULL,
    see_also TEXT,           -- JSON array as text for "see also" references
    source TEXT NOT NULL,
    full_action_json TEXT NOT NULL,  -- Store complete JSON for modal display
    UNIQUE(name, source)
);

-- Create indexes for common queries
CREATE INDEX idx_catalog_actions_name ON catalog_actions(name);
CREATE INDEX idx_catalog_actions_source ON catalog_actions(source);
CREATE INDEX idx_catalog_actions_time_type ON catalog_actions(time_type);