-- Remove archived_at column and index
DROP INDEX idx_campaigns_archived;
ALTER TABLE campaigns DROP COLUMN archived_at;