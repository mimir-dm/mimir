-- Workflow cards table for the Three-Board System
CREATE TABLE workflow_cards (
    id TEXT PRIMARY KEY,  -- UUID as string
    board_type TEXT NOT NULL CHECK(board_type IN ('campaign', 'module', 'session')),
    title TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_moved_at TEXT NOT NULL DEFAULT (datetime('now')),
    workflow_state TEXT NOT NULL,
    
    -- Entity references (only one should be non-null based on board_type)
    campaign_id INTEGER,
    module_id INTEGER,
    session_id INTEGER,
    
    priority INTEGER NOT NULL DEFAULT 0,
    
    FOREIGN KEY (campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE,
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    
    -- Ensure only one entity reference is set based on board type
    CHECK (
        (board_type = 'campaign' AND campaign_id IS NOT NULL AND module_id IS NULL AND session_id IS NULL) OR
        (board_type = 'module' AND module_id IS NOT NULL AND campaign_id IS NULL AND session_id IS NULL) OR
        (board_type = 'session' AND session_id IS NOT NULL AND campaign_id IS NULL AND module_id IS NULL)
    )
);

-- Card tags table (many-to-many relationship)
CREATE TABLE workflow_card_tags (
    card_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (card_id, tag),
    FOREIGN KEY (card_id) REFERENCES workflow_cards(id) ON DELETE CASCADE
);

CREATE INDEX idx_workflow_cards_board_type ON workflow_cards(board_type);
CREATE INDEX idx_workflow_cards_state ON workflow_cards(workflow_state);
CREATE INDEX idx_workflow_cards_board_state ON workflow_cards(board_type, workflow_state);
CREATE INDEX idx_workflow_cards_campaign ON workflow_cards(campaign_id);
CREATE INDEX idx_workflow_cards_module ON workflow_cards(module_id);
CREATE INDEX idx_workflow_cards_session ON workflow_cards(session_id);
CREATE INDEX idx_workflow_cards_priority ON workflow_cards(priority);
CREATE INDEX idx_workflow_cards_last_moved ON workflow_cards(last_moved_at);