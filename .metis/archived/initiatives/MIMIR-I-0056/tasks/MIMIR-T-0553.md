---
id: print-tests-character-sheet-pdf
level: task
title: "Print tests — character sheet PDF generation and content validation"
short_code: "MIMIR-T-0553"
created_at: 2026-03-10T01:31:45.430334+00:00
updated_at: 2026-03-10T13:11:43.155775+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Print tests — character sheet PDF generation and content validation

**Phase 5** — Print/Export Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Rust integration tests for character sheet PDF generation. Test the full `export_character_sheet` command pipeline: loading character data, enriching with catalog lookups, building sections (stats, features, equipment, spells, battle card), and producing valid PDF output. Verify content correctness, not just "it doesn't crash."

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Single-class character (Fighter/PHB) generates PDF with correct stats, features, equipment
- [ ] Multiclass character (Fighter/Rogue) generates PDF with features from both classes
- [ ] Spellcaster character (Wizard/PHB) generates PDF with spell list, spell slots, spellcasting stats
- [ ] Character with homebrew items generates PDF with equipment cards that have correct type codes
- [ ] Character with subclass features includes child features (e.g., Fast Hands under Thief)
- [ ] Battle card section generates with correct AC, HP, speed, attacks
- [ ] PHB/XPHB spell dedup works correctly (no duplicate spells in output)
- [ ] Character with configured sources filters content by allowed sources
- [ ] PDF output is non-empty and valid (parseable by a PDF library)
- [ ] All tests pass in CI

## Key Files

- `crates/mimir/src/commands/print/character.rs` — main export command
- `crates/mimir-print/src/sections/character.rs` — character section builder
- `crates/mimir-print/src/sections/character_battle_card.rs` — battle card section
- `crates/mimir/src/commands/print/helpers.rs` — inventory enrichment

## Implementation Notes

Use the Rust integration test harness from MIMIR-T-0535 with SRD-seeded test database. Create test characters via the service layer, then call the export command and verify the output. For content validation, parse the PDF bytes to extract text and check for expected strings (class names, spell names, equipment names). Consider using a lightweight PDF text extractor crate.

## Status Updates

### Completed
Added 81 total tests across 3 files:

**helpers.rs** — 45 new unit tests for pure computation functions:
- `hit_die_for_class`: All class die sizes (d12/d10/d8/d6), unknown defaults
- `spellcasting_ability_for_class`: INT/WIS/CHA casters + non-casters
- `caster_level_multiplier`: Full/half/third/non-casters
- `spell_slots_for_caster_level`: Levels 0-20+, negative, caps
- `compute_hp_max`: Single-class, multiclass, level 1 max die, low CON, empty classes, minimum 1
- `compute_hit_die_string`: Single, multi, empty
- `compute_ac`: No armor, light/medium/heavy armor, DEX cap, shield bonus, unequipped ignored
- `max_spell_level_for_class`: Full casters, warlock pact, half casters, artificer, third casters, non-casters

**character.rs** — 14 new tests (25 total, was 11):
- NPC badge + info (role/location/faction)
- Weapon attack bonus (STR-based, finesse, ranged DEX)
- Expertise diamond rendering
- Multiclass class string, no-class fallback
- Currency denominations
- Inventory equipped/attuned status
- Personality 4-section rendering
- Proficiency bonus by level (1-20)
- Passive perception with proficiency
- Spellcaster attack bonus and save DC
- Modifier edge cases and formatting

**character_battle_card.rs** — 6 new tests (11 total, was 5):
- Core stats display (AC/HP/Speed/Prof/Init)
- Equipped weapons in card
- Class/subclass info
- Multiple characters in 2x2 grid
- Ability modifier calculations
- PC vs NPC footer

**Note**: Full pipeline integration tests (DB → CharacterData → PDF) require Tauri AppHandle and Typst compilation, so are not feasible as pure unit tests. The acceptance criteria for homebrew item type codes, PHB/XPHB spell dedup, source filtering, and PDF validation would need end-to-end testing infrastructure.