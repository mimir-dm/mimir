---
id: room-config-validation
level: task
title: "Room config validation"
short_code: "MIMIR-T-0593"
created_at: 2026-03-11T23:56:31.888668+00:00
updated_at: 2026-03-12T01:01:26.714146+00:00
parent: MIMIR-I-0060
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0060
---

# Room config validation

## Parent Initiative

[[MIMIR-I-0060]]

## Objective

Add validation rules for room and corridor declarations to `validate_config()`: room overlap detection, rooms within map bounds, corridor endpoints referencing valid room IDs and walls, portal positions within wall bounds, and duplicate room ID detection.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Rooms within map bounds: `x + width <= map.width` and `y + height <= map.height`
- [ ] Room overlap detection: no two rooms share grid cells (AABB intersection check)
- [ ] Duplicate room ID detection
- [ ] Room dimensions > 0
- [ ] Portal position within wall length: `position + portal_width <= wall_length`
- [ ] Corridor endpoints reference valid room IDs
- [ ] Corridor `from_wall`/`to_wall` reference sides that exist (wall toggle is true or wall is opened by corridor)
- [ ] Corridor width > 0
- [ ] `terrain_slot` in range 0-3 when specified
- [ ] All validation errors returned as `Vec<ValidationError>` with clear field paths (e.g., `rooms[0].portals[1].position`)
- [ ] MCP `validate_map_config` tool returns room validation errors
- [ ] Unit tests for each validation rule

## Implementation Notes

- File: `crates/mimir-mapgen/src/pipeline.rs` — extend `validate_config()` (line 119)
- AABB overlap: for each pair of rooms, check if rectangles intersect
- Field paths should be descriptive: `rooms[guard_room].portals[0].position` using room ID where possible

## Status Updates

### Completed
- Extended `validate_config()` with `validate_rooms()` and `validate_corridors()` helpers
- Room validations: bounds check (x+w <= map.width), overlap detection (AABB), duplicate ID, dimensions > 0, terrain_slot 0-3
- Portal validations: position + width <= wall_length, width > 0
- Corridor validations: from/to room IDs exist, width > 0, terrain_slot 0-3
- Error field paths use room IDs: `rooms[guard_room].portals[0].position`
- 7 validation tests: out-of-bounds, overlap, duplicate ID, portal out-of-wall, invalid corridor room ref, terrain slot range, zero dimensions
- All 118 tests pass