---
id: water-body-and-elevation-contour
level: task
title: "Water body and elevation contour generation"
short_code: "MIMIR-T-0575"
created_at: 2026-03-11T21:23:34.440906+00:00
updated_at: 2026-03-11T22:54:36.191931+00:00
parent: MIMIR-I-0058
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0058
---

# Water body and elevation contour generation

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0058]]

## Objective

Generate water bodies (lakes, swamps, shores) and elevation contours (hills, cliffs) using Marching Squares on the noise map.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `water` module: `WaterGenerator` — extract water polygons at a configurable noise threshold using Marching Squares, output DD `WaterTree` structure with deep/shallow colors and blend distance
- [ ] Water edge paths: shore/bank `MapPath` along water polygon edges
- [ ] Polygon merging: handle multiple contour segments that should form a single water body
- [ ] Island mode support: when noise is modified with island mode, water naturally forms at map edges
- [ ] `elevation` module: `ElevationGenerator` — extract contour lines at multiple noise thresholds, output as cliff `MapPath` entities
- [ ] Shadow paths: optional offset paths below cliff paths for depth illusion (e.g., shadow texture at lower layer)
- [ ] Configurable hill levels: list of noise thresholds, each with texture, width, layer, smoothing factor, contour filter (min length)
- [ ] Integration test: generate water and hills for island-mode noise, verify water polygons form around edges and contour paths are valid

## Implementation Notes

- Marching Squares produces raw contour segments — need to chain them into polylines and close polygons
- Water tree has a recursive structure (children for islands within water)
- Contour filtering: discard contours shorter than `min_length` to avoid tiny artifacts
- Contour smoothing: average point positions with neighbors to reduce jaggedness
- Reference impl uses perpendicular offset for shadow paths — compute offset along the normal of each contour segment

### Dependencies
Depends on: MIMIR-T-0569 (scaffold), MIMIR-T-0570 (format/WaterTree/MapPath), MIMIR-T-0571 (Marching Squares + Bezier)

## Status Updates

### 2026-03-11
- `water` module: `generate_water()` extracts contour polygons at threshold → smooths → builds DD WaterTree with children
- `water_from_polygon()`: builds WaterTree from river corridor polygon
- `WaterConfig`: threshold, colors, blend distance, min contour points, smoothing
- `elevation` module: `generate_elevation()` extracts contour lines at multiple thresholds → scales to pixels → produces MapPath entities
- Shadow paths: optional perpendicular offset paths below cliff contours for depth
- `ElevationConfig` with multiple `ContourLevel`s, each with texture/width/layer/shadow
- 8 unit tests: basic water gen, uniform (no water), polygon builder, colors, elevation basic/shadows/uniform/scaling