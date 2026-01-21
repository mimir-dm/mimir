-- CharacterClass and CharacterFeat tables
-- Track character class levels and selected feats

-- Character class levels (supports multiclassing)
CREATE TABLE character_classes (
    id TEXT PRIMARY KEY NOT NULL,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    class_name TEXT NOT NULL,
    class_source TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    subclass_name TEXT,
    subclass_source TEXT,
    starting_class INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_character_classes_character ON character_classes(character_id);

-- Character feats
CREATE TABLE character_feats (
    id TEXT PRIMARY KEY NOT NULL,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    feat_name TEXT NOT NULL,
    feat_source TEXT NOT NULL,
    source_type TEXT NOT NULL DEFAULT 'asi'  -- 'asi', 'race', 'class', 'bonus'
);

CREATE INDEX idx_character_feats_character ON character_feats(character_id);
