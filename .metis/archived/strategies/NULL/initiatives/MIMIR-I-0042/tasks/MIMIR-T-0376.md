---
id: migration-006-remaining-12-catalog
level: task
title: "Migration 006: Remaining 12 catalog entity tables"
short_code: "MIMIR-T-0376"
created_at: 2026-01-20T02:44:01.453503+00:00
updated_at: 2026-01-20T20:44:37.955948+00:00
parent: MIMIR-I-0042
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0042
---

# Migration 006: Remaining 12 catalog entity tables

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0042]]

## Objective

Create Diesel migration for the remaining 12 catalog entity types: conditions, diseases, senses, skills, actions, languages, vehicles, objects, traps, hazards, cults, and deities.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Create tables for all 12 remaining entity types
- [x] Each table has `name`, `source`, and `data` columns minimum
- [x] All tables reference `catalog_sources(code)` for source
- [x] Index on `name` for each table
- [x] Diesel schema.rs generated and compiles

## SQL Schema

```sql
-- Game conditions (blinded, charmed, etc.)
CREATE TABLE conditions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_conditions_name ON conditions(name);

-- Diseases
CREATE TABLE diseases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_diseases_name ON diseases(name);

-- Senses (darkvision, tremorsense, etc.)
CREATE TABLE senses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_senses_name ON senses(name);

-- Skills (Athletics, Perception, etc.)
CREATE TABLE skills (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_skills_name ON skills(name);

-- Actions (Dash, Dodge, Help, etc.)
CREATE TABLE actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_actions_name ON actions(name);

-- Languages
CREATE TABLE languages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_languages_name ON languages(name);

-- Vehicles (ships, airships, etc.)
CREATE TABLE vehicles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    vehicle_type TEXT,  -- 'ship', 'infernal', 'creature', 'object'
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_vehicles_name ON vehicles(name);

-- Objects (doors, chests, etc.)
CREATE TABLE objects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_objects_name ON objects(name);

-- Traps
CREATE TABLE traps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    trap_tier TEXT,  -- Simple, complex
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_traps_name ON traps(name);

-- Hazards (environmental dangers)
CREATE TABLE hazards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_hazards_name ON hazards(name);

-- Cults and supernatural gifts
CREATE TABLE cults (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_cults_name ON cults(name);

-- Deities
CREATE TABLE deities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    pantheon TEXT,
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_deities_name ON deities(name);
CREATE INDEX idx_deities_pantheon ON deities(pantheon);
```

## Implementation Notes

### 5etools File Mapping

| Entity | 5etools File | Notes |
|--------|-------------|-------|
| Condition | `conditionsdiseases.json` | Shares file with diseases |
| Disease | `conditionsdiseases.json` | Shares file with conditions |
| Sense | `senses.json` | Core senses |
| Skill | `skills.json` | Core skills |
| Action | `actions.json` | Combat actions |
| Language | `languages.json` | - |
| Vehicle | `vehicles.json` | Multiple vehicle types |
| Object | `objects.json` | Interactive objects |
| Trap | `trapshazards.json` | Shares file with hazards |
| Hazard | `trapshazards.json` | Shares file with traps |
| Cult | `cultsboons.json` | Cults and supernatural gifts |
| Deity | `deities.json` | Gods by pantheon |

### Dependencies

- MIMIR-T-0370 (CatalogSource table must exist first)

## Status Updates **[REQUIRED]**

### 2026-01-20: Complete

**Migration 007_remaining_entities created and applied:**
- Created `migrations/007_remaining_entities/up.sql` with all 12 tables
- Created `migrations/007_remaining_entities/down.sql` with DROP TABLE statements
- All tables include: id, name, source (FK to catalog_sources), data, UNIQUE(name, source)
- Additional indexed columns: skills.ability, languages.language_type, vehicles.vehicle_type, objects.object_type, traps.trap_tier, deities.pantheon

**Model types created (12 files in `models/catalog/`):**
- action.rs, condition.rs, cult.rs, deity.rs, disease.rs, hazard.rs
- language.rs, object.rs, sense.rs, skill.rs, trap.rs, vehicle.rs
- Each has: Entity struct (Queryable), NewEntity struct (Insertable), `parse_data()` helper, builder methods for optional fields, unit tests

**DAL functions created (12 files in `dal/catalog/`):**
- All follow established pattern from background.rs
- Standard operations: insert_{entity}, insert_{entities}, get_{entity}, get_{entity}_by_name, list_{entities}, list_{entities}_by_source, delete_{entity}, delete_{entities}_by_source, count_{entities}
- Type-specific operations: list_skills_by_ability, list_languages_by_type, list_vehicles_by_type, list_objects_by_type, list_traps_by_tier, list_simple_traps, list_complex_traps, list_deities_by_pantheon
- All include comprehensive tests with setup_test_db() helper

**Tests: 264 tests passing**