---
id: implement-catalog-tauri-commands
level: task
title: "Implement Catalog Tauri commands"
short_code: "MIMIR-T-0404"
created_at: 2026-01-21T16:34:49.068079+00:00
updated_at: 2026-01-21T19:04:38.531168+00:00
parent: MIMIR-I-0045
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0045
---

# Implement Catalog Tauri commands

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0045]]

## Objective

Implement Tauri commands wrapping `mimir-core` catalog services for searching monsters, items, spells, and other 5e content.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Monster search with filtering (CR, type, size, source)
- [x] Item search with filtering (type, rarity, source)
- [x] Spell search with filtering (level, school, class, source)
- [x] Get individual entity details
- [x] Support for all catalog entity types

## Implementation Notes

### Core Commands

```rust
// Monsters
#[tauri::command] fn search_monsters(state, query: MonsterQuery) -> Result<Vec<MonsterSummary>>
#[tauri::command] fn get_monster(state, name: String, source: String) -> Result<Monster>

// Items
#[tauri::command] fn search_items(state, query: ItemQuery) -> Result<Vec<ItemSummary>>
#[tauri::command] fn get_item(state, name: String, source: String) -> Result<Item>

// Spells
#[tauri::command] fn search_spells(state, query: SpellQuery) -> Result<Vec<SpellSummary>>
#[tauri::command] fn get_spell(state, name: String, source: String) -> Result<Spell>
```

### Additional Entity Commands (as needed)
- Classes, Races, Backgrounds, Feats
- Conditions, Actions, Languages
- Traps, Hazards

### Query Patterns
Use existing `CatalogEntityService` trait implementations in `mimir-core`.

### Dependencies
- Blocked by: [[MIMIR-T-0399]] (Rust backend setup)

## Status Updates

### 2026-01-21: Implementation Complete

**Completed work:**
1. Added `serde::Deserialize` to all 12 catalog filter types in `mimir-core/src/models/catalog/`:
   - MonsterFilter, SpellFilter, ItemFilter, RaceFilter, BackgroundFilter, ClassFilter
   - FeatFilter, ConditionFilter, LanguageFilter, TrapFilter, HazardFilter, ActionFilter

2. Created `crates/mimir/src/commands/catalog.rs` with 60 Tauri commands:
   - For each of 12 entity types: search, get by ID, get by name/source, list sources, count
   - Uses existing `CatalogEntityService` trait implementations from mimir-core
   - Follows established patterns from other command modules (character.rs, document.rs)

3. Registered all catalog commands in `main.rs` invoke_handler

**Entity types supported:**
- Monsters, Spells, Items, Races, Backgrounds, Classes
- Feats, Conditions, Languages, Traps, Hazards, Actions

**Build status:** Successful with only pre-existing warnings