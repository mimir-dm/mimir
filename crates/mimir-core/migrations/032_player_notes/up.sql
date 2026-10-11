-- Player notes and party sharing (MIMIR-T-0718, ADR MIMIR-A-0010).

-- A player's notes about their encounters, in markdown; the DM reads them.
CREATE TABLE player_notes (
    id TEXT PRIMARY KEY NOT NULL,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    -- the module the note is about (optional)
    module_id TEXT REFERENCES modules(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_player_notes_character ON player_notes(character_id);
CREATE INDEX idx_player_notes_module ON player_notes(module_id);

-- A row: the character's player shares the sheet with the party (read-only
-- for the other players). No row: not shared (the default).
CREATE TABLE sheet_shares (
    character_id TEXT PRIMARY KEY NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
