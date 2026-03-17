---
id: declarative-lakes-placed-water
level: task
title: "Declarative lakes: placed water features with organic noise-perturbed shorelines"
short_code: "MIMIR-T-0638"
created_at: 2026-03-16T18:01:10.414192+00:00
updated_at: 2026-03-16T22:09:41.389587+00:00
parent: MIMIR-I-0063
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0063
---

# Declarative lakes: placed water features with organic noise-perturbed shorelines

**Depends on:** nothing (can be parallel with MIMIR-T-0635)
**Blocks:** MIMIR-T-0637 (rivers connect to lakes)

## Objective

Implement lakes as declarative placed features with center, radius, roughness, and color configuration. Generate organic noise-perturbed shorelines. Depress the noise map within lake boundaries to create natural basins. Contours terminate at lake edges. Object placement excludes lake areas.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `LakeConfig` struct with fields: `id`, `center`, `radius`, `roughness`, `deep_color`, `shallow_color`, `blend_distance`
- [ ] `MapConfig` gains `lakes: Vec<LakeConfig>` section
- [ ] Lake shorelines are noise-perturbed organic shapes (not perfect circles) controlled by `roughness` parameter
- [ ] Noise map depressed within lake boundary to create a natural basin in the elevation data
- [ ] Contour lines terminate at lake shoreline edge (do not enter lake area)
- [ ] Object placement (trees, clutter) excludes lake interior + small margin
- [ ] Lake registered in `GeneratedFeatures` with ID and shoreline polygon for downstream stages
- [ ] Water polygon generated from lake boundary and added to map output
- [ ] Terrain splat map reflects basin coloring within `blend_distance` of shoreline
- [ ] Generated maps opened and visually verified in Dungeondraft by human tester
- [ ] `cargo test -p mimir-mapgen` passes

## Implementation Notes

### New file
- `crates/mimir-mapgen/src/lakes.rs` — lake generation: shoreline polygon from noise-perturbed circle, noise depression, water polygon output

### Key files
- `pipeline.rs` — add `LakeConfig` to `MapConfig`, add lake stage after noise/contours but before roads/rivers; register lake in `GeneratedFeatures`
- `water.rs` — generate DD water polygon from lake shoreline boundary
- `elevation.rs` — depress noise values inside lake boundary; clip contours at lake edge
- `distribution.rs` — add lake polygons to exclusion zones for object placement

### Approach
1. For each lake config, generate shoreline: sample points around circle at `center`/`radius`, offset each radially by noise scaled by `roughness`
2. Depress noise map: for each pixel inside shoreline polygon, set noise to a low floor value (blended at edges by `blend_distance`)
3. Contour clipping: after contour generation, clip contour segments that enter lake polygon (same approach as corridor clipping in T-0635)
4. Water output: convert shoreline polygon to DD water path format
5. Object exclusion: add lake polygon + margin to exclusion list before Poisson disc sampling

## Status Updates

### 2026-03-16
- Created `lakes.rs` with `generate_lake()` — noise-perturbed shoreline from center/radius/roughness
- Shoreline: 120 radial samples, noise perturbation scaled by roughness, Bezier smoothed
- `depress_noise_for_lake()` — lowers noise map inside lake boundary with edge blending
- Lake pipeline stage runs after noise, before contours — so contours respect the depression
- Water tree added as child of root water node (DD format)
- Lake registered in GeneratedFeatures (boundary + center) for downstream stages
- Object exclusion: trees/clutter filtered from lake interior
- Added `LakeConfig` to `MapConfig` with id, center, radius, roughness, colors, blend_distance
- DD-validated: lake with organic shoreline, trees excluded, terrain shows basin
- 5 unit tests (basic, organic, smooth, depression, point-in-polygon)