---
id: room-layout-engine-walls-portals
level: task
title: "Room layout engine: walls, portals, and terrain override"
short_code: "MIMIR-T-0590"
created_at: 2026-03-11T23:56:30.585581+00:00
updated_at: 2026-03-12T00:51:38.501704+00:00
parent: MIMIR-I-0060
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0060
---

# Room layout engine: walls, portals, and terrain override

## Parent Initiative

[[MIMIR-I-0060]]

## Objective

Implement the core room layout engine: given a list of `RoomConfig` declarations, generate `MapWall` segments for room boundaries (with gaps for portals), `MapPortal` entries for doors/windows, and terrain override data to fill room interiors with the declared terrain slot.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `generate_room_layout()` function takes `&[RoomConfig]` and returns walls, portals, and terrain override regions
- [ ] Wall generation: each room side (where `walls.side == true`) produces `MapWall` line segments in pixel coordinates
- [ ] Portal gaps: walls split around portal positions — a wall with a door at position 2 becomes two wall segments with a gap
- [ ] Portal generation: each `PortalConfig` produces a `MapPortal` entry at the correct position and type
- [ ] Terrain override: returns a list of rectangular regions (in splat-map coordinates) with their target terrain slot
- [ ] Grid-to-pixel conversion: room grid coordinates × 256.0 (matching pipeline.rs:201)
- [ ] Unit tests: single room with 4 walls, room with portal gap, room with wall disabled on one side
- [ ] Generated map with rooms opens correctly in Dungeondraft

## Implementation Notes

- New file: `crates/mimir-mapgen/src/rooms.rs`
- Depends on MIMIR-T-0588 (typed MapWall/MapPortal) and MIMIR-T-0589 (RoomConfig schema)

### Key Design Decision: Portals Embedded in Walls
T-0587 revealed that portals live INSIDE their parent wall's `portals` array — they are NOT separate entities at the level. This means:
- Each room produces ONE `MapWall` with `loop: true` (closed polygon) containing all its portals
- Portal `wall_id` references the parent wall's `node_id`
- Portal `point_index` identifies which wall segment (0 = first segment between point 0 and point 1)
- Portal `wall_distance` = `segment_index + fraction_along_segment`

### Wall Generation Algorithm
1. Build room polygon: 4 corners in pixel coords (x*256, y*256, etc.)
2. For sides with `walls: false`, skip that side but keep the points (portals still need the polyline)
3. For each portal on a side: compute `position` (center point), `rotation` (0 for N/S walls, ±π/2 for E/W), `point_index`, `wall_distance`
4. Emit one `MapWall` with embedded portals
5. Also emit `Shapes.walls` entry (node_id as decimal int) and `Shapes.polygons` entry

### Portal Coordinate Mapping
- Door radius: 128px = half a grid square (1-grid-wide door)
- Window radius: 99.5px
- Rotation: 0 for portals on north/south walls, ±π/2 for east/west walls
- Direction: unit normal pointing outward from room

### Terrain Override
- Modify splat map bytes directly — set 4 bytes per cell to [0,0,0,255] for slot 3, [255,0,0,0] for slot 0, etc.
- Room interior region in splat coords: `(x*4, y*4)` to `((x+width)*4, (y+height)*4)`

## Status Updates

### Completed
- Added layout engine to `rooms.rs`:
  - `generate_room_layout()` — takes `&[RoomConfig]` and `&NodeIdAllocator`, returns `RoomLayoutResult`
  - `RoomLayoutResult` struct: walls, portals, shape_wall_ids, shape_polygons, terrain_overrides
  - `TerrainOverride` struct with `apply()` method for splat map modification
  - `generate_single_room()` — builds closed-polygon MapWall with embedded portals
  - `make_portal()` — converts PortalConfig to MapPortal with correct position, rotation, direction, point_index, wall_distance
- Portal coordinate mapping: North=0°, East=π/2, South=0°, West=-π/2; direction is outward normal
- Shapes integration: wall ID as decimal integer + polygon string
- 6 layout engine tests: single room with 4 walls, room with portal, terrain override, terrain apply, multiple rooms, east wall portal
- All 105 tests pass