---
id: homebrewservice-unit-tests
level: task
title: "HomebrewService unit tests"
short_code: "MIMIR-T-0526"
created_at: 2026-03-08T22:48:29.177688+00:00
updated_at: 2026-03-08T23:23:19.949191+00:00
parent: MIMIR-I-0055
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0055
---

# HomebrewService unit tests

## Parent Initiative
[[MIMIR-I-0055]]

## Objective
Add `#[cfg(test)] mod tests` to `crates/mimir-core/src/services/homebrew.rs` covering all CRUD operations for items, monsters, and spells, plus the `enrich_monster_data` helper and JSON validation.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [x] ~30 unit tests in homebrew.rs test module (39 tests)
- [x] Item CRUD: create, list, get, get_by_name, update, delete, not-found errors
- [x] Monster CRUD: same + data enrichment verification (cr/type/size injected into data blob)
- [x] Spell CRUD: same with level/school fields
- [x] enrich_monster_data: adds missing fields, preserves existing, handles invalid JSON
- [x] validate_json: valid and invalid cases
- [x] cloned_from metadata preserved on create
- [x] All tests pass via `cargo test -p mimir-core -- homebrew` (42 passed including 3 binding tests)

## Implementation Notes
- Tests already partially written (appended to homebrew.rs) — need to verify they compile and pass
- Uses `setup_test_db()` + `NewCampaign` pattern from document.rs tests
- File: `crates/mimir-core/src/services/homebrew.rs`

## Status Updates
- 2026-03-08: ~30 tests written inline, not yet verified
- 2026-03-08: All 39 tests compiled and passed (42 total including 3 binding export tests). Coverage:
  - enrich_monster_data: 9 tests (add missing cr/type/size, preserve existing, all fields, no args noop, invalid JSON)
  - validate_json: 2 tests (valid, invalid)
  - Item CRUD: 8 tests (create, create invalid json, list, get, get_by_name, update, update not found, delete, delete not found)
  - Monster CRUD: 9 tests (create, create with enrichment preserving existing, create empty data enriched, list, update, update not found, delete, delete not found, get_by_name)
  - Spell CRUD: 8 tests (create, create invalid json, list, update, update not found, delete, delete not found, get_by_name)
  - cloned_from: 2 tests (item with clone source, monster with clone source)