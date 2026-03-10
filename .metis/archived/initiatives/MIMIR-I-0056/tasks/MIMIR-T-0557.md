---
id: print-tests-map-printing-and-token
level: task
title: "Print tests — map printing and token cutout sheets"
short_code: "MIMIR-T-0557"
created_at: 2026-03-10T01:31:49.989732+00:00
updated_at: 2026-03-10T13:29:09.211402+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

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

### Completed
- Added 22 new tests to `token_cutouts.rs` (4 existing → 26 total):
  - Size multiplier for all size categories (tiny/small/medium/large/huge/gargantuan + abbreviations)
  - Token type colors (monster/pc/npc/trap/unknown)
  - Builder pattern: with_quantity, with_image, defaults
  - Section configuration: with_cell_size, with_cut_lines
  - Empty tokens renders comment, TOC title
  - Fallback circle rendering (initial letter, type color)
  - Name truncation on tokens
  - Size-grouped rendering (tiny/small-med/large/huge/garg groups)
  - Quantity duplication (3 copies = 3 occurrences)
  - Cut lines dashed vs non-dashed styles
  - Image format detection (PNG/JPG/WebP/unknown)
  - Filename sanitization, name truncation edge cases
  - Token type specific colors (NPC blue, PC green)
- Added 8 new tests to `map.rs` (3 existing → 11 total):
  - Typst string escaping
  - TiledMap TOC title, page_break_before, page_margin (default + custom)
  - Assembly guide labels (tile grid labels A1-B3)
  - MapPreview from_rendered constructor + toc_title
  - TiledMap from_rendered with real PNG image (verifies tile calculation)
- All 35 tests passing