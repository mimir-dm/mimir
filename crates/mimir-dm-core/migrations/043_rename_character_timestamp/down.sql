-- Revert: rename updated_at back to last_updated_at
ALTER TABLE characters RENAME COLUMN updated_at TO last_updated_at;
