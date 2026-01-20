CREATE TABLE catalog_psionics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    psionic_type TEXT NOT NULL, -- "D" for Discipline, "T" for Talent
    psionic_order TEXT, -- Avatar, Awakened, Immortal, Nomad, Wu Jen, etc.
    source TEXT NOT NULL,
    page INTEGER,
    full_psionic_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

CREATE INDEX idx_catalog_psionics_name ON catalog_psionics(name);
CREATE INDEX idx_catalog_psionics_type ON catalog_psionics(psionic_type);
CREATE INDEX idx_catalog_psionics_order ON catalog_psionics(psionic_order);
CREATE INDEX idx_catalog_psionics_source ON catalog_psionics(source);