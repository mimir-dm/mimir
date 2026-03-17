---
id: river-effort-model-valley
level: task
title: "River effort model: valley-following pathfinding with contour erosion and lake connections"
short_code: "MIMIR-T-0637"
created_at: 2026-03-16T18:01:09.486048+00:00
updated_at: 2026-03-17T01:36:50.203627+00:00
parent: MIMIR-I-0063
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0063
---

# River effort model: valley-following pathfinding with contour erosion and lake connections

**Depends on:** MIMIR-T-0635 (contour data), MIMIR-T-0638 (lake features for connection)

## Objective

Rivers get the same effort model as roads but with inverted noise preference: river pathfinding prefers low noise values (valleys) instead of high. Add `effort` parameter to `RiverConfig`. Rivers can connect to lakes via source/drain references instead of only running edge-to-edge.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `RiverConfig` gains `effort: f32` field (default 0.5), clamped to 0.0-1.0
- [ ] River pathfinding scoring inverted from roads: prefers low noise values (valleys), penalizes high noise (ridges)
- [ ] `effort=0.0`: rivers strongly follow valley lines, avoiding contour crossings
- [ ] `effort=1.0`: rivers path without contour awareness (current behavior)
- [ ] Contours crossed by rivers are clipped at river corridor edge (reuses T-0635 clipping)
- [ ] `RiverConfig` accepts optional `source` and `drain` fields referencing lake IDs from T-0638
- [ ] Rivers with lake source start at nearest lake shoreline point; rivers with lake drain end at nearest lake shoreline point
- [ ] Rivers without lake references continue to work edge-to-edge as before
- [ ] Generated maps opened and visually verified in Dungeondraft by human tester
- [ ] `cargo test -p mimir-mapgen` passes

## Implementation Notes

### Key files
- `paths.rs` — add valley-preference scoring (invert noise cost relative to roads); add effort parameter; add lake endpoint resolution
- `pipeline.rs` — thread `effort` and lake references from `RiverConfig` into path generation; ensure lakes generate before rivers
- `lakes.rs` (from T-0638) — expose shoreline geometry for endpoint snapping

### Approach
1. Refactor greedy walk scoring to accept a `TerrainPreference` enum (HighGround for roads, Valley for rivers) controlling noise cost sign
2. Add `effort` to `RiverConfig` identical to road effort model
3. For lake connections: resolve lake ID to shoreline polygon, find nearest point to river start/end edge position, use that as path endpoint
4. Pipeline order: noise -> contours -> lakes -> rivers -> roads -> contour clipping

## Status Updates

### 2026-03-16
- Added `source` and `drain` optional fields to `RiverConfig` for lake connections
- River start/end resolved to nearest lake shoreline point when source/drain specified
- `nearest_point_on_polygon()` helper for endpoint snapping
- FOV widening at low effort (same as roads)
- Contour avoidance penalty wired through (from T-0636)
- Pipeline passes lake shoreline lookup to river generation
- All tests pass

### Known Issues (need follow-up)
- River water polygon doesn't merge with lake water polygon — they're separate DD water tree children that overlap visually but aren't geometrically connected
- River water polygon shape is rectangular (from convex hull of banks) not organic
- River-to-lake connection needs the river water polygon to extend INTO the lake polygon or the lake polygon to have an inlet cut
- These are water polygon shape issues, not pathfinding issues