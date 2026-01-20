CREATE TABLE catalog_languages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    language_type TEXT NOT NULL,
    script TEXT NOT NULL,
    typical_speakers TEXT NOT NULL,
    is_srd INTEGER NOT NULL DEFAULT 0,
    source TEXT NOT NULL,
    full_language_json TEXT NOT NULL
);

-- Create indices for better performance
CREATE INDEX idx_catalog_languages_name ON catalog_languages(name);
CREATE INDEX idx_catalog_languages_type ON catalog_languages(language_type);
CREATE INDEX idx_catalog_languages_source ON catalog_languages(source);
CREATE INDEX idx_catalog_languages_script ON catalog_languages(script);