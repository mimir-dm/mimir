---
id: object-placement-trees-clumps-and
level: task
title: "Object placement: trees, clumps, and clutter via Poisson Disc"
short_code: "MIMIR-T-0573"
created_at: 2026-03-11T21:23:32.213829+00:00
updated_at: 2026-03-11T22:51:50.302405+00:00
parent: MIMIR-I-0058
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0058
---

# Object placement: trees, clumps, and clutter via Poisson Disc

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0058]]

## Objective

Place objects (trees, clumps, clutter) on the map using noise-gated Poisson Disc sampling, outputting `MapObject` entities for the DD format.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `objects` module: `TreePlacer` — places trees from a texture list using Poisson Disc + noise gating, with configurable min distance, probability, noise thresholds, size range, layer, rotation/mirror randomization
- [ ] `objects` module: tree shadows — optional shadow objects placed under trees with offset, separate texture, and lower layer
- [ ] `objects` module: tree canopy — optional canopy objects on level 1 (upper layer) for overhead foliage
- [ ] `objects` module: `ClumpPlacer` — places primary objects with clustered secondary objects at configurable distance/count ranges
- [ ] `objects` module: `ClutterPlacer` — simple scattered objects (grass, flowers) with noise gating
- [ ] All placers output `Vec<MapObject>` compatible with the format module
- [ ] Custom color support: clutter can have `custom_color` applied (e.g., tinted flowers)
- [ ] Objects respect the `NodeIdAllocator` for unique ref IDs
- [ ] Integration test: generate objects for a known seed, verify count is deterministic and positions are within map bounds

## Implementation Notes

- Reference config has multiple tree type groups with different noise bands (dense forest vs sparse edge)
- Clumps have primary + secondary objects: primary is placed by Poisson Disc, secondaries scattered around it within a radius
- Object positions are in pixels (grid_square * 256), with sub-pixel precision
- Rotation is in radians, random 0-2π unless `randomrotation: false`
- Scale uses `Vector2(s, s)` where s is random in configured size range
- See **MIMIR-S-0002** for default asset paths per object type

### Dependencies
Depends on: MIMIR-T-0569 (scaffold), MIMIR-T-0570 (format/MapObject), MIMIR-T-0571 (noise + Poisson Disc)

## Status Updates

### 2026-03-11
- `objects` module with `ObjectConfig`, `TreeConfig`, `ClumpConfig` structs
- `place_objects()`: noise-gated Poisson Disc with configurable textures, scale range, rotation, mirror, custom color, layer
- `place_trees()`: composite placement with shadow (lower layer, offset) and canopy (upper level) objects
- `place_clumps()`: primary objects via Poisson Disc + secondary scatter within radius
- `clear_corridor()`: removes objects within a distance of road/path center points
- 8 unit tests: basic placement, noise gating, determinism, shadows, clumps, corridor clearing, custom colors, unique refs