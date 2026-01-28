-- Character Features table
-- Stores class feature choices: Fighting Style, Metamagic, Maneuvers, Invocations, Pact Boon

CREATE TABLE character_features (
    id TEXT PRIMARY KEY NOT NULL,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    feature_type TEXT NOT NULL,  -- 'fighting_style', 'metamagic', 'maneuver', 'invocation', 'pact_boon'
    feature_name TEXT NOT NULL,
    feature_source TEXT NOT NULL,
    source_class TEXT NOT NULL   -- which class granted this feature
);

CREATE INDEX idx_character_features_character ON character_features(character_id);
CREATE INDEX idx_character_features_type ON character_features(feature_type);
