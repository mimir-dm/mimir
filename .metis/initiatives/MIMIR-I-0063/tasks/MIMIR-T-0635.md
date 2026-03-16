---
id: pipeline-reorder-generate-contours
level: task
title: "Pipeline reorder: generate contours before roads/rivers + contour clipping at corridors"
short_code: "MIMIR-T-0635"
created_at: 2026-03-16T18:01:07.070480+00:00
updated_at: 2026-03-16T21:06:53.452589+00:00
parent: MIMIR-I-0063
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0063
---

# Pipeline reorder: generate contours before roads/rivers + contour clipping at corridors

**Depends on:** nothing (first task in initiative)
**Blocks:** MIMIR-T-0636, MIMIR-T-0637

## Objective

Reorder the mapgen pipeline so contours generate before roads and rivers. Add post-processing to clip contour paths around road and river corridors, preventing contour lines from visually crossing linear features.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Pipeline stage order changed: contour generation runs before road and river generation
- [ ] New contour clipping pass removes contour segments that fall within road or river corridor bounds (road_width/2 + configurable margin)
- [ ] Contour paths split at corridor intersections produce clean endpoints (no dangling stubs)
- [ ] Maps without roads or rivers generate identical contours to current behavior (no regression)
- [ ] Contour data stored in `GeneratedFeatures` registry so downstream stages (roads, rivers) can query it
- [ ] Generated maps opened and visually verified in Dungeondraft by human tester
- [ ] `cargo test -p mimir-mapgen` passes

## Implementation Notes

### Key files
- `pipeline.rs` — reorder stages so contours run before paths; store contour data in `GeneratedFeatures`
- `elevation.rs` or new `contour_clip.rs` — clipping logic that takes contour polylines + corridor polygons and returns trimmed polylines
- `paths.rs` — expose road/river corridor geometry for clipping

### Approach
1. Move contour generation (currently in elevation stage) to run immediately after noise/elevation, before roads/rivers
2. After roads and rivers generate, run a clipping pass over stored contour paths
3. Clipping: for each contour segment, test against expanded corridor rectangles; split contour at intersection points and discard interior segments
4. Margin should default to ~2px beyond half-width to avoid visual overlap

## Status Updates

### 2026-03-16
- Reordered pipeline: contours now generate at step 3b (after terrain, before roads/rivers)
- Created `contour_clip.rs` with `clip_contours_against_corridors()` — walks contour polylines, removes segments within corridor half-width + 32px margin
- Added `contour_polylines` and `corridors` fields to `GeneratedFeatures`
- Road and river generation now registers corridors in features registry
- After roads/rivers, clipping pass trims contour paths before adding to level
- DD-validated: gull-rock-forest cliffs no longer cross through road
- 206 tests pass (4 new contour_clip tests)