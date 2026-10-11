-- Adventures in the reader (MIMIR-T-0736): a book row is a book or an
-- adventure; `meta` holds an adventure's index entry from adventures.json
-- (its source, group, level range, storyline, authors) as JSON.
ALTER TABLE books ADD COLUMN kind TEXT NOT NULL DEFAULT 'book';
ALTER TABLE books ADD COLUMN meta TEXT;
