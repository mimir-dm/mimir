---
id: corridor-generation-between-rooms
level: task
title: "Corridor generation between rooms"
short_code: "MIMIR-T-0591"
created_at: 2026-03-11T23:56:30.686349+00:00
updated_at: 2026-03-12T00:54:07.250136+00:00
parent: MIMIR-I-0060
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0060
---

# Corridor generation between rooms

## Parent Initiative

[[MIMIR-I-0060]]

## Objective

Implement corridor generation between rooms. Given a `CorridorConfig` referencing two room IDs and wall sides, generate walled corridors connecting the rooms. Corridors produce their own `MapWall` segments and terrain override for the corridor floor.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `generate_corridors()` function takes `&[CorridorConfig]` and resolved room positions, returns walls and terrain override regions
- [ ] Corridor walls: two parallel wall segments connecting the specified wall sides of two rooms
- [ ] Corridor terrain: rectangular region between rooms filled with the corridor's terrain slot
- [ ] Automatic wall gap: the connecting walls of source/destination rooms are opened where the corridor meets them
- [ ] Variable width corridors (1-3 grid squares)
- [ ] Corridor portal support: doors attached to corridor wall segments (e.g., door at the entrance to a room)
- [ ] L-shaped corridors when rooms aren't aligned (corridor turns a corner)
- [ ] Unit tests: straight horizontal corridor, straight vertical corridor, L-shaped corridor, corridor with door at entrance

## Implementation Notes

- Add to `crates/mimir-mapgen/src/rooms.rs` alongside room layout code
- Corridor routing: for aligned rooms, direct connection. For non-aligned rooms, route via an L-bend (horizontal then vertical or vice versa)
- Must coordinate with room wall generation — corridors open gaps in room walls at connection points
- Depends on MIMIR-T-0590 (room layout engine)

## Status Updates

### Completed
- Extended `generate_room_layout()` to accept `&[CorridorConfig]` alongside rooms
- Corridor types: straight horizontal, straight vertical, L-shaped (horizontal-first bend)
- Corridor walls: open polyline `MapWall` segments (2 per straight corridor, 4 per L-shaped)
- Corridor terrain: terrain override regions for corridor floors
- Corridor portals: freestanding `MapPortal` entries at corridor ends (wall_id = "ffffffff")
- Helper functions: `find_room()`, `wall_connection_point()`, `generate_corridor()`, `generate_straight_corridor_walls()`, `generate_l_corridor()`
- 4 corridor tests: straight horizontal, straight vertical, L-shaped, corridor with door
- All 109 tests pass