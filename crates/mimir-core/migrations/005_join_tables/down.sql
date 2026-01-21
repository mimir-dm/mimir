-- Migration 005 rollback: Remove join tables

DROP INDEX IF EXISTS idx_item_attunement_class;
DROP INDEX IF EXISTS idx_item_attunement_item;
DROP TABLE IF EXISTS item_attunement_classes;

DROP INDEX IF EXISTS idx_spell_subclasses_source;
DROP INDEX IF EXISTS idx_spell_subclasses_subclass;
DROP INDEX IF EXISTS idx_spell_subclasses_spell;
DROP TABLE IF EXISTS spell_subclasses;

DROP INDEX IF EXISTS idx_spell_classes_source;
DROP INDEX IF EXISTS idx_spell_classes_class;
DROP INDEX IF EXISTS idx_spell_classes_spell;
DROP TABLE IF EXISTS spell_classes;
