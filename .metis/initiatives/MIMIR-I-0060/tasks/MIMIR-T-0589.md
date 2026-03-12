---
id: room-and-corridor-config-schema-in
level: task
title: "Room and corridor config schema in MapConfig"
short_code: "MIMIR-T-0589"
created_at: 2026-03-11T23:56:30.483029+00:00
updated_at: 2026-03-12T00:49:07.704671+00:00
parent: MIMIR-I-0060
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0060
---

# Room and corridor config schema in MapConfig

## Parent Initiative

[[MIMIR-I-0060]]

## Objective

Add `rooms` and `corridors` fields to `MapConfig` with full serde support. Rooms are declared by grid position, dimensions, per-wall toggles, and portal placements. Corridors connect rooms by ID.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `RoomConfig` struct: `id` (string), `x`/`y` (u32 grid position), `width`/`height` (u32 grid squares), `terrain_slot` (Option<usize>), `walls` (per-side toggles), `portals` (vec of portal declarations)
- [ ] `CorridorConfig` struct: `from`/`to` (room ID), `from_wall`/`to_wall` (side enum), `width` (u32 grid squares), `terrain_slot` (Option<usize>), `portals` (Vec<PortalConfig> — doors at corridor ends)
- [ ] `PortalConfig` struct: `wall` (side enum), `position` (u32 offset along wall), `portal_type` (door/window/archway/secret_door), `width` (u32 grid squares)
- [ ] `WallSide` enum: North, South, East, West (with serde rename to lowercase)
- [ ] `PortalType` enum: Door, Window, Archway, SecretDoor
- [ ] `MapConfig` gains `rooms: Vec<RoomConfig>` and `corridors: Vec<CorridorConfig>` (both default empty)
- [ ] YAML deserialization works for the example config from the initiative design doc
- [ ] Existing configs without rooms continue to work (backward compatible)

## Implementation Notes

- File: `crates/mimir-mapgen/src/pipeline.rs` — add to `MapConfig` struct (line 22)
- New file: `crates/mimir-mapgen/src/rooms.rs` for room-specific types
- All coordinates in grid squares, converted to pixels internally (× 256.0, matching `pixel_width` calc at pipeline.rs:201)
- Walls default to all `true` (fully enclosed room) unless overridden

## Status Updates

### Completed
- Created `crates/mimir-mapgen/src/rooms.rs` with all config types:
  - `WallSide` enum (North/South/East/West, serde lowercase)
  - `PortalType` enum (Door/Window/Archway/SecretDoor, serde snake_case) with `default_texture()` and `default_radius()` methods
  - `PortalConfig` struct (wall, position, portal_type, width)
  - `WallToggles` struct (per-side booleans, default all true) with `is_enabled()` method
  - `RoomConfig` struct (id, x, y, width, height, terrain_slot, walls, portals)
  - `CorridorConfig` struct (from, from_wall, to, to_wall, width, terrain_slot, portals)
  - `CorridorPortalConfig` struct (end, portal_type, width)
  - `CorridorEnd` enum (From/To)
- Added `rooms` module to `lib.rs`
- Added `rooms: Vec<RoomConfig>` and `corridors: Vec<CorridorConfig>` to `MapConfig` (serde default empty)
- Updated all 3 biome presets and test helper with the new fields
- 8 unit tests for YAML deserialization: full room, minimal room, full corridor, minimal corridor, wall toggles, wall side serde, portal type serde, portal type textures
- All 98 tests pass, backward compatible with existing configs