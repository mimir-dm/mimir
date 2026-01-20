-- Create uploaded_books table for tracking uploaded book archives
CREATE TABLE uploaded_books (
    id TEXT PRIMARY KEY,              -- Book ID (e.g., "PHB", "DMG") - prevents collisions
    name TEXT NOT NULL,               -- Display name (e.g., "Player's Handbook") 
    location TEXT NOT NULL,           -- Full path to extracted directory
    archive_path TEXT NOT NULL,       -- Full path to stored .tar.gz file
    uploaded_at TEXT NOT NULL,        -- ISO timestamp when uploaded
    metadata_json TEXT                -- Full metadata.json content for reference
);

-- Index for common queries
CREATE INDEX idx_uploaded_books_uploaded_at ON uploaded_books(uploaded_at);
CREATE INDEX idx_uploaded_books_name ON uploaded_books(name);