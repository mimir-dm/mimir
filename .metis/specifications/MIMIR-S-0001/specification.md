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
  "ref": 12345,
  "texture": "res://textures/objects/vegetation/trees/tree_big_green_03.png",
  "position": "Vector2( 1234.56, 789.01 )",
  "rotation": 0.0,
  "scale": "Vector2( 1, 1 )",
  "mirror": false,
  "layer": 400,
  "custom_color": null,
  "block_light": false
}
```

| Field | Type | Description |
|-------|------|-------------|
| `ref` | int | Unique node ID (from `next_node_id`) |
| `texture` | string | `res://` path to asset PNG |
| `position` | Vector2 | Pixel coordinates |
| `rotation` | float | Radians |
| `scale` | Vector2 | Scale factor |
| `mirror` | bool | Horizontal flip |
| `layer` | int | Z-sorting layer (higher = on top) |
| `custom_color` | string/null | ARGB hex color override |
| `block_light` | bool | Whether object blocks dynamic lights |

## Paths

Array of drawn paths (roads, rivers, cliffs).

```json
{
  "ref": 12346,
  "texture": "res://textures/paths/cliff.png",
  "edit_points": "PoolVector2Array( x1, y1, x2, y2, ... )",
  "width": 256,
  "layer": 100,
  "fade_in": false,
  "fade_out": false,
  "grow": false,
  "shrink": false,
  "block_light": false,
  "smoothness": 0.5
}
```

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

Filled polygon regions with a texture pattern.

```json
{
  "ref": 12348,
  "polygon": "PoolVector2Array( ... )",
  "texture": "res://textures/patterns/pattern_name.png",
  "color": "ffffffff",
  "layer": 0,
  "block_light": false
}
```

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