---
id: general-path-generation-waypoints
level: task
title: "General path generation: waypoints, room-to-room, offset, and intermittent styles"
short_code: "MIMIR-T-0630"
created_at: 2026-03-15T00:42:42.309130+00:00
updated_at: 2026-03-15T00:42:42.309130+00:00
parent: MIMIR-I-0062
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0062
---

# General path generation: waypoints, room-to-room, offset, and intermittent styles

**Depends on:** MIMIR-T-0627 (format structs), MIMIR-T-0628 (named IDs)

## Objective

Implement general-purpose path generation beyond the existing road/river generators. Four styles: waypoints (explicit coordinates), room-to-room (connect rooms by name), offset (companion path at perpendicular distance from a parent feature), intermittent (break a parent path into segments with random gaps). All produce `MapPath` entries. Existing roads/rivers remain untouched.

## Acceptance Criteria

- [ ] `CustomPathConfig` struct with style enum (Waypoints, RoomToRoom, Offset, Intermittent) and shared fields (texture, width, color, layer, smooth, loop)
- [ ] `Waypoints` style: accepts points in grid coordinates, converts to pixels, optional Bezier smoothing via `curves::bezier_smooth`
- [ ] `RoomToRoom` style: looks up two room centers from `GeneratedFeatures`, generates a path between them with smoothing
- [ ] `Offset` style: looks up a named parent feature's polyline, applies `curves::offset_polyline` at configurable distance, supports `reverse` flag
- [ ] `Intermittent` style: looks up a parent feature, breaks into segments with configurable `segment_length`, `segment_variation`, and `gap`
- [ ] New `src/custom_paths.rs` module
- [ ] Pipeline wires custom paths stage after roads/rivers/elevation, pushes to `Level.paths`
- [ ] Config section: `paths: Vec<CustomPathConfig>` in `MapConfig`
- [ ] Path `edit_points` are relative to `position` (per DD format spec MIMIR-S-0001)
- [ ] Generated maps with paths open correctly in DD and paths render visually
- [ ] Unit tests for each style
- [ ] `cargo test -p mimir-mapgen` passes

## Implementation Notes

### New file
- `crates/mimir-mapgen/src/custom_paths.rs`

### Reuse
- `curves.rs` — `bezier_smooth()` for smoothing, `offset_polyline()` for offset style
- `GeneratedFeatures` registry — room/road/river/elevation lookups

### Gull Rock reference patterns
- Offset shadow paths: offset 0.3 from cliff contours, shadow texture, layer 300
- Intermittent water flows: density 0.2, length 8 ± 1 squares, gap 1
- Edge paths: along road/river edges with offset

## Status Updates

*To be added during implementation*