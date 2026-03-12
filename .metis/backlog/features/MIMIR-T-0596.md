---
id: polygon-based-room-corridor-layout
level: task
title: "Polygon-based room/corridor layout for proper DD interior fill"
short_code: "MIMIR-T-0596"
created_at: 2026-03-12T02:38:25.612642+00:00
updated_at: 2026-03-12T02:42:41.460758+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/active"


exit_criteria_met: false
initiative_id: NULL
---

# Polygon-based room/corridor layout for proper DD interior fill

## Objective

Replace the current room-rectangle + corridor-connection wall generation with explicit polygon-based bounding areas defined in YAML. Currently, rooms are closed wall polygons (DD fills interior) but corridors are open polyline walls with no closed shape, so DD doesn't fill the corridor interior — it shows the outdoor terrain instead.

## Problem

Dungeondraft fills interior space based on closed wall polygon shapes. The current approach:
1. Each room → closed polygon wall → DD fills interior ✓
2. Corridors → open polyline walls connecting rooms → no closed shape → no interior fill ✗

Corridors show outdoor terrain even when `terrain_slot` is configured because the splat map override works, but DD's shape-based rendering overrides it.

## Proposed Solution

Allow YAML configs to define closed polygons as bounding areas. Each bounding area becomes a closed wall polygon that DD recognizes as interior space. This gives full control over:
- Irregular room shapes (not just rectangles)
- Corridors as part of a larger polygon
- L-shaped rooms, alcoves, etc.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Corridor interiors display the configured terrain texture, not outdoor terrain
- [ ] YAML config supports polygon-based area definitions
- [ ] Existing rectangular room configs continue to work
- [ ] Generated maps open correctly in Dungeondraft with proper interior fill

## Implementation Notes

### Current Architecture
- `rooms.rs`: `RoomConfig` → rectangle → `MapWall::new_room()` (closed polygon)
- Corridors → `MapWall::new_open()` (open polylines, no shape)
- Shapes are registered in `shape_wall_ids` / `shape_polygons` — corridors don't get shapes

### Key Insight
DD's interior fill is driven by the shapes system (closed polygon + wall ID), not just by walls existing. Corridors need to either be merged into the room polygon or have their own closed polygon + shape entry.

## Status Updates

### 2026-03-12: Polygon layout working

**Edge subtraction + loop chaining implemented and verified in DD.**

- `polygons.rs` rewritten: collect all edges → remove shared edges (weight > 1 → zero) → chain survivors into closed loops
- Each loop = one closed loop wall + one shape (wall points = shape polygon points)
- Single polygon: 1 wall, 1 shape — opens and fills correctly in DD
- Two adjacent polygons: shared edge removed, merged into 1 wall/shape — works
- Three rooms + two corridors (1-square-wide): all 5 input polygons merge into 1 perimeter loop — works, interior fills correctly

**DD freestanding door format discovered:**

User placed a door in DD on a corridor. Key findings for portal implementation:
- Freestanding doors are **level portals** (`lvl.portals[]`), NOT embedded in walls
- `wall_id: "ffffffff"` — sentinel meaning "not attached to any wall"
- `point_index: 0` and `wall_distance: 0` — zeroed for freestanding
- `position`: pixel coordinates of door center
- `rotation`: radians (π/2 = vertical door across horizontal corridor)
- `direction`: unit vector (e.g., `(0, 1)` pointing south)
- `radius: 128` — half-width in pixels (128 = 1 grid square door)
- `closed: true` — door starts closed
- `texture: "res://textures/portals/door_00.png"`
- `node_id`: unique hex ID

**Implication**: portal support for polygons should emit freestanding level portals, not wall-embedded portals. This is simpler — just position + rotation at corridor entrances.

**DD wall-anchored portal format (window):**

User placed a window on the north wall of room A. Key differences from freestanding:
- **Embedded in wall**: lives in `wall.portals[]`, not `lvl.portals[]`
- `wall_id: "1"` — references parent wall's node_id
- `point_index: 0` — edge index (0 = first edge of closed loop polygon)
- `wall_distance: 0.416667` — fractional position along the full perimeter (not just the edge)
- `position: Vector2(1408, 1536)` — pixel coordinates of portal center
- `rotation: 0` — 0 for horizontal wall
- `direction: Vector2(1, 0)` — unit vector along the wall
- `texture: "res://textures/portals/window_05.png"`
- `radius: 100.5` — half-width in pixels (~0.39 grid squares, smaller than door's 128)
- `closed: true`

**Two portal modes in DD:**
1. **Freestanding**: `lvl.portals[]`, `wall_id: "ffffffff"`, free-positioned (doors in corridors)
2. **Wall-anchored**: `wall.portals[]`, `wall_id` = parent wall, `point_index` = edge index, `wall_distance` = fractional position along perimeter (windows, wall-mounted doors)

### 2026-03-12: Portal generation implemented

**Two-mode portal system working:**

Portal generation in `polygons.rs` via `generate_portals()`:
1. For each polygon's portal config, compute pixel position on original edge
2. Search merged loops for a segment containing that point
3. **If found on perimeter** → wall-anchored: embedded in `wall.portals[]` with `point_index` (segment index in loop) and `wall_distance` (fractional t along segment)
4. **If not found** (shared edge removed) → freestanding: `wall_id: "ffffffff"`, positioned at correct pixel location

**Helpers added:**
- `point_on_segment()` — parametric point-on-segment test with EPS tolerance
- `edge_normal()` — computes perpendicular direction + rotation for portal orientation

**Test results (15/15 passing):**
- `portal_on_perimeter_wall_anchored` — single room, door on north wall → wall-anchored
- `portal_on_shared_edge_freestanding` — two adjacent rooms, door on shared edge → freestanding
- `portal_position_pixels` — verifies pixel coordinate accuracy
- `multiple_portals_on_different_edges` — two portals on different walls of same room
- `three_rooms_corridor_portal_freestanding` — corridor doors at room junctions → freestanding

**End-to-end verification with three-rooms layout:**
- Window on room_b north wall → wall-anchored (point_index=4, wall_distance=0.5)
- 4 corridor doors on shared edges → freestanding at correct pixel positions
- Old dead `make_polygon_portal` replaced with new two-mode system

### Remaining work
- [ ] Test opening generated map with portals in DD to verify visual correctness
- [ ] Test with more complex layouts (T-shapes, non-rectangular polygons)