-- Book content storage for 5etools books
CREATE TABLE books (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL UNIQUE,  -- Source code (PHB, DMG, etc.) references catalog_sources
    name TEXT NOT NULL,           -- Display name
    data TEXT NOT NULL,           -- Full book content JSON (sections/entries array)
    contents TEXT,                -- Table of contents JSON from books.json
    cover_path TEXT,              -- Path to cover image (local asset path)
    FOREIGN KEY (source) REFERENCES catalog_sources(code)
);

-- Index for quick lookups by source
CREATE INDEX idx_books_source ON books(source);
