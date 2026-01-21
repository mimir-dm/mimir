-- Migration 006 rollback: Remove character building tables

DROP INDEX IF EXISTS idx_feats_source;
DROP INDEX IF EXISTS idx_feats_name;
DROP TABLE IF EXISTS feats;

DROP INDEX IF EXISTS idx_backgrounds_source;
DROP INDEX IF EXISTS idx_backgrounds_name;
DROP TABLE IF EXISTS backgrounds;

DROP INDEX IF EXISTS idx_races_source;
DROP INDEX IF EXISTS idx_races_name;
DROP TABLE IF EXISTS races;

DROP INDEX IF EXISTS idx_subclasses_source;
DROP INDEX IF EXISTS idx_subclasses_class;
DROP INDEX IF EXISTS idx_subclasses_name;
DROP TABLE IF EXISTS subclasses;

DROP INDEX IF EXISTS idx_classes_source;
DROP INDEX IF EXISTS idx_classes_name;
DROP TABLE IF EXISTS classes;
