-- Migration 007 rollback: Remove remaining entity tables

DROP INDEX IF EXISTS idx_deities_pantheon;
DROP INDEX IF EXISTS idx_deities_source;
DROP INDEX IF EXISTS idx_deities_name;
DROP TABLE IF EXISTS deities;

DROP INDEX IF EXISTS idx_cults_source;
DROP INDEX IF EXISTS idx_cults_name;
DROP TABLE IF EXISTS cults;

DROP INDEX IF EXISTS idx_hazards_source;
DROP INDEX IF EXISTS idx_hazards_name;
DROP TABLE IF EXISTS hazards;

DROP INDEX IF EXISTS idx_traps_source;
DROP INDEX IF EXISTS idx_traps_name;
DROP TABLE IF EXISTS traps;

DROP INDEX IF EXISTS idx_objects_source;
DROP INDEX IF EXISTS idx_objects_name;
DROP TABLE IF EXISTS objects;

DROP INDEX IF EXISTS idx_vehicles_type;
DROP INDEX IF EXISTS idx_vehicles_source;
DROP INDEX IF EXISTS idx_vehicles_name;
DROP TABLE IF EXISTS vehicles;

DROP INDEX IF EXISTS idx_languages_type;
DROP INDEX IF EXISTS idx_languages_source;
DROP INDEX IF EXISTS idx_languages_name;
DROP TABLE IF EXISTS languages;

DROP INDEX IF EXISTS idx_actions_source;
DROP INDEX IF EXISTS idx_actions_name;
DROP TABLE IF EXISTS actions;

DROP INDEX IF EXISTS idx_skills_source;
DROP INDEX IF EXISTS idx_skills_name;
DROP TABLE IF EXISTS skills;

DROP INDEX IF EXISTS idx_senses_source;
DROP INDEX IF EXISTS idx_senses_name;
DROP TABLE IF EXISTS senses;

DROP INDEX IF EXISTS idx_diseases_source;
DROP INDEX IF EXISTS idx_diseases_name;
DROP TABLE IF EXISTS diseases;

DROP INDEX IF EXISTS idx_conditions_source;
DROP INDEX IF EXISTS idx_conditions_name;
DROP TABLE IF EXISTS conditions;
