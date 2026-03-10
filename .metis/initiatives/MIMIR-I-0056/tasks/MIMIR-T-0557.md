---
id: print-tests-map-printing-and-token
level: task
title: "Print tests — map printing and token cutout sheets"
short_code: "MIMIR-T-0557"
created_at: 2026-03-10T01:31:49.989732+00:00
updated_at: 2026-03-10T01:31:49.989732+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Print tests — map printing and token cutout sheets

**Phase 5** — Print/Export Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Rust integration tests for map printing and token cutout sheet export. Test that maps render with correct dimensions, tokens appear at correct positions, and cutout sheets produce the right number of token images (catalog + homebrew).

## Acceptance Criteria

- [ ] Map print generates PDF with correct page dimensions
- [ ] Token cutout sheet generates with all module monster tokens
- [ ] Catalog monster tokens include the catalog token image reference
- [ ] Homebrew monster tokens use size-based fallback (no catalog image)
- [ ] Token cutout respects quantity (correct number of copies)
- [ ] Empty module (no tokens) returns appropriate error
- [ ] All tests pass in CI

## Key Files

- `crates/mimir/src/commands/print/document.rs` — token cutout and map export
- `crates/mimir-print/src/sections/` — map and token section builders

## Implementation Notes

Map printing depends on UVTT map data format. Create a minimal test map with a few tokens at known positions. Test the homebrew monster path for token cutouts — it should extract size from the JSON data and skip catalog token image lookup. Use SRD monsters for catalog path testing.

## Status Updates

*To be added during implementation*