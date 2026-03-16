---
id: road-effort-model-elevation-aware
level: task
title: "Road effort model: elevation-aware pathfinding with configurable contour-crossing penalty"
short_code: "MIMIR-T-0636"
created_at: 2026-03-16T18:01:08.138469+00:00
updated_at: 2026-03-16T18:01:08.138469+00:00
parent: MIMIR-I-0063
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0063
---

# Road effort model: elevation-aware pathfinding with configurable contour-crossing penalty

**Depends on:** MIMIR-T-0635 (contours must exist before roads)

## Objective

Add an `effort` parameter (0.0-1.0) to `RoadConfig`. Modify the greedy walk scoring in road pathfinding to penalize contour crossings scaled by `(1.0 - effort)`. At effort=1.0, roads ignore contours entirely. When roads cross contours, the contour clipping pass from T-0635 clips contour paths at the road corridor edge.

## Acceptance Criteria

- [ ] `RoadConfig` gains `effort: f32` field (default 0.5), clamped to 0.0-1.0
- [ ] Greedy walk scoring queries contour data from `GeneratedFeatures` and adds crossing penalty proportional to `(1.0 - effort)`
- [ ] `effort=0.0`: roads strongly avoid contour crossings, routing around elevated terrain
- [ ] `effort=1.0`: roads path as if contours don't exist (straight-line behavior unchanged from current)
- [ ] `effort=0.5`: roads accept some contour crossings but prefer flatter routes
- [ ] Contours crossed by the final road path are clipped at road corridor edge (reuses T-0635 clipping)
- [ ] Existing configs without `effort` field deserialize with default and produce unchanged output
- [ ] Generated maps opened and visually verified in Dungeondraft by human tester
- [ ] `cargo test -p mimir-mapgen` passes

## Implementation Notes

### Key files
- `paths.rs` — modify greedy walk scoring to accept contour data and effort parameter; add contour-crossing penalty to cost function
- `pipeline.rs` — thread `effort` from `RoadConfig` into path generation; pass contour data from registry
- `elevation.rs` — expose contour line positions for spatial queries (point-in-corridor or nearest-contour distance)

### Approach
1. Add `effort` to `RoadConfig` with `#[serde(default = "default_effort")]` returning 0.5
2. In greedy walk, for each candidate step, count contour lines crossed. Multiply crossing count by `(1.0 - effort) * contour_penalty_weight`
3. Contour query: use spatial index or brute-force segment intersection (contour count is small)
4. After road path is finalized, contour clipping from T-0635 handles visual cleanup

## Status Updates