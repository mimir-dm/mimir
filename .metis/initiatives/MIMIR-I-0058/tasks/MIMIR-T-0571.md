---
id: core-algorithms-noise-poisson-disc
level: task
title: "Core algorithms: noise, Poisson Disc, Marching Squares, Bezier"
short_code: "MIMIR-T-0571"
created_at: 2026-03-11T21:23:29.581246+00:00
updated_at: 2026-03-11T22:48:16.692316+00:00
parent: MIMIR-I-0058
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0058
---

# Core algorithms: noise, Poisson Disc, Marching Squares, Bezier

## Objective

Implement the foundational math algorithms used across all generation stages: noise generation, spatial distribution, contour extraction, and curve smoothing.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `noise` module: wrapper around the `noise` crate providing `NoiseMap::generate(width, height, config)` → 2D heightmap (`Vec<Vec<f64>>` normalized 0.0-1.0), configurable octaves/persistence/lacunarity/scale, seeded via `rand_chacha`
- [ ] `noise` module: noise modifiers — `island_mode()` (push edges up) and `canyon_mode(levels)` (force below thresholds down)
- [ ] `distribution` module: `PoissonDisc::sample(width, height, min_distance, rng)` → `Vec<(f64, f64)>` point list
- [ ] `distribution` module: noise-gated sampling — `PoissonDisc::sample_gated(noise_map, lower, upper, ...)` only places points where noise is in range
- [ ] `contour` module: Marching Squares — `find_contours(noise_map, threshold)` → `Vec<Vec<(f64, f64)>>` contour polylines
- [ ] `contour` module: contour filtering (min length) and smoothing
- [ ] `curves` module: cubic Bezier evaluation — `bezier_smooth(points, density)` → smoothed point list
- [ ] Unit tests for each algorithm with known inputs/outputs
- [ ] All algorithms accept a seeded `&mut impl Rng` for reproducibility

## Implementation Notes

- Use `noise` crate's `Fbm<Perlin>` for fractal Brownian motion noise
- Poisson Disc: Bridson's algorithm is O(n) and well-suited — may be simple enough to implement directly rather than pulling a crate
- Marching Squares: implement the 16-case lookup table, walk contours, handle ambiguous saddle points
- Bezier: only need cubic evaluation (4 control points → interpolated curve), not the full `lyon` tessellation engine
- Reference impl's `randombiome-maths.rb` has all algorithms — use as reference for expected behavior

### Dependencies
Depends on: MIMIR-T-0569 (crate scaffold)

## Status Updates

### 2026-03-11
- `noise_gen` module: `NoiseMap::generate()` using `Fbm<Perlin>` with configurable octaves/persistence/lacunarity/scale, normalized 0-1 output, bilinear `sample()`, `apply_island_mode()`, `apply_canyon_mode()`, `to_byte_map()`
- `distribution` module: `PoissonDisc::sample()` (Bridson's algorithm O(n)), `sample_gated()` with noise thresholds + probability filtering
- `contour` module: Marching Squares with 16-case lookup, `find_contours()`, `filter_contours()`, `smooth_contours()`, contour tracing with edge interpolation
- `curves` module: `cubic_bezier()`, `bezier_smooth()` (Catmull-Rom style control points), `polyline_length()`, `offset_polyline()` with perpendicular normals
- 23 unit tests for algorithms, all passing