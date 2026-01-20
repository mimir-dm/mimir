-- Add archived_at column for soft archiving campaigns
ALTER TABLE campaigns ADD COLUMN archived_at TEXT;

-- Create index for efficient querying of archived campaigns
CREATE INDEX idx_campaigns_archived ON campaigns(archived_at);