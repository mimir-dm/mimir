---
id: campaign-archive-tests-export-and
level: task
title: "Campaign archive tests — export and import round-trip validation"
short_code: "MIMIR-T-0560"
created_at: 2026-03-10T01:31:56.030724+00:00
updated_at: 2026-03-10T01:31:56.030724+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

*To be added during implementation*