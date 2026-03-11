---
id: terrain-generation-from-noise-with
level: task
title: "Terrain generation from noise with splat map output"
short_code: "MIMIR-T-0572"
created_at: 2026-03-11T21:23:30.880534+00:00
updated_at: 2026-03-11T22:50:05.387868+00:00
parent: MIMIR-I-0058
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0058
---

# Terrain generation from noise with splat map output

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0058]]

## Objective

Generate terrain data from a noise map: map Perlin noise values to 4 texture slots with configurable boundaries/blending, and encode as a DD-compatible splat map `PoolByteArray`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `terrain` module: `TerrainGenerator::generate(noise_map, config)` → `Terrain` struct with splat data and texture assignments
- [ ] Configurable 4-slot terrain: each slot has a noise boundary range and a `res://` texture path
- [ ] Boundary blending: smooth transitions between terrain types with configurable blend width
- [ ] Splat map encoding: 4 bytes per cell (RGBA weights summing to ~255), at 4x4 cells per grid square
- [ ] Road terrain modification: accept road corridor data to blend terrain along roads (terrain value override in road area)
- [ ] Integration test: generate terrain for a 10x10 map, verify splat byte count = `10*4 * 10*4 * 4 = 6400`
- [ ] Visual verification: output splat data matches expected noise-to-terrain mapping for known seed

## Implementation Notes

- Noise map resolution may differ from splat resolution (16 cells/square) — interpolate or resample
- Reference impl uses boundaries at absolute noise values (e.g., 200, 120, 50 on 0-255 scale) with weighted blending
- Weight normalization: ensure RGBA always sums to 255 for each cell
- Road terrain: the reference impl overrides terrain within a road corridor — accept a mask or polygon

### Dependencies
Depends on: MIMIR-T-0569 (scaffold), MIMIR-T-0570 (format types), MIMIR-T-0571 (noise module)

## Status Updates

### 2026-03-11
- `terrain` module: `generate_terrain()` maps noise → 4-slot splat weights with configurable boundaries + blend zones
- `TerrainConfig` with 4 `TerrainSlot`s (texture path, lower/upper noise bounds, blend width)
- Splat normalization: floor-based rounding with fractional remainder distribution, guaranteed sum=255
- `apply_road_corridor()`: overrides terrain weights within a radius of road center points
- 6 unit tests: dimensions, weight normalization, pure/blend zones, road corridor, textures