---
id: road-and-river-generation-with
level: task
title: "Road and river generation with path smoothing"
short_code: "MIMIR-T-0574"
created_at: 2026-03-11T21:23:33.126697+00:00
updated_at: 2026-03-11T22:53:14.934023+00:00
parent: MIMIR-I-0058
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0058
---

# Road and river generation with path smoothing

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0058]]

## Objective

Generate roads and rivers that traverse the map using greedy pathfinding along noise ridges/valleys, with Bezier smoothing and object corridor clearing.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `paths` module: `RoadGenerator` — greedy pathfinding from one map edge to another following high noise values, configurable step distance, FOV angle, progress weight, and margin
- [ ] Bezier smoothing of road waypoints into smooth `MapPath` edit points
- [ ] Road corridor: compute a polygon around the road for object clearing and terrain modification
- [ ] Object clearing: given a road corridor and a list of placed objects, remove objects within the corridor
- [ ] Road edge paths: optional edge/border paths (e.g., cliff texture) along road sides with configurable offset
- [ ] `RiverGenerator` — similar pathfinding but follows noise valleys (low values), outputs water polygon + bank paths
- [ ] River features: `MapPath` for flow effects (whitecaps), edge bank paths, and a water polygon for the river body
- [ ] Configurable road/river: from/to edge (left/right/top/bottom), width, texture, layer
- [ ] Integration test: generate a road across a 20x20 map, verify path connects edges and has valid edit points

## Implementation Notes

- Reference impl's greedy walk: at each step, evaluate candidate positions within an FOV cone, score by (noise_value * weight + progress_toward_target * (1-weight)), pick best
- Bezier density controls how many interpolated points per segment
- Road terrain blending: modify terrain splat weights within the road corridor to create a dirt/gravel path appearance
- River water: construct a water polygon from the road corridor expanded by river width, use DD water tree format
- Object clearing should happen after object placement — accept mutable object list

### Dependencies
Depends on: MIMIR-T-0569 (scaffold), MIMIR-T-0570 (format/MapPath), MIMIR-T-0571 (noise + Bezier)

## Status Updates

### 2026-03-11
- `paths` module: `RoadConfig`, `RiverConfig`, `EdgePathConfig` structs
- `generate_road()`: greedy walk along noise ridges → Bezier smoothed → MapPath with optional edge paths
- `generate_river()`: greedy walk along noise valleys → bank paths (offset polyline) + water polygon
- `greedy_walk()`: evaluates 7 candidates per step within FOV cone, scores by noise*weight + progress*(1-weight)
- `RoadResult` includes corridor points + half-width for object clearing / terrain modification
- `RiverResult` includes water polygon (closed outline from left+right bank offsets)
- 6 unit tests: walk reaches target, road gen, road with edges, river gen, determinism, edge point generation