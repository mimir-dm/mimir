---
id: reverse-engineer-dungeondraft-wall
level: task
title: "Reverse-engineer Dungeondraft wall and portal JSON format"
short_code: "MIMIR-T-0587"
created_at: 2026-03-11T23:56:25.180578+00:00
updated_at: 2026-03-12T00:39:53.576475+00:00
parent: MIMIR-I-0060
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0060
---

# Reverse-engineer Dungeondraft wall and portal JSON format

## Parent Initiative

[[MIMIR-I-0060]]

## Objective

Document the exact JSON structure Dungeondraft uses for walls and portals (doors/windows). Currently `MapWall` and `MapPortal` are `serde_json::Value` wrappers — we need to know every field, type, and valid value before we can generate them.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Create a Dungeondraft map with walls (straight segments, corners, different wall types) and export/inspect the JSON
- [ ] Create a Dungeondraft map with portals (doors, windows, archways) and inspect the JSON
- [ ] Document wall JSON structure: all fields, coordinate system, how segments connect, wall type identifiers
- [ ] Document portal JSON structure: all fields, how portals reference walls, portal type identifiers
- [ ] Document the relationship between `Level.walls`, `Level.portals`, and `Shapes.walls` — are they different representations?
- [ ] Save example JSON snippets as a reference file in the mapgen crate (e.g., `docs/dungeondraft-wall-format.md`)
- [ ] Identify all portal types available in Dungeondraft (door, window, archway, secret door, etc.)

## Implementation Notes

### Approach
1. Open Dungeondraft and create a small test map with various wall configurations
2. Save the `.dungeondraft_map` file (it's just JSON)
3. Parse and document the wall/portal entries
4. Pay special attention to: coordinate system (pixel vs grid), how wall segments reference each other at corners, how portals split walls

### Key Questions to Answer
- Are walls defined as line segments (point A to point B) or polylines?
- How do walls at corners connect — do they reference each other?
- What's the `Shapes.walls` vs `Level.walls` distinction?
- Do portals reference specific walls, or are they positioned independently?
- What texture/type identifiers exist for walls (stone, wood, cave, etc.)?

### Dependencies
- Requires Dungeondraft application installed
- This task blocks all other tasks in the initiative — the format must be understood before we can generate it

## Status Updates

### 2026-03-11 — Complete
- Format fully documented from `test.dungeondraft_map` analysis
- Saved reference file: `crates/mimir-mapgen/docs/dungeondraft-wall-format.md`
- All downstream tasks (T-0588, T-0590, T-0592) updated with format findings

Analyzed `test.dungeondraft_map` (35x20 grid, battlements walls, doors and windows).

#### Wall JSON Structure
```json
{
  "points": "PoolVector2Array( x1, y1, x2, y2, ... )",
  "texture": "res://textures/walls/battlements.png",
  "color": "ff605f58",
  "loop": true,
  "type": 0,
  "joint": 1,
  "normalize_uv": true,
  "shadow": true,
  "node_id": "c",
  "portals": [ /* embedded portals */ ]
}
```
- **Coordinates**: pixel coords, 256px per grid square
- **loop**: true = closed polygon, false = open polyline
- **type**: 0 = standard wall (other values TBD)
- **joint**: 1 = standard joint
- **node_id**: hex string, unique across the map
- **portals**: portals are embedded INSIDE their parent wall

#### Portal JSON Structure
```json
{
  "position": "Vector2( 2304, 2176 )",
  "rotation": -1.570796,
  "scale": "Vector2( 1, 1 )",
  "direction": "Vector2( 0, -1 )",
  "texture": "res://textures/portals/door_00.png",
  "radius": 128,
  "point_index": 15,
  "wall_id": "c",
  "wall_distance": 15.388889,
  "closed": true,
  "node_id": "f"
}
```
- **position**: center point in pixel coords
- **rotation**: radians (0 = horizontal, ±π/2 = vertical)
- **direction**: unit normal vector
- **texture**: visual appearance only — `door_00.png`, `window_03.png`, etc. Does NOT determine door vs window semantically; user confirmed freestanding "doors" used `window_03.png` texture
- **radius**: half-width in pixels (128 = 1 grid square, 99.5 = smaller opening). Determines the gap size in the wall, not the portal type
- **point_index**: which wall segment this portal sits on
- **wall_id**: parent wall's node_id
- **wall_distance**: fractional position along wall polyline (integer part = segment index, fractional = position within segment)
- **closed**: door open/closed state

#### Portal Placement
- Wall-attached portals: stored in `wall.portals[]` with valid `wall_id` and `point_index`
- Freestanding portals: stored in `level.portals[]` with `wall_id: "ffffffff"`, `point_index: 0`, `wall_distance: 0`

#### Shapes Relationship
- `Shapes.walls` = wall node_ids as **decimal integers** (node_id "b" = 11, "c" = 12, "d" = 13)
- `Shapes.polygons` = corresponding room polygons as `PoolVector2Array`
- These are the "room shapes" that define interior floor regions