---
id: print-tests-equipment-cards
level: task
title: "Print tests — equipment cards (catalog items, homebrew items, type code mapping)"
short_code: "MIMIR-T-0556"
created_at: 2026-03-10T01:31:48.990397+00:00
updated_at: 2026-03-10T13:27:10.214460+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Print tests — equipment cards (catalog items, homebrew items, type code mapping)

**Phase 5** — Print/Export Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Rust integration tests for equipment card PDF export. This was recently broken for homebrew items (missing type codes) and fixed — these tests prevent regressions. Cover catalog items, homebrew items with various type codes, and the `homebrew_item_type_to_code` mapping function.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Catalog weapon item generates equipment card with correct damage/properties
- [ ] Catalog armor item generates equipment card with correct AC
- [ ] Homebrew weapon item generates card (type code injected from `item_type` field)
- [ ] Homebrew armor item generates card with correct type
- [ ] Homebrew wondrous/potion/ring/rod/wand/scroll items get correct type codes
- [ ] `homebrew_item_type_to_code` maps all frontend form values correctly
- [ ] `homebrew_item_type_to_code` passes through raw 5etools codes (M, R, LA, etc.)
- [ ] Equipment card with missing type code renders gracefully (fallback behavior)
- [ ] `enrich_inventory_item` falls back to DB `item_type` for homebrew items
- [ ] All tests pass in CI

## Key Files

- `crates/mimir/src/commands/print/character.rs` — `homebrew_item_type_to_code` function, equipment card generation
- `crates/mimir/src/commands/print/helpers.rs` — `enrich_inventory_item` with homebrew fallback
- `crates/mimir-print/src/sections/equipment_card.rs` — equipment card section builder

## Implementation Notes

This is a high-value regression test task — the homebrew equipment card bug was discovered manually and would have been caught by these tests. Test both the unit-level `homebrew_item_type_to_code` function and the integration-level `export_equipment_cards` command. Create test homebrew items with various `item_type` values.

## Status Updates

### Completed
- Added 18 tests to `character.rs` for `homebrew_item_type_to_code`:
  - Weapon melee (default), ranged (with range), ranged (with ammo property)
  - Case insensitive matching
  - All item types: armor, shield, potion, ring, rod, wand, scroll, staff, wondrous item, adventuring gear
  - Raw 5etools code passthrough (M, R, A, AF, S, LA, MA, HA, RG, RD, WD, W, P, SC)
  - Lowercase passthrough, unknown type returns None, empty range = melee
- Added 27 new tests to `equipment_cards.rs` (10 existing → 37 total):
  - Icon mapping for all types, type name mapping for all types
  - Rarity formatting (common through artifact, none, empty)
  - Full card rendering: weapon (damage/properties), armor (AC), magic item (rarity/attunement/description)
  - Ranged weapon with range/properties, attunement with requirement string
  - Missing type defaults to gear, description truncation at 350 chars
  - Notes as description fallback, damage variations (single/no type)
  - Grid layout (3x3), cut lines toggle, empty section, page breaks after 9 items
  - Icon definitions in output, from_json (array and non-array)
  - is_card_worthy: attunement, entries, pipe-separated type
  - page_break_before, no-damage card (shield)
- All 55 tests passing across both files