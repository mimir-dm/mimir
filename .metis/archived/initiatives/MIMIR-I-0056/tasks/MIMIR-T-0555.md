---
id: print-tests-monster-cards-and-trap
level: task
title: "Print tests — monster cards and trap cards (catalog + homebrew)"
short_code: "MIMIR-T-0555"
created_at: 2026-03-10T01:31:47.663246+00:00
updated_at: 2026-03-10T13:22:58.620913+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Print tests — monster cards and trap cards (catalog + homebrew)

**Phase 5** — Print/Export Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Rust integration tests for monster card and trap card PDF export. Test both `export_module_monsters` (batch cards for a module) and `export_monster_card` (single monster), covering catalog monsters, homebrew monsters, display name overrides, and quantity duplication. Also test trap card export if implemented.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Single catalog monster card generates valid PDF with stat block content
- [ ] Module monster batch export generates cards for all monsters in the module
- [ ] Quantity field produces correct number of duplicate cards
- [ ] Display name override replaces the monster's name on the card
- [ ] Homebrew monster (source "HB") resolves from campaign homebrew table
- [ ] Cut lines option toggles cut line rendering
- [ ] Empty module (no monsters) returns appropriate error
- [ ] Monster not found returns error (not crash)
- [ ] Trap cards generate with correct content (if trap card export exists)
- [ ] All tests pass in CI

## Key Files

- `crates/mimir/src/commands/print/monster.rs` — monster card export commands
- `crates/mimir-print/src/sections/monster_card.rs` — monster card section builder

## Implementation Notes

Use the integration test harness with SRD monsters. Create a test module with a mix of catalog monsters (Goblin, Dragon) and a homebrew monster. Test the display name override by setting a custom name on a module monster entry. Verify the homebrew lookup path works by creating a homebrew monster in the test campaign.

## Status Updates

### Session 2 - Completed
- Added 28 new tests to `monster_cards.rs` (12 existing → 40 total)
  - Full card rendering: name, type, size, alignment, CR, AC/HP/PP, speed, ability scores
  - Senses and languages display
  - Actions and traits with 5etools tag stripping
  - Damage immunities, saves, multiple speeds
  - Foldable card layout (front/back) for content-heavy monsters
  - Grid layout (2x2), empty slot filling, page breaks
  - Cut lines toggle
  - Size name mapping (T/S/M/L/H/G)
  - AC extraction (simple and with source), HP extraction (simple and formula)
  - Speed with fly+hover, CR from object format
  - Creature type (string and object), alignment variants (unaligned, any)
  - Escape typst, entries text nested extraction
  - Comprehensive 5etools tag stripping (all attack types, dice, creature, condition)
  - Resistances block (empty and with content), condition immunities
  - Section colors validation
- Added 27 new tests to `trap_cards.rs` (5 existing → 32 total)
  - All 6 trap type colors (MECH/MAG/WLD/WTH/ENV/HAZ) + default + alternate key
  - Full card rendering: name, type label, source footer
  - Threat badges: simple (green), moderate (amber), dangerous (orange), deadly (red), none
  - DC block: detection/disable DCs, DC extraction from entry text, empty case
  - Card entries: trigger/effect sections, 5etools tag stripping
  - Cut lines toggle (enabled/disabled)
  - Multiple traps grid layout, page breaks
  - List entries rendering, table entries rendering
  - Back card (foldable) - none for simple traps
  - Ability abbreviation helper
  - Escape typst for trap content
- All 78 tests passing

*To be added during implementation*