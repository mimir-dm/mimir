-- Rename last_updated_at to updated_at for consistency with other tables
ALTER TABLE characters RENAME COLUMN last_updated_at TO updated_at;
