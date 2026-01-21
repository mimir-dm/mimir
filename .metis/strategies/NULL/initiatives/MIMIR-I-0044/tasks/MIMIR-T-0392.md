---
id: implement-remaining-catalog
level: task
title: "Implement remaining catalog services (Spell, Item, Race, Background, etc.)"
short_code: "MIMIR-T-0392"
created_at: 2026-01-21T03:02:30.004771+00:00
updated_at: 2026-01-21T03:21:33.185460+00:00
parent: MIMIR-I-0044
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0044
---

# Implement remaining catalog services (Spell, Item, Race, Background, etc.)

## Parent Initiative

[[MIMIR-I-0044]] - Implement Service Layer for mimir-core

## Objective

Implement `CatalogEntityService` for all remaining catalog entity types, following the pattern established by `MonsterService`. This provides unified search and retrieval for all D&D 5e content.

## Acceptance Criteria

- [x] `SpellService` - search by level, school, class, components
- [x] `ItemService` - search by type, rarity, attunement
- [x] `RaceService` - search by source, size
- [x] `BackgroundService` - search by source
- [x] `ClassService` - search by source
- [x] `FeatService` - search by prerequisite
- [x] `ConditionService` - list/get (simple, no complex filters)
- [x] `ActionService` - list/get
- [x] `LanguageService` - list/get (with type filtering)
- [x] `TrapService` - search by source, tier
- [x] `HazardService` - search by source
- [x] All services have unit tests (47 total)
- [x] All services exported from `services::catalog` module

## Implementation Notes

### Files to Create

```
crates/mimir-core/src/services/catalog/
├── mod.rs              # Re-exports all services
├── monster.rs          # (from T-0391)
├── spell.rs
├── item.rs
├── race.rs
├── background.rs
├── class.rs
├── feat.rs
├── condition.rs
├── action.rs
├── language.rs
├── trap.rs
└── hazard.rs
```

### Service Priority

High priority (needed for character creation):
1. `RaceService` - character race selection
2. `BackgroundService` - character background selection
3. `ClassService` - character class selection
4. `SpellService` - spell selection for casters
5. `ItemService` - equipment and magic items

Lower priority (reference content):
6. `FeatService`
7. `ConditionService`
8. `ActionService`
9. `LanguageService`
10. `TrapService`
11. `HazardService`

### Pattern

Each service follows the same pattern:
1. Struct with `conn: &'a mut SqliteConnection`
2. `new()` constructor
3. Implement `CatalogEntityService` trait
4. Delegate to DAL functions
5. Map `diesel::result::Error` to `ServiceError::Database`

### Dependencies

- MIMIR-T-0391 (CatalogEntityService trait, MonsterService pattern)
- Existing DAL modules for each entity type
- Existing model types and filters

## Status Updates

### Completed - 2026-01-20

**All 11 catalog services implemented:**

Services created in `crates/mimir-core/src/services/catalog/`:
- `spell.rs` - SpellService with level, school, ritual, cantrip filtering
- `item.rs` - ItemService with rarity, type, attunement filtering
- `race.rs` - RaceService with source, size filtering  
- `background.rs` - BackgroundService with source filtering
- `class.rs` - ClassService with source filtering
- `feat.rs` - FeatService with name, source filtering
- `condition.rs` - ConditionService with name, source filtering
- `action.rs` - ActionService with name, source filtering
- `language.rs` - LanguageService with type (standard/exotic/secret) filtering
- `trap.rs` - TrapService with tier (simple/complex) filtering
- `hazard.rs` - HazardService with name, source filtering

**Implementation approach:**
- Each service implements `CatalogEntityService` trait
- Services delegate to DAL functions, mapping errors with `ServiceError::from`
- Filter types use builder pattern (`with_name_contains`, `with_source`, etc.)
- All services have 3-4 unit tests each

**Test results:** 47 catalog service tests pass, 634 total unit tests pass