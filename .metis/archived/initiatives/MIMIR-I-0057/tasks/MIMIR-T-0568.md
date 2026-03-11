---
id: print-export-homebrew-monster
level: task
title: "Print & Export: Homebrew monster cards and archive support"
short_code: "MIMIR-T-0568"
created_at: 2026-03-11T14:49:25.348091+00:00
updated_at: 2026-03-11T20:32:00.207257+00:00
parent: MIMIR-I-0057
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0057
---

# Print & Export: Homebrew monster cards and archive support

## Parent Initiative

[[MIMIR-I-0057]]

## Objective

Ensure homebrew monsters in modules work correctly with the print/PDF pipeline (monster cards) and campaign export/import (archive).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `export_module_monsters` generates monster cards for homebrew module monsters using their JSON stat blocks
- [ ] Homebrew monster cards render with the same foldable card layout as catalog monsters
- [ ] `export_campaign` includes homebrew module monster references in the archive
- [ ] `import_campaign` restores homebrew module monster associations (homebrew monsters imported first, then module monster refs reconnected)
- [ ] Monster card for homebrew monster shows "Homebrew" source indicator instead of book abbreviation

## Implementation Notes

### Technical Approach

**Print pipeline** (`crates/mimir-print/src/sections/monster_cards.rs`):
- `export_module_monsters` currently resolves monsters from catalog only
- Add branch: if `homebrew_monster_id` is set, load from `campaign_homebrew_monsters`, parse JSON into the same monster stats struct used for catalog monsters
- Card rendering should be identical — the foldable card system works on the stat block, not the source

**Archive/export** (`crates/mimir-core/src/services/archive.rs` or similar):
- Export: when serializing module monsters, include `homebrew_monster_id` in the JSON
- Import: homebrew monsters are imported as part of campaign data; module monster references need to map old homebrew IDs to new IDs (same UUID remapping pattern used for other entities)

### Dependencies
- MIMIR-T-0564 (migration)
- MIMIR-T-0565 (DAL/services — monster resolution)

### Risk Considerations
- Archive import ID remapping must handle homebrew_monster_id alongside existing module_monster references
- Homebrew monster JSON format must be compatible with the stat block parser used for card rendering

### Key Files
- `crates/mimir-print/src/sections/monster_cards.rs`
- `crates/mimir-lib/src/commands/print.rs`
- `crates/mimir-core/src/services/archive.rs`

## Status Updates

### Completed (mostly done in T-0564, verified and polished here)
- ✅ `export_module_monsters` generates monster cards for homebrew module monsters — checks `homebrew_monster_id` first, loads JSON, renders same card layout
- ✅ Homebrew monster cards use identical foldable card layout as catalog monsters (same `MonsterCardSection`)
- ✅ `export_campaign` includes `homebrew_monster_id` in serialized module monsters
- ✅ `import_campaign` restores homebrew associations: homebrew monsters imported at step 8 (before modules at step 10), old→new ID mapping via `id_maps.homebrew_monsters`
- ✅ Monster cards show "Homebrew" source indicator instead of "HB" abbreviation (updated in `monster.rs`, `document.rs`, and `module.rs`)
- ✅ Archive test verifies round-trip of homebrew monsters
- ✅ All crates compile, all tests pass