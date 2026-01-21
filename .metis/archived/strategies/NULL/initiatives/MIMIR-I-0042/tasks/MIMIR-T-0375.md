---
id: migration-005-class-race
level: task
title: "Migration 005: Class, Race, Background, Feat tables"
short_code: "MIMIR-T-0375"
created_at: 2026-01-20T02:43:49.585475+00:00
updated_at: 2026-01-20T20:36:38.278252+00:00
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

# Migration 005: Class, Race, Background, Feat tables

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0042]]

## Objective

Create Diesel migration for character-building catalog entities: classes, subclasses, races, backgrounds, and feats.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Create `classes` table with indexed columns (name, source)
- [ ] Create `subclasses` table with class reference
- [ ] Create `races` table with indexed columns (name, source)
- [ ] Create `backgrounds` table with indexed columns (name, source)
- [ ] Create `feats` table with indexed columns (name, source)
- [ ] All tables store full 5etools JSON in `data` column
- [ ] Diesel schema.rs generated and compiles

## SQL Schema

```sql
-- Character classes
CREATE TABLE classes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_classes_name ON classes(name);

-- Subclasses (linked to parent class)
CREATE TABLE subclasses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    class_name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, class_name, source)
);
CREATE INDEX idx_subclasses_name ON subclasses(name);
CREATE INDEX idx_subclasses_class ON subclasses(class_name);

-- Races/lineages
CREATE TABLE races (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_races_name ON races(name);

-- Character backgrounds
CREATE TABLE backgrounds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_backgrounds_name ON backgrounds(name);

-- Character feats
CREATE TABLE feats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES catalog_sources(code),
    data TEXT NOT NULL,
    UNIQUE(name, source)
);
CREATE INDEX idx_feats_name ON feats(name);
```

## Implementation Notes

### 5etools Field Mapping

| Entity | 5etools File Pattern | Notes |
|--------|---------------------|-------|
| Class | `class/class-*.json` | Complex nested structure with class features |
| Subclass | `class/class-*.json` | Embedded in class files under `subclass` array |
| Race | `races.json` | Includes subraces, lineages |
| Background | `backgrounds.json` | Simple structure |
| Feat | `feats.json` | May have prerequisites |

### Dependencies

- MIMIR-T-0370 (CatalogSource table must exist first)

## Status Updates

### Session 2026-01-20
- Created migration files `migrations/006_character_building/up.sql` and `down.sql`
- Ran diesel migration, schema.rs auto-updated with all new tables
- Created model files:
  - `models/catalog/class.rs` - Class, NewClass
  - `models/catalog/subclass.rs` - Subclass, NewSubclass
  - `models/catalog/race.rs` - Race, NewRace
  - `models/catalog/background.rs` - Background, NewBackground
  - `models/catalog/feat.rs` - Feat, NewFeat
- Created DAL files with CRUD operations:
  - `dal/catalog/class.rs` - insert, get, list, delete, count
  - `dal/catalog/subclass.rs` - insert, get, list (by class), delete, count
  - `dal/catalog/race.rs` - insert, get, list, search, delete, count
  - `dal/catalog/background.rs` - insert, get, list, delete, count
  - `dal/catalog/feat.rs` - insert, get, list, search, delete, count
- All 228 tests passing