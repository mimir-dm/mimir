---
id: general-path-generation-waypoints
level: task
title: "General path generation: waypoints, room-to-room, offset, and intermittent styles"
short_code: "MIMIR-T-0630"
created_at: 2026-03-15T00:42:42.309130+00:00
updated_at: 2026-03-15T11:52:33.017410+00:00
parent: MIMIR-I-0062
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0062
---

# General path generation: waypoints, room-to-room, offset, and intermittent styles

**Depends on:** MIMIR-T-0627 (format structs), MIMIR-T-0628 (named IDs)

## Objective

Implement general-purpose path generation beyond the existing road/river generators. Four styles: waypoints (explicit coordinates), room-to-room (connect rooms by name), offset (companion path at perpendicular distance from a parent feature), intermittent (break a parent path into segments with random gaps). All produce `MapPath` entries. Existing roads/rivers remain untouched.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

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

### 2026-03-15
- Created `src/custom_paths.rs` with `generate_custom_paths()` and 4 styles
- Waypoints: grid coords → pixels, optional Bezier smoothing
- RoomToRoom: looks up room centers from GeneratedFeatures registry
- Offset: applies `curves::offset_polyline` at configurable distance from named parent
- Intermittent: `break_into_segments()` with random segment lengths, gaps, and optional offset
- Added `CustomPathConfig`, `PathStyle` enum with serde tagged union
- Wired into pipeline after elevation, before lights
- Added `custom_paths: Vec<CustomPathConfig>` to `MapConfig`
- 7 unit tests: waypoints, smoothing, room-to-room, offset, intermittent, missing refs, segment breaking
- All 186 tests pass