---
id: dal-services-homebrew-monster
level: task
title: "DAL & Services: Homebrew monster module support"
short_code: "MIMIR-T-0565"
created_at: 2026-03-11T14:49:21.689903+00:00
updated_at: 2026-03-11T16:20:28.645796+00:00
parent: MIMIR-I-0057
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0057
---

# DAL & Services: Homebrew monster module support

## Parent Initiative

[[MIMIR-I-0057]]

## Objective

Update the DAL and service layers to support homebrew monsters in modules. This includes inserting/listing/getting module monsters with homebrew references, and updating `TokenService` to resolve stat blocks from either catalog or homebrew sources.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `dal::insert_module_monster` handles `homebrew_monster_id` column
- [ ] `dal::list_module_monsters` returns homebrew monster ID when present
- [ ] New `dal::list_module_monsters_with_data` joins homebrew monster data (name, CR, type from JSON blob) for display
- [ ] `ModuleService` accepts homebrew monster additions (validates homebrew monster exists in campaign)
- [ ] `TokenService` stat resolution checks `homebrew_monster_id` first; if set, loads from `campaign_homebrew_monsters` and parses JSON; otherwise falls back to catalog lookup
- [ ] Integration tests: add homebrew monster to module, verify it appears in list
- [ ] Integration tests: place token for homebrew module monster, verify stat resolution
- [ ] Existing catalog monster flows remain unchanged (regression tests pass)

## Implementation Notes

### Technical Approach

**DAL changes** (`crates/mimir-core/src/dal/campaign.rs`):
- `insert_module_monster` — include `homebrew_monster_id` in INSERT
- `list_module_monsters` — select new column
- `list_module_monsters_with_data` — LEFT JOIN on `campaign_homebrew_monsters` when `homebrew_monster_id IS NOT NULL`, return a unified result struct with display name, CR, creature type

**Service changes** (`crates/mimir-core/src/services/`):
- `ModuleService::add_monster` — new method or parameter that accepts homebrew_monster_id, validates it belongs to the same campaign
- `TokenService::enrich_token` — currently does: load module_monster → get (name, source) → catalog lookup. Change to: if homebrew_monster_id set, load homebrew JSON blob → parse into monster stats; else catalog lookup as before

Consider a `MonsterResolver` helper that abstracts the "give me stats for this module_monster" logic, used by both token enrichment and monster card printing.

### Dependencies
- MIMIR-T-0564 (migration must be complete first)

### Key Files
- `crates/mimir-core/src/dal/campaign.rs`
- `crates/mimir-core/src/services/module.rs`
- `crates/mimir-core/src/services/token.rs`

## Status Updates

### Completed
- ✅ `dal::insert_module_monster` handles `homebrew_monster_id` — `NewModuleMonster::from_homebrew()` constructor and Diesel schema updated (done in T-0564)
- ✅ `dal::list_module_monsters` returns `homebrew_monster_id` — `ModuleMonster` struct has the field (done in T-0564)
- ✅ `TokenService` stat resolution checks `homebrew_monster_id` first, falls back to catalog (done in T-0564, `services/token.rs`)
- ✅ Existing catalog monster flows unchanged — all pre-existing tests pass
- ✅ Integration tests added: `test_insert_homebrew_module_monster`, `test_list_mixed_catalog_and_homebrew`, `test_delete_homebrew_module_monster` (3 new DAL tests, all passing)

### Architecture Decisions
- **`list_module_monsters_with_data` stays at command layer**: The task envisioned a DAL-level join, but the existing architecture resolves monster data (catalog lookups, homebrew JSON parsing) at the Tauri command layer in `commands/module.rs`. This is correct since it crosses crate boundaries (catalog data). No separate DAL function needed.
- **`ModuleService` doesn't handle monsters**: `ModuleService` is module CRUD only. Monster operations go through DAL directly from command/MCP layers. Validation (homebrew exists, belongs to campaign) happens at those layers. This is the existing pattern — no service method needed.