---
id: named-id-system-config-id-fields
level: task
title: "Named ID system: config id fields and GeneratedFeatures registry"
short_code: "MIMIR-T-0628"
created_at: 2026-03-15T00:42:36.607569+00:00
updated_at: 2026-03-15T00:42:36.607569+00:00
parent: MIMIR-I-0062
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0062
---

# Named ID system: config id fields and GeneratedFeatures registry

**Blocks:** MIMIR-T-0629, MIMIR-T-0630, MIMIR-T-0631, MIMIR-T-0632

## Objective

Add optional `id` fields to all existing config structs so that downstream features (lights, paths, patterns, materials) can reference generated geometry by name. Introduce a `GeneratedFeatures` registry that is populated during pipeline execution and passed to later stages.

## Acceptance Criteria

- [ ] Optional `id: Option<String>` added to: `RoadConfig`, `RiverConfig`, `RoomConfig`, `PolygonConfig`, `ElevationLevel` (or equivalent), `TreeConfig`, `ObjectConfig` (clutter), `ClumpConfig`
- [ ] `GeneratedFeatures` struct holds: roads (name → polyline points), rivers (name → polyline + water polygon), rooms (name → center + boundary polygon), polygons (name → boundary polygon), elevation (name → contour polylines), object_positions (name → Vec of placed positions)
- [ ] Auto-naming fallback: if no `id` given, features register as `{type}_{index}` (e.g., `road_0`, `room_1`)
- [ ] Registry is populated during each pipeline stage (road gen populates roads, room gen populates rooms, etc.)
- [ ] Registry is passed to `generate()` output or threaded through stages
- [ ] Existing configs without `id` fields continue to work unchanged (backward compatible)
- [ ] All existing tests pass

## Implementation Notes

### Files to modify
- `crates/mimir-mapgen/src/pipeline.rs` — add `id` to config structs, create `GeneratedFeatures`, populate during `generate()`
- `crates/mimir-mapgen/src/rooms.rs` — return room center/boundary data for registry
- `crates/mimir-mapgen/src/paths.rs` — return road/river polyline data for registry
- `crates/mimir-mapgen/src/elevation.rs` — return contour polylines for registry
- `crates/mimir-mapgen/src/objects.rs` — return placed positions for registry
- `crates/mimir-mapgen/src/polygons.rs` — return polygon boundaries for registry

### Design
The registry is a simple `HashMap<String, FeatureGeometry>` where `FeatureGeometry` is an enum of the different geometry types (polyline, polygon, point set). Lookup by name returns an `Option` — callers handle missing references with clear error messages.

## Status Updates

*To be added during implementation*