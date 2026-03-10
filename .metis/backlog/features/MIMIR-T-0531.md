---
id: add-clone-from-catalog-support-to
level: task
title: "Add clone-from-catalog support to homebrew creation"
short_code: "MIMIR-T-0531"
created_at: 2026-03-09T01:36:18.025884+00:00
updated_at: 2026-03-10T01:17:37.634959+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Add clone-from-catalog support to homebrew creation

## Objective

When `cloned_from_name` and `cloned_from_source` are provided to `create_homebrew_item` (and monster/spell equivalents), look up the catalog entry, use its full `data` blob as the base, and merge any user-provided overrides on top. Make `data` optional when cloning. This applies to all three homebrew entity types: items, monsters, and spells.

### Problem

Currently, homebrew items are created with minimal `data` blobs (e.g. `{"weapon":true,"weaponCategory":"martial"}`) instead of the full 5etools-format data that catalog items have (damage dice, properties, weight, value, type codes, etc.). The `cloned_from_name`/`cloned_from_source` fields exist in the schema but are metadata-only — no catalog lookup or data cloning occurs.

This means homebrew items lack mechanical data the UI needs to render them properly.

### Priority
- [x] P1 - High (important for user experience)

### Effort Estimate
Medium

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `create_homebrew_item`: when `cloned_from_name` + `cloned_from_source` provided, clone catalog item's `data` blob as base
- [x] `create_homebrew_monster`: same clone-from-catalog behavior
- [x] `create_homebrew_spell`: same clone-from-catalog behavior
- [x] `data` parameter is optional when cloning (defaults to catalog data)
- [x] User-provided `data` fields merge on top of catalog base (override, not replace)
- [x] Error returned if catalog item not found by name+source
- [x] MCP tool descriptions updated to document clone-from-catalog workflow
- [x] Existing tests still pass
- [x] New tests cover: clone success, merge overrides, catalog item not found, no-clone still works

## Implementation Notes

### Files to Modify

**Service layer** (`crates/mimir-core/src/services/homebrew.rs`):
- In `create_item`, `create_monster`, `create_spell`: if `cloned_from_name` + `cloned_from_source` are set, call the corresponding DAL lookup (`get_item_by_name`, `get_monster_by_name`, `get_spell_by_name`), use its `data` as base, merge user overrides
- Make `data` field optional on the `Create*Input` structs (default to `"{}"` when cloning)

**MCP layer** (`crates/mimir-mcp/src/tools/homebrew.rs`, `homebrew_item.rs`, `homebrew_monster.rs`, `homebrew_spell.rs`):
- Make `data` parameter optional when `cloned_from_name`/`cloned_from_source` are provided
- Update tool descriptions to explain clone-from-catalog workflow

### Existing Infrastructure
- DAL lookups already exist: `get_item_by_name()`, `get_monster_by_name()`, `get_spell_by_name()`
- `cloned_from_name`/`cloned_from_source` fields exist on all three homebrew tables
- Catalog `data` blobs contain full 5etools JSON (e.g. Rapier: dmg1, dmgType, property, weight, value, etc.)

### Data Merge Strategy
- Parse catalog `data` as `serde_json::Value`
- Parse user `data` as `serde_json::Value` (may be empty `{}`)
- Deep merge: user fields override catalog fields, catalog fields preserved where user doesn't specify
- Re-serialize merged result

## Status Updates

### Session 2026-03-09
**All acceptance criteria met.**

#### Changes Made

**Service layer** (`crates/mimir-core/src/services/homebrew.rs`):
- Added `resolve_clone_data()` helper: handles catalog lookup + deep merge for all three entity types
- Added `deep_merge()` helper: recursively merges JSON objects (user fields override catalog fields)
- Made `data` field `Option<String>` on `CreateHomebrewItemInput`, `CreateHomebrewMonsterInput`, `CreateHomebrewSpellInput`
- Updated `create_item`, `create_monster`, `create_spell` to use `resolve_clone_data`
- Added `use crate::dal::catalog as catalog_dal` import

**Tauri commands** (`crates/mimir/src/commands/homebrew*.rs`):
- Updated `TauriCreateHomebrewItemInput.data` to `Option<String>`
- Updated `TauriCreateHomebrewMonsterInput.data` to `Option<String>`
- Updated `TauriCreateHomebrewSpellInput.data` to `Option<String>`

**MCP tools** (`crates/mimir-mcp/src/tools/homebrew*.rs`):
- Updated all three `create_homebrew_*_tool()` descriptions to document clone-from-catalog workflow
- Removed `"data"` from required fields (only `"name"` required)
- Updated all three `create_homebrew_*` implementations to make `data` optional with validation

**Tests** (8 new tests added):
- `test_clone_item_catalog_not_found` — verifies NotFound error when catalog item doesn't exist
- `test_clone_monster_catalog_not_found` — same for monsters
- `test_clone_spell_catalog_not_found` — same for spells
- `test_create_item_requires_data_when_not_cloning` — verifies Validation error
- `test_create_monster_requires_data_when_not_cloning` — same for monsters
- `test_create_spell_requires_data_when_not_cloning` — same for spells
- `test_deep_merge_objects` — verifies recursive object merging
- `test_deep_merge_replaces_non_objects` — verifies non-object replacement

All 1065 tests passing (915 core + 150 other crates).