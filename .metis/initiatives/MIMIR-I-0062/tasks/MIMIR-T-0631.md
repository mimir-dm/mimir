---
id: pattern-placement-region-based
level: task
title: "Pattern placement: region-based texture fills for water, rooms, polygons, and noise areas"
short_code: "MIMIR-T-0631"
created_at: 2026-03-15T00:42:43.618169+00:00
updated_at: 2026-03-15T00:42:43.618169+00:00
parent: MIMIR-I-0062
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0062
---

# Pattern placement: region-based texture fills for water, rooms, polygons, and noise areas

**Depends on:** MIMIR-T-0627 (format structs), MIMIR-T-0628 (named IDs)

## Objective

Implement region-based pattern placement. Patterns are polygon-bounded texture fills — used for floor tiles inside rooms, water overlays, courtyard surfaces, and noise-gated ground detail. Each produces a `MapPattern` entry pushed to `Level.patterns`.

## Acceptance Criteria

- [ ] `PatternConfig` struct with region enum (Water, Room, Polygon, Noise) and shared fields (texture, color, rotation, layer, outline)
- [ ] `Water` region: uses generated water polygon boundaries from `GeneratedFeatures`
- [ ] `Room` region: uses named room wall boundary polygon
- [ ] `Polygon` region: uses named polygon boundary
- [ ] `Noise` region: generates contour polygon at noise threshold via `contour.rs` marching squares
- [ ] New `src/patterns.rs` module with `generate_patterns(config, noise_map, features, node_id_counter) -> Vec<MapPattern>`
- [ ] Pipeline wires patterns stage after rooms/polygons/water, pushes to `Level.patterns`
- [ ] Config section: `patterns: Vec<PatternConfig>` in `MapConfig`
- [ ] Pattern `points` are in pixel coords, `position` defaults to (0,0), `node_id` allocated from counter
- [ ] Generated maps with patterns open correctly in DD and render as tiled fills
- [ ] Unit tests for each region type
- [ ] `cargo test -p mimir-mapgen` passes

## Implementation Notes

### New file
- `crates/mimir-mapgen/src/patterns.rs`

### Reuse
- `contour.rs` — marching squares for noise region boundaries
- `GeneratedFeatures` registry — water/room/polygon boundary lookups
- `rooms.rs` — room boundary geometry

### DD Pattern format (from MIMIR-S-0001 spike)
```json
{
  "position": "Vector2( 0, 0 )",
  "shape_rotation": 0,
  "scale": "Vector2( 1, 1 )",
  "points": "PoolVector2Array( ... )",
  "layer": 100,
  "color": "ff929292",
  "outline": false,
  "texture": "res://textures/tilesets/simple/tileset_cobble.png",
  "rotation": 0,
  "node_id": "0"
}
```

## Status Updates

*To be added during implementation*