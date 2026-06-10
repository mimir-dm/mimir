# Mapgen Reference

Technical reference for `mimir-mapgen` — Mimir's procedural map generator that outputs Dungeondraft-compatible `.dungeondraft_map` files.

## CLI Usage

```bash
mimir-mapgen <command>
```

### Commands

| Command | Description |
|---------|-------------|
| `generate` | Generate a `.dungeondraft_map` from a YAML config or biome preset |
| `validate` | Validate a YAML config without generating |
| `list-presets` | List available biome presets |

### Generate

```bash
# From a YAML config file
mimir-mapgen generate config.yaml -o output.dungeondraft_map

# From a biome preset
mimir-mapgen generate --preset forest -o output.dungeondraft_map

# With a random seed override
mimir-mapgen generate config.yaml --seed 42 -o output.dungeondraft_map
```

| Flag | Description |
|------|-------------|
| `-o`, `--output` | Output file path (default: `output.dungeondraft_map`) |
| `-s`, `--seed` | Random seed override (takes precedence over config file seed) |
| `-p`, `--preset` | Generate from a biome preset instead of a config file |

You must provide either a config file path or `--preset`, but not both.

### Validate

```bash
mimir-mapgen validate config.yaml
```

Reports validation errors without generating a map. Useful for checking configs before generation.

### List Presets

```bash
mimir-mapgen list-presets
```

## Biome Presets

12 built-in presets with sensible defaults for terrain, objects, and noise parameters.

| Preset | Size | Description |
|--------|------|-------------|
| `forest` | 32×32 | Dense temperate forest with dirt paths, scattered rocks, and natural clearings |
| `grassland` | 32×32 | Open rolling hills with sparse trees and wildflowers |
| `cave` | 24×24 | Underground cavern with rocky terrain and dark ambient lighting |
| `desert` | 32×32 | Arid sandy wasteland with rocky outcrops and sparse scrub |
| `lake` | 32×32 | Tranquil woodland pond with grassy shores and scattered trees |
| `ice_lake` | 32×32 | Frozen lake covered in solid ice, with snow-covered shores |
| `arctic` | 32×32 | Frozen tundra with snow drifts, exposed rock, and harsh conditions |
| `island_tropical` | 32×32 | Tropical island with sandy beaches, palm trees, and warm ocean |
| `island_forest` | 32×32 | Forested island in a lake with dirt shores and dense tree cover |
| `island_arctic` | 32×32 | Snow-covered island surrounded by frigid dark water |
| `swamp` | 32×32 | Dark, murky wetland with stagnant water, dead trees, and dim lighting |
| `forest_river` | 32×32 | Dense forest bisected by a meandering river with rocky banks |

Some presets accept alternate names: `lake`/`pond`, `swamp`/`marsh`/`bog`, `island_tropical`/`tropical_island` (hyphens also accepted).

## YAML Configuration Schema

### Top-Level Fields

```yaml
name: "My Forest Map"          # string, required — map display name
width: 32                       # u32, required — width in grid squares
height: 32                      # u32, required — height in grid squares
seed: 42                        # u64, optional — random seed
```

### Noise

Controls Perlin noise that drives terrain blending, object placement, and feature generation.

```yaml
noise:
  seed: 0              # u32 — noise seed (separate from map seed), default 0
  octaves: 6           # usize — noise layers (more = more detail), default 6
  persistence: 0.5     # f64 — amplitude decay per octave (0–1), default 0.5
  lacunarity: 2.0      # f64 — frequency multiplier per octave, default 2.0
  scale: 0.01          # f64 — base frequency (lower = larger features), default 0.01
```

### Island Mode

Optional radial falloff creating island/clearing shapes where the center is low and edges are high.

```yaml
island_mode: 1.5       # f64, optional — falloff strength (higher = steeper)
```

### Terrain

Defines 4 terrain texture slots and their noise-based blending.

```yaml
terrain:
  slots:                        # exactly 4 slots required
    - texture: "res://textures/terrain/terrain_dirt.png"
      lower: 0.0                # f64 — noise range lower bound
      upper: 0.3                # f64 — noise range upper bound
    - texture: "res://textures/terrain/terrain_grass.png"
      lower: 0.25
      upper: 0.55
    - texture: "res://textures/terrain/terrain_moss.png"
      lower: 0.5
      upper: 0.8
    - texture: "res://textures/terrain/terrain_gravel.png"
      lower: 0.75
      upper: 1.0
  blend_width: 0.05            # f64 — overlap width for blending
  smooth_blending: false        # bool — smooth interpolation vs hard boundaries
```

Slots should have overlapping `lower`/`upper` ranges for natural blending. Each slot's weight is determined by how well the noise value falls within its range.

### Trees

Array of tree configurations with optional shadow and canopy layers.

```yaml
trees:
  - id: "oaks"                           # string, optional — name for cross-referencing (e.g., from lights)
    tree:
      textures:                          # list of textures (randomly selected)
        - "res://textures/objects/trees/tree_01.png"
        - "res://textures/objects/trees/tree_02.png"
      min_distance: 180.0                # f64 — minimum spacing (pixels)
      noise_lower: 0.3                   # f64 — place only where noise > this
      noise_upper: 0.8                   # f64 — place only where noise < this
      probability: 0.8                   # f64 — placement chance (0–1)
      scale_min: 0.8                     # f64 — minimum random scale
      scale_max: 1.4                     # f64 — maximum random scale
      layer: 300                         # i32 — Dungeondraft render layer
      random_rotation: true              # bool — randomize rotation
      random_mirror: true                # bool — randomly flip horizontally
      custom_color: null                 # string, optional — ARGB hex override
    shadow:                              # optional shadow beneath tree
      texture: "res://textures/objects/trees/tree_shadow.png"
      offset: "Vector2( 10, 15 )"        # Vector2 — offset from tree position in pixels (Godot string form)
      scale_factor: 1.2                  # f64 — scale relative to tree
      layer: 50                          # i32 — render layer (below tree)
    canopy:                              # optional canopy placed on level 1 (overhead foliage)
      texture: "res://textures/objects/trees/tree_canopy.png"
      scale_factor: 1.5                  # f64 — scale relative to tree
      layer: 500                         # i32 — render layer (above tree)
```

All fields of the inner `tree` object block are required except `id` and `custom_color`. The shadow `offset` is a Vector2, written in Godot's string form: `"Vector2( x, y )"`.

### Clutter

Small decorative objects scattered across the map.

```yaml
clutter:
  - id: "grass"                          # string, optional — name for cross-referencing
    textures:
      - "res://textures/objects/grass/grass_01.png"
    min_distance: 80.0
    noise_lower: 0.2
    noise_upper: 0.7
    probability: 0.6
    scale_min: 0.5
    scale_max: 1.0
    layer: 100
    random_rotation: true
    random_mirror: false
    custom_color: null
```

Each clutter entry is an object config with the same fields as the `tree` block above; all fields are required except `id` and `custom_color`.

### Clumps

Clustered object groups (rock formations, flower patches).

```yaml
clumps:
  - id: "rock_piles"                     # string, optional — name for cross-referencing
    primary:                             # main object (placed via Poisson Disc)
      textures:
        - "res://textures/objects/rocks/rock_01.png"
      min_distance: 200.0
      noise_lower: 0.4
      noise_upper: 0.9
      probability: 0.5
      scale_min: 0.8
      scale_max: 1.2
      layer: 150
      random_rotation: true
      random_mirror: false
    secondary:                           # scattered around each primary
      textures:
        - "res://textures/objects/rocks/pebble_01.png"
      min_distance: 30.0
      noise_lower: 0.0
      noise_upper: 1.0
      probability: 1.0
      scale_min: 0.3
      scale_max: 0.6
      layer: 100
      random_rotation: true
      random_mirror: false
    secondary_count: [3, 6]              # [u32, u32] — min/max secondaries per primary
    secondary_radius: 100.0              # f64 — max distance of secondaries from primary (pixels)
```

`primary` and `secondary` are both full object configs (same fields and requirements as the `tree` block above).

### Roads

Road paths using noise-following walk (`straight`) or sinusoidal meander (`meandering`).

```yaml
roads:
  - id: "main_road"                      # string, optional — name for cross-referencing (lights, custom paths)
    from: Left                           # Edge — start: Left, Right, Top, Bottom (default Left)
    to: Right                            # Edge — end side (default Right)
    texture: "res://textures/paths/path_dirt.png"   # default "res://textures/paths/path_dirt.png"
    width: 512.0                         # f64 — road width in pixels (512 = 2 grid squares), default 512.0
    layer: 100                           # i32 — render layer, default 100
    style: straight                      # PathStyle — "straight" or "meandering", default straight
    step_distance: 64.0                  # f64 — walk step size (pixels), default 64.0
    fov: 1.047                           # f64 — field of view angle (radians), default π/3
    noise_weight: 0.5                    # f64 — 0=all progress, 1=all noise, default 0.5
    margin: 128.0                        # f64 — margin from map edge (pixels), default 128.0
    smooth_density: 8                    # usize — Bezier smoothing density, default 8
    effort: 0.5                          # f64 — contour crossing: 0.0 avoids contours, 1.0 ignores them, default 0.5
    edge_paths:                          # optional — border textures along road sides
      texture: "res://textures/paths/path_edge.png"
      offset: 20.0                       # f64 — perpendicular offset from center
      width: 8.0                         # f64 — edge path width
      layer: 190                         # i32 — render layer
```

All road fields have defaults, so `roads: [{}]` produces a valid default road.

**Path styles:**
- `straight` — greedy walk following noise ridges/valleys. Roughly straight with gentle bends.
- `meandering` — sinusoidal S-curves. More natural for rivers and winding roads.

Roads automatically route around rooms and corridors.

### Rivers

River paths with water polygon and bank paths. Default style is `meandering`.

```yaml
rivers:
  - id: "east_river"                     # string, optional — name for cross-referencing
    from: Top                            # Edge — start side (default Top)
    to: Bottom                           # Edge — end side (default Bottom)
    width: 768.0                         # f64 — water width (pixels), default 768.0
    style: meandering                    # PathStyle — "straight" or "meandering", default meandering
    step_distance: 64.0                  # f64 — walk step size, default 64.0
    fov: 1.047                           # f64 — FOV for straight style, default π/3
    noise_weight: 0.5                    # f64 — noise following weight, default 0.5
    margin: 128.0                        # f64 — margin from map edge, default 128.0
    smooth_density: 8                    # usize — Bezier smoothing, default 8
    deep_color: "ff3aa19a"               # string — ARGB hex for deep water, default "ff3aa19a"
    shallow_color: "ff3ac3b2"            # string — ARGB hex for shallow water, default "ff3ac3b2"
    bank_texture: "res://textures/paths/path_rocks.png"  # default "res://textures/paths/path_rocks.png"
    bank_width: 256.0                    # f64 — bank path width, default 256.0
    bank_layer: 100                      # i32 — bank render layer, default 100
    effort: 0.5                          # f64 — contour crossing: 0.0 follows valleys, 1.0 ignores contours, default 0.5
    source: null                         # string, optional — lake id where the river starts (overrides `from`)
    drain: null                          # string, optional — lake id where the river ends (overrides `to`)
```

Rivers generate a water polygon between two banks plus bank paths on each side. Very wide rivers (1000+) can simulate coastlines. Rivers automatically route around rooms and corridors.

`source` and `drain` connect a river to declarative lakes (see [Lakes](#lakes)) by their `id`. When set, the river starts/ends at the lake shoreline instead of a map edge.

### Water Bodies

Noise-threshold water generation for lakes and ponds.

```yaml
water:
  threshold: 0.75                        # f64 — noise values above this become water
  deep_color: "ff3aa19a"                 # string — ARGB hex
  shallow_color: "ff3ac3b2"              # string — ARGB hex
  blend_distance: 40.0                   # f64 — edge blend distance
  min_contour_points: 6                  # usize — minimum polygon size
  smooth_iterations: 2                   # usize — edge smoothing passes
  pixels_per_cell: 64.0                  # f64 — coordinate scaling
  disable_border: false                  # bool — disable water border effect
```

### Lakes

Declarative lakes placed by center and radius, with a noise-perturbed organic shoreline. The noise map is depressed inside the lake to form a basin.

```yaml
lakes:
  - id: "north_lake"                     # string, required — unique id (referenced by river source/drain)
    center: [16, 10]                     # [f64, f64], required — center in grid squares
    radius: 5.0                          # f64, required — approximate radius in grid squares
    roughness: 0.4                       # f64 — shoreline jaggedness 0.0–1.0, default 0.4
    deep_color: "ff2a7f6f"               # string — ARGB hex, default "ff2a7f6f"
    shallow_color: "ff3ac3b2"            # string — ARGB hex, default "ff3ac3b2"
    blend_distance: 3.0                  # f64 — water edge blend distance, default 3.0
```

### Elevation Contours

Cliff and hill contour lines at noise thresholds.

```yaml
elevation:
  levels:
    - threshold: 0.65                    # f64 — noise threshold for this contour
      texture: "res://textures/paths/path_rocks.png"
      width: 12.0                        # f64 — contour line width
      layer: 100                         # i32 — render layer
      min_points: 8                      # usize — minimum contour length
      smooth_iterations: 2              # usize — smoothing passes
      shadow:                            # optional shadow below contour
        texture: "res://textures/paths/path_rocks.png"
        offset: 8.0                      # f64 — perpendicular offset
        width: 16.0                      # f64 — shadow width
        layer: 50                        # i32 — shadow layer
  pixels_per_cell: 64.0                  # f64 — coordinate scaling
```

### Rooms

Rectangular rooms with walls, doors, and terrain overrides. Outdoor features (trees, clutter, roads) automatically route around rooms.

```yaml
rooms:
  - id: "guard_room"              # string, required — unique identifier
    x: 4                          # u32 — grid X position (left edge)
    y: 6                          # u32 — grid Y position (top edge)
    width: 5                      # u32 — width in grid squares
    height: 4                     # u32 — height in grid squares
    terrain_slot: 3               # usize, optional — terrain texture slot (0–3)
    walls:                        # optional — per-wall toggles (default: all true)
      north: true
      south: true
      east: true
      west: false                 # open side
    portals:                      # optional — doors/windows on walls
      - wall: "north"            # WallSide — north, south, east, west
        position: 2              # u32 — grid offset along wall from start
        type: "door"             # PortalType — door, window, archway, secret_door
        width: 1                 # u32 — portal width in grid squares
```

All coordinates are in grid squares (256 pixels per grid square internally). Rooms must not overlap or extend beyond map boundaries.

### Corridors

Connections between rooms with parallel wall segments.

```yaml
corridors:
  - from: "guard_room"           # string — source room ID
    from_wall: "east"            # WallSide — exit wall of source
    to: "throne_room"            # string — destination room ID
    to_wall: "west"              # WallSide — entry wall of destination
    width: 2                     # u32 — width in grid squares
    terrain_slot: 3              # usize, optional — floor terrain slot
    portals:                     # optional — doors at corridor ends
      - end: "from"             # CorridorEnd — "from" or "to"
        type: "door"            # PortalType
        width: 1                # u32 — portal width
```

Corridors are straight when rooms are aligned on the connecting axis; L-shaped otherwise.

### Polygons

Arbitrary closed shapes defined by vertices in grid coordinates. Use for irregular rooms, oval chambers, or L-shaped areas.

```yaml
polygons:
  - id: "room_a"                   # string, required — unique identifier
    points:                        # list of [x, y] vertices tracing the perimeter in order
      - [3, 6]
      - [9, 6]
      - [9, 14]
      - [3, 14]
    terrain_slot: 3                # usize, optional — terrain texture slot (0–3)
    wall_texture: "res://textures/walls/battlements.png"  # optional
    portals:
      - edge: 0                   # usize — 0-indexed segment (edge 0 = points[0]→points[1])
        position: 0.5             # f64 — fractional position along edge (0.0–1.0)
        type: door                # PortalType — door, window, archway, secret_door
```

Vertices may be listed clockwise or counter-clockwise; validation only rejects self-intersecting outlines and degenerate (zero-length) edges, plus polygons with fewer than 3 vertices.

**Adjacency:** When two polygons share an edge, the shared wall is automatically removed, creating an open connection. **Overlapping polygons** are merged into a single outer wall. **Edge indexing:** edge 0 = `points[0]` → `points[1]`, the last edge wraps to `points[0]`.

### Portal Types

| Type | Description |
|------|-------------|
| `door` | Standard door |
| `window` | Standard window |
| `archway` | Open archway |
| `secret_door` | Hidden door |

### Pattern Fills

Polygon-bounded texture fills — floor tiles inside rooms, water overlays, or noise-gated ground detail. Key: `pattern_fills`.

```yaml
pattern_fills:
  - region:                              # PatternRegion, required — where to place the pattern
      type: room                         # "water" | "room" | "polygon" | "noise"
      name: "guard_room"                 # room/polygon id (room and polygon types only)
    texture: "res://textures/patterns/tile_01.png"   # string, required — tileset texture
    color: "ffffffff"                    # string — ARGB hex tint, default "ffffffff"
    rotation: 0                          # i32 — texture rotation in degrees, default 0
    layer: 100                           # i32 — render layer, default 100
    outline: false                       # bool — outline instead of fill, default false
```

**Region types:**

| `type` | Extra fields | Fills |
|--------|-------------|-------|
| `water` | none | Each generated water polygon |
| `room` | `name` (string — room id) | The named room's boundary |
| `polygon` | `name` (string — polygon id) | The named polygon's boundary |
| `noise` | `noise_lower`, `noise_upper` (f64) | Contours where noise falls in the range |

### Custom Paths

General-purpose paths beyond roads and rivers.

```yaml
custom_paths:
  - style:                               # PathStyle, required — how the path is generated
      type: waypoints                    # "waypoints" | "room_to_room" | "offset" | "intermittent"
      points:                            # waypoints style: [x, y] in grid squares
        - [4, 4]
        - [12, 8]
        - [20, 6]
    texture: "res://textures/paths/path_dirt.png"    # string, required
    width: 1.0                           # f64, required — width in grid squares
    color: "ffffffff"                    # string — ARGB hex, default "ffffffff"
    layer: 100                           # i32 — render layer, default 100
    smooth: 8                            # usize — Bezier smoothing density (0 = none), default 8
    loop_path: false                     # bool — close the path into a loop, default false
```

**Style types:**

| `type` | Fields | Behavior |
|--------|--------|----------|
| `waypoints` | `points` (list of `[x, y]` in grid squares) | Explicit path through the points |
| `room_to_room` | `from`, `to` (string — room ids) | Connects the two rooms' centers |
| `offset` | `along` (string — feature id), `offset` (f64, grid squares, positive = left), `reverse` (bool, default false) | Companion path at a perpendicular distance from a named road/river/feature |
| `intermittent` | `along` (string), `segment_length` (f64), `segment_variation` (f64, default 0), `gap` (f64), `offset` (f64, default 0), `reverse` (bool, default false) | Breaks a parent path into segments with gaps |

### Lights

Point light placement.

```yaml
lights:
  - placement:                           # LightPlacement, required — placement mode
      type: scatter                      # "scatter" | "along_path" | "with_objects"
      density: 6.0                       # f64, required — minimum spacing in grid squares
      noise_lower: 0.0                   # f64 — lower noise bound, default 0.0
      noise_upper: 1.0                   # f64 — upper noise bound, default 1.0
      probability: 1.0                   # f64 — keep chance per point, default 1.0
      margin: 0.1                        # f64 — edge margin fraction (0.0–0.5), default 0.1
    color: "ffeaefca"                    # string, required — ARGB hex
    intensity: 0.6                       # f64, required — typical range 0.0–1.0
    range: 4.0                           # f64, required — light range in grid squares
    shadows: false                       # bool — cast shadows, default false
    layer: 100                           # i32 — render layer, default 100
```

**Placement types:**

| `type` | Fields | Behavior |
|--------|--------|----------|
| `scatter` | `density` (f64, required), `noise_lower`, `noise_upper`, `probability`, `margin` | Poisson disc scatter gated by noise and probability |
| `along_path` | `path` (string — road/river/feature id), `density` (f64 — lights per grid square of path) | Lights at intervals along a named path |
| `with_objects` | `group` (string — tree/clutter/clump id) | One light per placed object in the named group |

### Materials

Ground-level material scatter (ice, lava, acid, debris) as bit-packed bitmaps.

```yaml
materials:
  - region:                              # MaterialRegion, required — where to place material
      type: noise                        # "noise" | "room" | "polygon" | "along_path"
      noise_lower: 0.0                   # f64 — lower noise bound (noise type)
      noise_upper: 0.25                  # f64 — upper noise bound (noise type)
    texture: "res://textures/materials/ice.png"      # string, required
    layer: "-400"                        # string — Dungeondraft layer id, default "-400" (Below Ground)
    smooth: true                         # bool — edge smoothing, default true
```

**Region types:**

| `type` | Extra fields | Fills |
|--------|-------------|-------|
| `noise` | `noise_lower`, `noise_upper` (f64) | Cells where noise falls in the range |
| `room` | `name` (string — room id) | Inside the named room |
| `polygon` | `name` (string — polygon id) | Inside the named polygon |
| `along_path` | `path` (string — feature id), `width` (f64 — corridor half-width in grid squares) | Within a distance of the named path |

### Lighting

Ambient lighting and shadow configuration.

```yaml
lighting:
  ambient_light: "ffffffff"              # string, required — ARGB hex ambient color
  ambient_energy: 1.0                    # f64, optional — brightness multiplier
  shadow_color: "66000000"               # string, optional — ARGB hex shadow color
```

#### Time-of-Day Reference Values

There is no YAML key, CLI flag, or MCP parameter that selects a time-of-day preset. The values below are reference colors to copy into an explicit `lighting:` block:

| Time of day | `ambient_light` | `ambient_energy` | `shadow_color` |
|-------------|-----------------|------------------|----------------|
| Dawn | `fff5d0a0` | 0.7 | `66483020` |
| Day | `ffffffff` | omit | omit |
| Dusk | `ffdd8866` | 0.6 | `66301830` |
| Night | `ff4466aa` | 0.3 | `cc000020` |
| Underground | `ff333333` | 0.2 | `cc000000` |

## Common Dungeondraft Texture Paths

These `res://` paths reference Dungeondraft's default asset pack.

### Terrain
- `res://textures/terrain/terrain_grass.png`
- `res://textures/terrain/terrain_dirt.png`
- `res://textures/terrain/terrain_dry_grass.png`
- `res://textures/terrain/terrain_moss.png`
- `res://textures/terrain/terrain_gravel.png`
- `res://textures/terrain/terrain_stone.png`
- `res://textures/terrain/terrain_sand.png`
- `res://textures/terrain/terrain_snow.png`
- `res://textures/terrain/terrain_ice.png`

### Walls
- `res://textures/walls/battlements.png`
- `res://textures/walls/stone.png`
- `res://textures/walls/wood.png`
- `res://textures/walls/cave.png`

### Portals
- `res://textures/portals/door_00.png`
- `res://textures/portals/window_03.png`
- `res://textures/portals/archway_00.png`
- `res://textures/portals/secret_00.png`

## See Also

- [Generate Maps How-To](../how-to/maps/generate-map.md)
- [MCP Server Reference](./mcp-server.md)
