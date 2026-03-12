---
id: fully-type-mapwall-and-mapportal
level: task
title: "Fully type MapWall and MapPortal structs"
short_code: "MIMIR-T-0588"
created_at: 2026-03-11T23:56:26.503188+00:00
updated_at: 2026-03-12T00:46:41.845633+00:00
parent: MIMIR-I-0060
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0060
---

# Fully type MapWall and MapPortal structs

## Parent Initiative

[[MIMIR-I-0060]]

## Objective

Replace the `serde_json::Value` wrappers in `MapWall` and `MapPortal` with fully typed Rust structs that match Dungeondraft's actual JSON format. This enables the generation pipeline to construct walls and portals programmatically rather than passing through opaque JSON.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `MapWall` struct with typed fields matching Dungeondraft's JSON (based on MIMIR-T-0587 findings)
- [ ] `MapPortal` struct with typed fields for door/window/archway types
- [ ] Builder methods on both structs (similar to `MapObject::new()`, `.with_layer()`, etc.)
- [ ] Existing serialization round-trip tests pass — maps with walls/portals deserialize and re-serialize correctly
- [ ] New unit tests for constructing walls and portals programmatically

## Implementation Notes

- File: `crates/mimir-mapgen/src/format/entities.rs` (lines 136-148)
- Follow the pattern of `MapObject` and `MapPath` — typed fields with builder methods

### MapWall Fields (from T-0587 analysis)
```rust
pub struct MapWall {
    pub points: PoolVector2Array,    // wall polyline in pixel coords (256px/grid)
    pub texture: String,             // e.g., "res://textures/walls/battlements.png"
    pub color: String,               // ARGB hex, e.g., "ff605f58"
    pub loop_path: bool,             // true = closed polygon (serde rename "loop")
    pub wall_type: i32,              // 0 = standard wall (serde rename "type")
    pub joint: i32,                  // 1 = standard joint
    pub normalize_uv: bool,
    pub shadow: bool,
    pub node_id: String,             // hex string, unique ID
    pub portals: Vec<MapPortal>,     // portals embedded in this wall
}
```

### MapPortal Fields (from T-0587 analysis)
```rust
pub struct MapPortal {
    pub position: Vector2,           // center point in pixel coords
    pub rotation: f64,               // radians (0 = horizontal, ±π/2 = vertical)
    pub scale: Vector2,              // typically (1, 1)
    pub direction: Vector2,          // unit normal vector
    pub texture: String,             // e.g., "res://textures/portals/door_00.png"
    pub radius: f64,                 // half-width in px (128 = 1-grid door, 99.5 = window)
    pub point_index: i32,            // which wall segment this sits on
    pub wall_id: String,             // parent wall's node_id ("ffffffff" = freestanding)
    pub wall_distance: f64,          // fractional position along wall polyline
    pub closed: bool,                // door open/closed state
    pub node_id: String,             // hex string, unique ID
}
```

### Shapes.walls Integration
- `Shapes.walls` holds wall `node_id`s as **decimal integers** (hex "b" = 11)
- `Shapes.polygons` holds corresponding room polygon outlines
- When adding walls, must also update `Shapes.walls` and `Shapes.polygons`

## Status Updates

### Completed
- Replaced `MapWall` serde_json::Value wrapper with fully typed struct in `entities.rs`
  - Fields: points, texture, color, is_loop (`#[serde(rename = "loop")]`), wall_type (`#[serde(rename = "type")]`), joint, normalize_uv, shadow, node_id, portals
  - Builder methods: `new_room()`, `new_open()`, `with_color()`, `with_portals()`
- Replaced `MapPortal` serde_json::Value wrapper with fully typed struct
  - Fields: position, rotation, scale, direction, texture, radius, point_index, wall_id, wall_distance, closed, node_id
  - Builder methods: `new()`, `new_freestanding()`, `with_closed()`
- Added `test_wall_portal_roundtrip` test — parses test.dungeondraft_map, verifies 3 walls, 4 embedded portals, 3 freestanding portals, full round-trip
- Added `test_wall_builder` test — programmatic construction and JSON round-trip
- Fixed `Water.tree` from `WaterTree` to `Option<WaterTree>` to handle maps without water tree data
- Updated `Level::new_ground()`, `pipeline.rs`, and `water.rs` for the Option change
- All 90 mimir-mapgen tests pass