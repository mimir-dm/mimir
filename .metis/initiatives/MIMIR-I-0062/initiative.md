---
id: expressive-mapgen-lights-general
level: initiative
title: "Expressive mapgen: lights, general paths, and patterns for Dungeondraft"
short_code: "MIMIR-I-0062"
created_at: 2026-03-14T19:07:01.770991+00:00
updated_at: 2026-03-15T00:42:28.506021+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: L
initiative_id: expressive-mapgen-lights-general
---

# Expressive Mapgen: Lights, Paths, and Patterns

## Problem

Generated maps from mimir-mapgen are structurally complete but visually lifeless. A hand-authored Dungeondraft map has atmosphere — torchlight pooling along a forest road, shadow paths tracing cliff edges, stone floor patterns inside dungeon rooms, water ripples layered over a river. Our generated maps have none of this. The gap is not subtle: side-by-side with a hand-made DD map, a generated one looks like a schematic.

This isn't a polish problem — it's a usability ceiling. The three missing DD features (point lights, general paths, patterns) are what make a map *feel* like a place. Without them, mapgen output requires extensive manual finishing in Dungeondraft, which defeats the purpose of procedural generation.

The [Gull Rock procedural forest project](https://gitlab.com/gull-rock-maps/dungeondraft-procedural-forest) — the only other known procedural DD generator — confirms this: it generates lights at every tier (ambient, road, tree-top, perlin-gated), uses patterns for water overlays, and draws offset/intermittent paths for natural detail. These aren't extras; they're the minimum for a usable map.

## Success Criteria

This initiative succeeds when:

1. **Biome presets produce aesthetically competitive maps** — using the Gull Rock procedural forest configs as a quality reference for tuning light placement, shadow layering, water overlays, and terrain blending. The Gull Rock source configs (forest, river, swamp, island, canyon, hills) provide concrete values for densities, textures, colors, and layers that we can adapt directly into our presets.

2. **A generated dungeon map has finished interiors** — floor patterns inside rooms, torch lights along corridors, decorative paths connecting areas. The map is playable at the table without post-processing.

3. **Features compose declaratively** — a user can write `lights: [{placement: along_path, path: main_road}]` and it just works. No pixel math, no node ID management, no DD format knowledge required. The config language stays high-level and intention-driven.

4. **Existing maps don't break** — all current configs and biome presets continue to generate valid maps. New features are purely additive.

## Scope

**In scope:**
- Point light placement (scatter, path-following, object-attached)
- General-purpose path generation (waypoints, room-to-room, offset, intermittent)
- Pattern placement (water overlays, room floors, polygon fills, noise-gated regions)
- Material scatter (ice, lava, acid, ground detail — DD's `Level.materials` + `MapSpaceInfo`)
- Named ID cross-referencing between features
- Evaluate all 12 biome presets for new feature adoption

**Out of scope (phase 2):**
- Cave bitmap generation (separate algorithm)
- Roof generation (requires building detection)
- Multi-level maps
- 1:1 DD JSON schema exposure — the config stays expressive

## Detailed Design

### Point Lights

Three placement modes, all producing `MapLight` entries (format already fully typed):

- **Scatter**: Poisson disc placement gated by noise bounds + probability. Reuses existing distribution infrastructure from `objects.rs`.
- **Along path**: Samples points at regular intervals along a named road/river/elevation path.
- **With objects**: Creates a light at every placed object in a named tree/clutter/clump group.

Each mode configures: color (ARGB hex), intensity, range (grid squares), shadows, layer, margin.

### General Paths

Four styles, all producing `MapPath` entries (format already fully typed):

- **Waypoints**: User provides points in grid coordinates. Optional Bezier smoothing. For explicit decorative lines, fences, hedges.
- **Room-to-room**: Connects two rooms by ID via their centers. For interior walkways.
- **Offset**: Draws a companion path at a perpendicular distance from a named parent feature (road, river, elevation contour). For shadow paths, bank decorations, cliff edges. Reuses `curves::offset_polyline`.
- **Intermittent**: Breaks a parent path into random-length segments with gaps. For natural water flows, cracked ground, scattered detail.

### Patterns

Region-based texture overlays producing `MapPattern` entries:

- **Water region**: Covers generated water polygon area. For river/lake overlays.
- **Room region**: Fills a named room's wall boundary. For interior floor patterns.
- **Polygon region**: Fills a named polygon's boundary. For courtyard surfaces.
- **Noise region**: Fills area above/below a noise threshold via marching squares contour. For organic ground detail.

Each pattern configures: texture, color (with alpha for transparency), rotation, layer.

**Risk**: `MapPattern.data` is currently typed as `serde_json::Value`. A format spike is required — create patterns in DD, inspect the saved JSON, and properly type the struct before implementation.

### Material Scatter

DD's material scatter system places ground-level detail (ice sheets, lava pools, acid puddles, loose rocks, fallen leaves) via `Level.materials` and the `MapSpaceInfo` spatial hash. This is distinct from patterns — materials are placed as individual instances with position/rotation/scale, while patterns are tiled texture fills.

Region-based placement, similar to patterns:

- **Noise region**: Scatter materials where noise falls in a range. For organic ground detail.
- **Room/polygon region**: Fill a named area. For ice floors, lava rooms.
- **Along path**: Scatter along a river/road corridor. For riverbank debris, road-edge gravel.

Each material configures: texture, color, density, scale range, rotation.

**Risk**: `Level.materials` is `BTreeMap<String, serde_json::Value>` and `MapSpaceInfo` is initialized but never populated. Same spike approach as patterns — create materials in DD, inspect the JSON, type it.

### Named ID System

All referenceable configs gain an optional `id: String` field:
- `roads[].id`, `rivers[].id`, `rooms[].id`, `polygons[].id`, `elevation.levels[].id`
- `trees[].id`, `clutter[].id`, `clumps[].id`

Cross-references resolve by name. Fallback: `{type}_{index}` (e.g., `road_0`) when no id given. A `GeneratedFeatures` registry is populated during pipeline execution and passed to downstream stages.

## Alternatives Considered

**1:1 DD JSON passthrough** — Expose raw DD format fields in config. Rejected: defeats the purpose of an expressive abstraction. Users shouldn't need to know about node IDs, pixel coordinates, or Godot serialization.

**Defer patterns, ship lights + paths first** — Lower risk but user explicitly wants all three together. Pattern spike de-risks Phase 3.

**Index-only references (road_0, river_1)** — Simpler but fragile when config order changes. Named IDs are worth the small complexity cost.

## Implementation Plan

### Phase 0: Format Spike (patterns + materials) — COMPLETED

#### Pattern Format (from `Untitled.dungeondraft_map`)

Patterns are polygon-bounded texture fills. Each entry in `Level.patterns`:

```
position: Vector2         — offset (typically 0,0)
shape_rotation: i32       — shape rotation (0)
scale: Vector2            — scale (typically 1,1)
points: PoolVector2Array  — polygon boundary in pixel coords (256px = 1 grid sq)
layer: i32                — DD layer (100 = User Layer 1, -100 = Below Water, etc.)
color: String             — ARGB hex tint ("ff929292")
outline: bool             — outline mode (false for fills)
texture: String           — res:// path to tileset texture
rotation: i32             — texture rotation in degrees
node_id: String           — hex node ID from world.next_node_id counter
```

Example: a cobblestone floor pattern spanning a 2x2 grid square area:
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

#### Material Scatter Format

Materials are stored per-layer as bit-packed bitmaps in `Level.materials` (a `BTreeMap<String, Vec<MaterialEntry>>`). Key = layer ID string (e.g., `"-400"` = Below Ground).

Each `MaterialEntry`:
```
bitmap: PoolByteArray  — flat bit-packed placement mask
texture: String        — res:// path (acid_tile.png, ice_tile.png, lava_tile.png, etc.)
smooth: bool           — edge smoothing toggle
```

**Bitmap encoding:**
- Cell grid: `(map_width * 2 + 3) × (map_height * 2 + 3)` — each cell = 0.5 grid squares, +3 border for blend
- Flat bit-packed (NOT row-padded): bit index = `row * cell_width + col`
- Total bytes = `ceil(cell_width * cell_height / 8)`
- Example: 35×20 map → 73×43 cells → 3139 bits → 393 bytes per material
- LSB-first within each byte

Verified by decoding the acid scatter bitmap — produces a clear blob matching the painted region.

**Note:** `MapSpaceInfo` (world.msi) is already typed and initialized in `format/world.rs`. It controls DD's internal offset noise — not directly related to material bitmap encoding. No changes needed.

### Phase 0: Format Spike (patterns + materials)
- Create a `.dungeondraft_map` in DD with: floor patterns in rooms, water overlay pattern, material scatter (ice, lava, ground debris)
- Save, inspect JSON, and properly type:
  - `MapPattern.data` structure in `format/entities.rs`
  - `Level.materials` value structure in `format/mod.rs`
  - `MapSpaceInfo` fields that matter for scatter

### Phase 1-4: Lights + Paths + Patterns + Materials (parallel, independent modules)
- `src/lights.rs` — scatter, along_path, with_objects placement
- `src/custom_paths.rs` — waypoints, room_to_room, offset, intermittent
- `src/patterns.rs` — region-based pattern placement
- `src/materials.rs` — region-based material scatter
- `src/pipeline.rs` — config structs, named ID registry, wire into generate()
- `src/biomes.rs` — evaluate all 12 presets for new feature adoption, using [Gull Rock configs](https://gitlab.com/gull-rock-maps/dungeondraft-procedural-forest) (cloned to `/tmp/dd-procedural-forest/`) as aesthetic reference for light densities, shadow path layering, water overlays, terrain blending, and object color tinting

### Verification
- `cargo test -p mimir-mapgen` after each module
- Generate test maps with each feature, open in Dungeondraft to confirm they render
- `angreal test unit --core` for regression check
- End-to-end: generate a forest map and a dungeon map using all four features, verify visual quality against success criteria