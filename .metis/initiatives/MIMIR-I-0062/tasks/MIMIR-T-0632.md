---
id: material-scatter-bit-packed-bitmap
level: task
title: "Material scatter: bit-packed bitmap generation for ice, lava, acid, and ground detail"
short_code: "MIMIR-T-0632"
created_at: 2026-03-15T00:42:44.314119+00:00
updated_at: 2026-03-15T00:42:44.314119+00:00
parent: MIMIR-I-0062
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0062
---

# Material scatter: bit-packed bitmap generation for ice, lava, acid, and ground detail

**Depends on:** MIMIR-T-0627 (format structs), MIMIR-T-0628 (named IDs)

## Objective

Implement region-based material scatter generation. Materials are bit-packed bitmaps representing ground-level detail like ice, lava, acid, and debris. Each produces a `MaterialEntry` in `Level.materials` keyed by layer.

## Acceptance Criteria

- [ ] `MaterialConfig` struct with region enum (Noise, Room, Polygon, AlongPath) and shared fields (texture, layer, smooth, density)
- [ ] `Noise` region: set bits where noise falls in `noise_lower..noise_upper` range, gated by probability/density
- [ ] `Room`/`Polygon` region: set bits inside named room/polygon boundary
- [ ] `AlongPath` region: set bits within a distance of a named road/river path
- [ ] Bitmap encoding: `(map_width * 2 + 3) × (map_height * 2 + 3)` grid, flat bit-packed, LSB-first (per MIMIR-S-0001)
- [ ] New `src/materials.rs` module with `generate_materials(config, noise_map, features, map_width, map_height) -> BTreeMap<String, Vec<MaterialEntry>>`
- [ ] Pipeline wires materials stage, merges into `Level.materials`
- [ ] Config section: `materials: Vec<MaterialConfig>` in `MapConfig`
- [ ] Generated maps with materials open correctly in DD and render scatter textures
- [ ] Unit tests: bitmap dimensions correct for various map sizes, bit-setting logic, boundary detection
- [ ] `cargo test -p mimir-mapgen` passes

## Implementation Notes

### New file
- `crates/mimir-mapgen/src/materials.rs`

### Bitmap encoding details (from MIMIR-S-0001)
- Cell grid: `(w*2+3) × (h*2+3)`, each cell = 0.5 grid squares, +3 border for blending
- Flat bit-packed: bit index = `row * cell_width + col`
- Total bytes: `ceil(cell_width * cell_height / 8)`
- LSB-first within each byte
- Example: 35×20 map → 73×43 cells → 3139 bits → 393 bytes

### Point-in-region testing
- For noise regions: sample noise at cell center, compare to threshold
- For room/polygon: point-in-polygon test against boundary
- For along-path: distance-to-polyline test against corridor width

### Reuse
- `GeneratedFeatures` registry — room/polygon/path boundary lookups
- Noise map — for noise-gated placement

## Status Updates

*To be added during implementation*