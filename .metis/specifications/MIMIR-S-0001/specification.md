---
id: dungeondraft-map-format
level: specification
title: "Dungeondraft Map Format Specification"
short_code: "MIMIR-S-0001"
created_at: 2026-03-11T21:11:41.472262+00:00
updated_at: 2026-03-11T21:11:41.472262+00:00
parent: MIMIR-I-0058
blocked_by: []
archived: false

tags:
  - "#specification"
  - "#phase/discovery"


exit_criteria_met: false
initiative_id: NULL
---

# Dungeondraft Map Format Specification

Reverse-engineered specification of the `.dungeondraft_map` JSON format used by Dungeondraft (Godot-based map editor). This is the target output format for `mimir-mapgen`.

## Overview

A `.dungeondraft_map` file is a JSON document with two top-level keys: `header` and `world`. The format version is currently `3`. All coordinates use Dungeondraft's pixel system where **256 pixels = 1 grid square**. Godot types are serialized as strings (e.g., `"Vector2( 100, 200 )"`).

## Top-Level Structure

```json
{
  "header": { ... },
  "world": { ... }
}
```

## Header

Map metadata and asset pack manifest.

```json
{
  "creation_build": "1.0.0.0 prehistoric centipede",
  "creation_date": { "year": 2021, "month": 3, "day": 23, "weekday": 2, "dst": false, "hour": 14, "minute": 58, "second": 59 },
  "uses_default_assets": true,
  "asset_manifest": [ ... ],
  "editor_state": { ... }
}
```

### Asset Manifest

Array of pack references. Each entry declares a third-party asset pack used by the map:

```json
{
  "name": "Pack Display Name",
  "id": "XpMDQfBe",
  "version": "1.0",
  "author": "Author Name",
  "keywords": null,
  "allow_3rd_party_mapping_software_to_read": false,
  "custom_color_overrides": {
    "enabled": false,
    "min_redness": 0.1,
    "min_saturation": 0,
    "red_tolerance": 0.04
  }
}
```

The `id` is an 8-character alphanumeric string used in `res://packs/{id}/...` resource paths.

### Editor State

Preserves UI state (camera position/zoom, color palettes, grid colors, water colors). Not required for map generation but should be populated with sensible defaults.

Key palettes: `object_custom_colors`, `scatter_custom_colors`, `light_colors`, `grid_colors`, `deep_water_colors`, `shallow_water_colors`, `cave_ground_colors`.

## World

Contains map dimensions, settings, and level data.

```json
{
  "format": 3,
  "width": 32,
  "height": 32,
  "next_node_id": 36,
  "next_prefab_id": 0,
  "msi": { "offset_map_size": 512, "max_offset_distance": 0.2, "cell_size": 64, "seed": "605a0233" },
  "grid": { "color": "7f000000", "texture": "res://textures/grid/dotted_line.png" },
  "building_wear": null,
  "wall_shadow": true,
  "object_shadow": false,
  "trace_image_visible": false,
  "embedded": {},
  "levels": { "0": { ... }, "1": { ... } }
}
```

### Key World Fields

| Field | Type | Description |
|-------|------|-------------|
| `format` | int | Format version (currently 3) |
| `width`, `height` | int | Map size in grid squares |
| `next_node_id` | int | Auto-incrementing ID counter for all placed entities |
| `next_prefab_id` | int | Auto-incrementing ID for prefabs |
| `msi` | object | Material shader info (offset map params) |
| `grid` | object | Grid display settings |
| `wall_shadow` | bool | Global wall shadow toggle |
| `object_shadow` | bool | Global object shadow toggle |
| `embedded` | object | Embedded asset data (usually empty) |

### Coordinate System

- **Pixels**: All positions are in pixels. 256px = 1 grid square.
- **Vector2**: Serialized as `"Vector2( x, y )"` with spaces after `(` and before `)`.
- **Origin**: Top-left of the map canvas.
- **Map pixel dimensions**: `width * 256` x `height * 256` (e.g., 32x32 map = 8192x8192 pixels).

## Levels

Maps support multiple levels (0 = ground, 1 = canopy/upper). Each level contains:

```json
{
  "label": "",
  "environment": { "baked_lighting": true, "ambient_light": "ffffffff" },
  "layers": [],
  "shapes": [],
  "tiles": [],
  "patterns": [],
  "walls": [],
  "portals": [],
  "cave": { "bitmap": "PoolByteArray( ... )" },
  "terrain": { ... },
  "water": { ... },
  "materials": [],
  "paths": [],
  "objects": [],
  "lights": [],
  "roofs": { "shade": true, "shade_contrast": 0.5, "sun_direction": 45, "roofs": [] },
  "texts": []
}
```

## Terrain

4-texture blending system using a splat map.

```json
{
  "enabled": true,
  "expand_slots": false,
  "smooth_blending": false,
  "texture_1": "res://textures/terrain/terrain_dirt.png",
  "texture_2": "res://textures/terrain/terrain_dry_grass.png",
  "texture_3": "res://textures/terrain/terrain_moss.png",
  "texture_4": "res://textures/terrain/terrain_gravel.png",
  "splat": "PoolByteArray( 255, 0, 0, 0, 255, 0, 0, 0, ... )"
}
```

### Splat Map Encoding

- Serialized as `PoolByteArray( b0, b1, b2, b3, ... )`
- 4 bytes per terrain cell: one weight per texture slot (0-255)
- Cell resolution: **16 cells per grid square** (4x4 subdivision)
- Total cells: `(width * 4) * (height * 4)`
- Total bytes: `width * 4 * height * 4 * 4`
- Byte order: R=texture_1, G=texture_2, B=texture_3, A=texture_4
- Weights should sum to ~255 for each cell (blending)
- When `expand_slots` is true, 8 texture slots with a second splat map

## Objects (Props)

Array of placed objects/sprites.

```json
{
  "position": "Vector2( 1234.56, 789.01 )",
  "rotation": 0.0,
  "scale": "Vector2( 1, 1 )",
  "mirror": false,
  "texture": "res://textures/objects/vegetation/trees/tree_big_green_03.png",
  "layer": 400,
  "shadow": true,
  "block_light": false,
  "node_id": "6"
}
```

With optional fields (custom color, prefab):
```json
{
  "position": "Vector2( -455.505, 457.585 )",
  "rotation": 1.047198,
  "scale": "Vector2( 0.5, 0.5 )",
  "mirror": false,
  "texture": "res://textures/objects/vegetation/flowers/flowers_01.png",
  "layer": 200,
  "shadow": false,
  "block_light": false,
  "custom_color": "ff809dab",
  "node_id": "32b",
  "prefab_id": 14
}
```

| Field | Type | Description |
|-------|------|-------------|
| `position` | Vector2 | Pixel coordinates (can be negative for off-canvas objects) |
| `rotation` | float | Rotation in radians |
| `scale` | Vector2 | Scale factor (negative X = horizontal mirror) |
| `mirror` | bool | Horizontal flip flag |
| `texture` | string | `res://` path to asset PNG |
| `layer` | int | Z-sorting layer (higher = on top) |
| `shadow` | bool | Whether object casts shadow |
| `block_light` | bool | Whether object blocks dynamic lights |
| `custom_color` | string (optional) | ARGB hex color tint (omitted if none) |
| `node_id` | string | Hex node ID from `next_node_id` counter |
| `prefab_id` | int (optional) | Groups objects from same prefab placement |

*Note: Mirror can be achieved via `mirror: true` OR negative X scale (e.g., `Vector2( -0.5, 0.5 )`). Both appear in DD output. Verified from spike map (2026-03-14).*

## Paths

Array of drawn paths (roads, rivers, cliffs, decorative lines). Coordinates in `edit_points` are **relative to `position`**.

```json
{
  "position": "Vector2( 2048, 768 )",
  "rotation": 0,
  "scale": "Vector2( 1, 1 )",
  "edit_points": "PoolVector2Array( -256, 0, 3328, 0, 3584, 256, 3968, 384 )",
  "smoothness": 1,
  "texture": "res://textures/paths/battlements.png",
  "width": 69,
  "layer": 100,
  "fade_in": false,
  "fade_out": false,
  "grow": false,
  "shrink": false,
  "block_light": false,
  "loop": true,
  "node_id": "3"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `position` | Vector2 | Origin point in pixel coords |
| `rotation` | int/float | Path rotation (radians) |
| `scale` | Vector2 | Scale factor |
| `edit_points` | PoolVector2Array | Control points relative to `position` |
| `smoothness` | float | Bezier smoothing (0 = angular, 1 = full smooth) |
| `texture` | string | `res://` path to path texture |
| `width` | float | Path width in pixels |
| `layer` | int | DD layer |
| `fade_in` | bool | Fade at start |
| `fade_out` | bool | Fade at end |
| `grow` | bool | Width grows from start |
| `shrink` | bool | Width shrinks to end |
| `block_light` | bool | Blocks dynamic light |
| `loop` | bool | Closed path (last point connects to first) |
| `node_id` | string | Hex node ID |

*Verified from DD spike map (2026-03-14) and [Dungeondraft Modding API Pathway class](https://megasploot.github.io/DungeondraftModdingAPI/reference/Pathway/). Note: `edit_points` are relative to `position`, not absolute.*

### PoolVector2Array Encoding

Serialized as `PoolVector2Array( x1, y1, x2, y2, x3, y3, ... )` — flat list of alternating x,y coordinates.

## Water

Tree structure of water polygons with nesting for islands/holes.

```json
{
  "disable_border": false,
  "tree": {
    "ref": -2058744202,
    "polygon": "PoolVector2Array( x1, y1, x2, y2, ... )",
    "join": 0,
    "end": 0,
    "is_open": false,
    "deep_color": "ff3aa19a",
    "shallow_color": "ff3ac3b2",
    "blend_distance": 2.0,
    "children": []
  }
}
```

Children can contain nested water polygons (for islands within lakes).

## Lights

```json
{
  "ref": 12347,
  "position": "Vector2( 500, 600 )",
  "color": "ffeaefca",
  "intensity": 0.6,
  "range": 384,
  "shadows": false
}
```

Color is ARGB hex (alpha first). Range is in pixels.

## Patterns

Polygon-bounded texture fills. Used for floor tiles inside rooms, water overlays, ground detail. Each entry in `Level.patterns`:

```json
{
  "position": "Vector2( 0, 0 )",
  "shape_rotation": 0,
  "scale": "Vector2( 1, 1 )",
  "points": "PoolVector2Array( 2304, 1792, 2816, 1792, 2816, 2304, 2304, 2304 )",
  "layer": 100,
  "color": "ff929292",
  "outline": false,
  "texture": "res://textures/tilesets/simple/tileset_cobble.png",
  "rotation": 0,
  "node_id": "0"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `position` | Vector2 | Position offset (typically 0,0) |
| `shape_rotation` | int | Shape rotation (0) |
| `scale` | Vector2 | Scale factor (typically 1,1) |
| `points` | PoolVector2Array | Polygon boundary in pixel coords |
| `layer` | int | DD layer (100, -100, etc.) |
| `color` | string | ARGB hex tint |
| `outline` | bool | Outline-only mode (false for fills) |
| `texture` | string | `res://` path to tileset texture |
| `rotation` | int | Texture rotation in degrees |
| `node_id` | string | Hex node ID from `next_node_id` counter |

*Verified from hand-authored DD map spike (2026-03-14).*

## Materials (Scatter)

Ground-level material scatter (ice, lava, acid, etc.) stored as bit-packed bitmaps per layer. `Level.materials` is a `BTreeMap<String, Vec<MaterialEntry>>` keyed by layer ID string (e.g., `"-400"` = Below Ground).

```json
{
  "materials": {
    "-400": [
      {
        "bitmap": "PoolByteArray( 0, 0, 0, ... )",
        "texture": "res://textures/materials/acid_tile.png",
        "smooth": true
      },
      {
        "bitmap": "PoolByteArray( 0, 0, 0, ... )",
        "texture": "res://textures/materials/ice_tile.png",
        "smooth": true
      },
      {
        "bitmap": "PoolByteArray( 0, 0, 0, ... )",
        "texture": "res://textures/materials/lava_tile.png",
        "smooth": true
      }
    ]
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `bitmap` | PoolByteArray | Flat bit-packed placement mask |
| `texture` | string | `res://` path to material texture |
| `smooth` | bool | Edge smoothing toggle |

### Bitmap Encoding

- **Cell grid**: `(map_width × 2 + 3) × (map_height × 2 + 3)` — each cell = 0.5 grid squares, +3 cells border for blend region
- **Flat bit-packed** (NOT row-padded): bit index = `row × cell_width + col`
- **Total bytes**: `ceil(cell_width × cell_height / 8)`
- **Bit order**: LSB-first within each byte
- **Example**: 35×20 map → 73×43 cells → 3139 bits → 393 bytes per material

All materials in the same layer share the same bitmap dimensions. A `1` bit means the material is present at that cell; `0` means absent.

*Verified by decoding acid scatter bitmap from DD spike map — produces correct blob shape matching the painted region (2026-03-14). Materials are managed via `MaterialMesh` objects per the [Dungeondraft Modding API Level class](https://megasploot.github.io/DungeondraftModdingAPI/reference/Level/), keyed by layer + texture with `GetOrMakeMaterialMesh(layer, texture, smoothDefault)`.*

## Godot Type Serialization

| Type | Format | Example |
|------|--------|---------|
| Vector2 | `Vector2( x, y )` | `"Vector2( 1234.56, 789.01 )"` |
| PoolByteArray | `PoolByteArray( b0, b1, ... )` | `"PoolByteArray( 255, 0, 0, 0 )"` |
| PoolVector2Array | `PoolVector2Array( x1, y1, x2, y2, ... )` | `"PoolVector2Array( 100, 200, 300, 400 )"` |
| Color | `AARRGGBB` hex string | `"ff3aa19a"` |
| null | `"null"` or JSON null | `"null"` or `null` |

**Important**: Spaces inside the parentheses matter for Dungeondraft compatibility. Always use `Vector2( x, y )` with spaces, not `Vector2(x,y)`.

## Constraints

- Map size is in grid squares (typical range: 10-140)
- `next_node_id` must be strictly incrementing and unique across all entities
- Objects, paths, lights, patterns, and water nodes all consume `ref` IDs from the same counter
- Splat map size must exactly match `width * 4 * height * 4 * 4` bytes
- Asset paths must use forward slashes and the `res://` prefix
- The format is Godot-native JSON — floats may be written as integers where the value is whole