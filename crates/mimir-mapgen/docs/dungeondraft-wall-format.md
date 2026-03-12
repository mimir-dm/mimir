# Dungeondraft Wall & Portal Format Reference

Reverse-engineered from `.dungeondraft_map` files (JSON format, Dungeondraft v2).

## Coordinate System

- **256 pixels per grid square** — a room at grid position (4, 6) starts at pixel (1024, 1536)
- All wall points and portal positions use pixel coordinates
- Node IDs are hex strings (e.g., "b", "c", "1a") that serve as unique identifiers

## Wall Structure

Walls are polylines stored in `Level.walls[]`.

```json
{
  "points": "PoolVector2Array( x1, y1, x2, y2, x3, y3, ... )",
  "texture": "res://textures/walls/battlements.png",
  "color": "ff605f58",
  "loop": true,
  "type": 0,
  "joint": 1,
  "normalize_uv": true,
  "shadow": true,
  "node_id": "c",
  "portals": []
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `points` | PoolVector2Array | Wall polyline vertices in pixel coords (256px/grid) |
| `texture` | string | Wall texture path (`res://textures/walls/*.png`) |
| `color` | string | ARGB hex color (e.g., `"ff605f58"`) |
| `loop` | bool | `true` = closed polygon (room outline), `false` = open polyline |
| `type` | int | Wall type: `0` = standard wall |
| `joint` | int | Joint style: `1` = standard |
| `normalize_uv` | bool | UV texture normalization |
| `shadow` | bool | Whether wall casts shadows |
| `node_id` | string | Unique hex identifier |
| `portals` | array | Portals (doors/windows) embedded in this wall |

### Wall Textures (Default Pack)

- `res://textures/walls/battlements.png`
- `res://textures/walls/stone.png`
- `res://textures/walls/wood.png`
- `res://textures/walls/cave.png`

## Portal Structure

Portals represent openings in walls — doors, windows, archways. They can be:
- **Wall-attached**: stored in `wall.portals[]` with a valid `wall_id`
- **Freestanding**: stored in `level.portals[]` with `wall_id: "ffffffff"`

Portal texture is purely cosmetic — it determines visual appearance, not semantic type.

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

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `position` | Vector2 | Center point in pixel coords |
| `rotation` | f64 | Radians: `0` = horizontal (N/S wall), `±π/2` = vertical (E/W wall) |
| `scale` | Vector2 | Scale factor, typically `(1, 1)` |
| `direction` | Vector2 | Unit normal vector (outward from room) |
| `texture` | string | Portal texture path (cosmetic only) |
| `radius` | f64 | Half-width in pixels: `128` = 1 grid square, `99.5` = smaller |
| `point_index` | int | Which wall segment this portal sits on (0-indexed) |
| `wall_id` | string | Parent wall's `node_id` (`"ffffffff"` = freestanding) |
| `wall_distance` | f64 | Position along wall: integer part = segment index, fractional = position within segment |
| `closed` | bool | Door open/closed state |
| `node_id` | string | Unique hex identifier |

### Portal Textures (Default Pack)

- `res://textures/portals/door_00.png` — standard door
- `res://textures/portals/window_03.png` — standard window
- `res://textures/portals/archway_00.png` — archway
- `res://textures/portals/secret_00.png` — secret door

### Portal Rotation by Wall Orientation

| Wall Side | Rotation (rad) | Direction |
|-----------|---------------|-----------|
| North (horizontal, top) | `0.0` | `(1, 0)` or `(-1, 0)` |
| South (horizontal, bottom) | `0.0` | `(1, 0)` or `(-1, 0)` |
| East (vertical, right) | `π/2` (~1.5708) | `(0, 1)` or `(0, -1)` |
| West (vertical, left) | `-π/2` (~-1.5708) | `(0, -1)` or `(0, 1)` |

## Shapes Integration

The `Level.shapes` struct ties walls to their floor polygons:

```json
{
  "shapes": {
    "polygons": [
      "PoolVector2Array( x1, y1, x2, y2, ... )",
      "PoolVector2Array( ... )"
    ],
    "walls": [11, 12, 13]
  }
}
```

- `shapes.walls` — wall `node_id`s as **decimal integers** (hex `"b"` = `11`, hex `"c"` = `12`)
- `shapes.polygons` — corresponding room floor outlines as `PoolVector2Array`
- These define the interior regions that Dungeondraft fills with tile textures
- The arrays are parallel: `shapes.walls[i]` corresponds to `shapes.polygons[i]`

## Node ID Allocation

- Node IDs are hex strings: `"1"`, `"a"`, `"1f"`, etc.
- `world.next_node_id` tracks the next available ID
- IDs must be unique across the entire map (walls, portals, objects, paths all share the namespace)
- The existing `NodeIdAllocator` in the codebase handles this
