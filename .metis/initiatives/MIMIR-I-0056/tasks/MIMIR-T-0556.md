---
id: print-tests-equipment-cards
level: task
title: "Print tests — equipment cards (catalog items, homebrew items, type code mapping)"
short_code: "MIMIR-T-0556"
created_at: 2026-03-10T01:31:48.990397+00:00
updated_at: 2026-03-10T01:31:48.990397+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

*To be added during implementation*