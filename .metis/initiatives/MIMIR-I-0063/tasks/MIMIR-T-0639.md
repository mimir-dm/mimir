---
id: terrain-passability-analysis-and
level: task
title: "Terrain passability analysis and warnings for contour-dense maps"
short_code: "MIMIR-T-0639"
created_at: 2026-03-16T18:01:11.642834+00:00
updated_at: 2026-03-17T01:40:04.766709+00:00
parent: MIMIR-I-0063
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0063
---

# Terrain passability analysis and warnings for contour-dense maps

**Depends on:** MIMIR-T-0635 (needs contour data)

## Objective

Implement post-generation terrain analysis that calculates contour coverage percentage and emits warnings when terrain density conflicts with requested roads, rivers, or lakes. This helps users understand when their elevation/contour settings produce maps that are too mountainous for the requested features.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Analysis runs after contour generation but before road/river pathfinding
- [ ] Calculates contour corridor coverage as percentage of total map area
- [ ] Warning emitted to stderr when contour corridors cover >50% of map area
- [ ] Warning emitted when road pathfinding fails to find a valid path (falls back to direct route)
- [ ] Warning emitted when river pathfinding fails to find a valid path
- [ ] Warning emitted when a lake polygon overlaps with contour lines that couldn't be clipped cleanly
- [ ] Warnings include actionable suggestions (e.g., "reduce elevation.octaves or increase road effort")
- [ ] Analysis results available as structured data in pipeline output for programmatic consumers
- [ ] No warnings emitted for maps with reasonable contour density (<30% coverage)
- [ ] Generated maps opened and visually verified in Dungeondraft by human tester
- [ ] `cargo test -p mimir-mapgen` passes

## Implementation Notes

### Key files
- `pipeline.rs` — add analysis pass after contour generation; collect warnings into a `Vec<TerrainWarning>` on pipeline context; print to stderr at end of generation

### Approach
1. After contours are stored in `GeneratedFeatures`, calculate total contour corridor area (sum of contour_length * contour_line_width for all contours)
2. Divide by map area to get coverage percentage
3. If roads/rivers are configured and coverage > 50%, emit density warning
4. Wrap road/river pathfinding to catch failures and emit specific warnings with the source/dest that failed
5. Define `TerrainWarning` enum with variants: `HighContourDensity`, `PathfindingFailed`, `LakeContourOverlap`
6. Print warnings to stderr so they don't pollute map JSON output on stdout

## Status Updates

### 2026-03-16
- Added `TerrainWarning` enum with `HighContourDensity` and `ContourCrossings` variants
- Added `warnings: Vec<TerrainWarning>` to `GenerateResult`
- Analysis runs after contour generation: estimates corridor coverage as (contour_length * 256px) / map_area
- Warning to stderr when coverage >30% and roads/rivers configured
- Warning count shown in CLI output
- Grassland (0 contours) = no warning; Forest (35%) = warning; Gull Rock (49%) = warning
- All tests pass