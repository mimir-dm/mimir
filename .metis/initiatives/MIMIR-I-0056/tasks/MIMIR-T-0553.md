---
id: print-tests-character-sheet-pdf
level: task
title: "Print tests — character sheet PDF generation and content validation"
short_code: "MIMIR-T-0553"
created_at: 2026-03-10T01:31:45.430334+00:00
updated_at: 2026-03-10T01:31:45.430334+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

*To be added during implementation*