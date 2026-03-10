---
id: campaign-archive-tests-export-and
level: task
title: "Campaign archive tests — export and import round-trip validation"
short_code: "MIMIR-T-0560"
created_at: 2026-03-10T01:31:56.030724+00:00
updated_at: 2026-03-10T13:48:50.849101+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Campaign archive tests — export and import round-trip validation

**Phase 6** — Homebrew & Advanced Features

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Rust integration tests for campaign archive export and import. Test that a full campaign round-trips correctly: export to archive, import to new campaign, verify all data matches. This is critical for data integrity — a broken export/import can lose user campaign data.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Export produces a valid archive file with all campaign data
- [ ] Archive includes: campaign metadata, characters, modules, documents, homebrew, maps
- [ ] Import creates a new campaign from the archive
- [ ] Imported campaign has correct name, description, metadata
- [ ] Imported characters match originals (stats, classes, inventory, proficiencies)
- [ ] Imported modules match originals (documents, monsters, maps)
- [ ] Imported homebrew items/monsters/spells match originals
- [ ] Import with duplicate campaign name handles gracefully
- [ ] Corrupt/malformed archive returns error (not crash)
- [ ] All tests pass in CI

## Key Files

- `crates/mimir/src/commands/campaign.rs` — export/import commands
- `crates/mimir-core/src/services/archive.rs` — archive service (if exists)

## Implementation Notes

Create a test campaign with representative data (characters, modules, documents, homebrew, maps), export it, then import into a fresh database. Compare all entities field-by-field. This is a high-value test — archive corruption is one of the worst possible data loss scenarios.

## Status Updates

### Completed
- Archive service already had 16 tests covering: export file creation, manifest/data contents, preview, catalog references, nonexistent campaign/file errors, import round-trip (name, modules, documents, characters), UUID regeneration, name collision auto-increment, invalid archive rejection, empty campaign round-trip, document content/title preservation, module-document associations
- Added 3 new tests covering gaps in acceptance criteria:
  - `test_homebrew_item_round_trip`: Verifies homebrew items (name, type, rarity, cloned_from fields, data JSON), monsters (name, CR, creature_type, size), and spells (name, level, school) all survive export→import. Also verifies preview counts include homebrew.
  - `test_character_details_round_trip`: Verifies character ability scores (all 6), race/source, background, class records (name, source), inventory (item name, equipped flag, notes), and proficiencies (name, type) survive round-trip.
  - `test_module_entities_round_trip`: Verifies module monsters (name, source, display_name, notes) and NPCs (name, role, description, personality) survive round-trip with correct module association.
- All 19 tests passing (16 existing + 3 new)
- Added seed helpers: `seed_campaign_with_homebrew()` and `seed_campaign_with_detailed_character()`