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
| `ice_lake` | 32×32 | Frozen lake with cracked ice, snow-covered shores, and frigid water |
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
  seed: 0              # u32 — noise seed (separate from map seed)
  octaves: 6           # u32 — noise layers (more = more detail)
  persistence: 0.5     # f64 — amplitude decay per octave (0–1)
  lacunarity: 2.0      # f64 — frequency multiplier per octave
  scale: 0.03          # f64 — base frequency (lower = larger features)
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
  - tree:
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
      offset_x: 10.0                     # f64 — shadow offset X
      offset_y: 15.0                     # f64 — shadow offset Y
      scale_factor: 1.2                  # f64 — scale relative to tree
      layer: 50                          # i32 — render layer (below tree)
    canopy:                              # optional canopy on upper level
      texture: "res://textures/objects/trees/tree_canopy.png"
      scale_factor: 1.5                  # f64 — scale relative to tree
      layer: 500                         # i32 — render layer (above tree)
      level: 1                           # i32 — Dungeondraft level (1 = upper)
```

### Clutter

Small decorative objects scattered across the map.

```yaml
clutter:
  - textures:
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

### Clumps

Clustered object groups (rock formations, flower patches).

```yaml
clumps:
  - primary:                             # main object (placed via Poisson Disc)
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
      scale_min: 0.3
      scale_max: 0.6
      layer: 100
      random_rotation: true
      random_mirror: false
    scatter_count: 5                     # i32 — secondaries per primary
    scatter_radius: 100.0                # f64 — scatter radius around primary
```

### Roads

Road paths using noise-following walk (`straight`) or sinusoidal meander (`meandering`).

```yaml
roads:
  - from: Left                           # Edge — start: Left, Right, Top, Bottom
    to: Right                            # Edge — end side
    texture: "res://textures/paths/path_dirt.png"
    width: 512.0                         # f64 — road width in pixels (512 = 2 grid squares)
    layer: 100                           # i32 — render layer
    style: straight                      # PathStyle — "straight" or "meandering"
    step_distance: 64.0                  # f64 — walk step size (pixels)
    fov: 1.047                           # f64 — field of view angle (radians, default π/3)
    noise_weight: 0.5                    # f64 — 0=all progress, 1=all noise
    margin: 128.0                        # f64 — margin from map edge (pixels)
    smooth_density: 8                    # usize — Bezier smoothing density
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
  - from: Top                            # Edge — start side
    to: Bottom                           # Edge — end side
    width: 120.0                         # f64 — water width (pixels)
    style: meandering                    # PathStyle — "straight" or "meandering"
    step_distance: 64.0                  # f64 — walk step size
    fov: 1.047                           # f64 — FOV for straight style
    noise_weight: 0.5                    # f64 — noise following weight
    margin: 128.0                        # f64 — margin from map edge
    smooth_density: 8                    # usize — Bezier smoothing
    deep_color: "ff3aa19a"               # string — ARGB hex for deep water
    shallow_color: "ff3ac3b2"            # string — ARGB hex for shallow water
    bank_texture: "res://textures/paths/path_rocks.png"
    bank_width: 20.0                     # f64 — bank path width
    bank_layer: 100                      # i32 — bank render layer
```

Rivers generate a water polygon between two banks plus bank paths on each side. Very wide rivers (1000+) can simulate coastlines. Rivers automatically route around rooms and corridors.

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
    points:                        # list of [x, y] vertices, clockwise
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

**Adjacency:** When two polygons share an edge, the shared wall is automatically removed, creating an open connection. **Overlapping polygons** are merged into a single outer wall. **Edge indexing:** edge 0 = `points[0]` → `points[1]`, the last edge wraps to `points[0]`.

### Portal Types

| Type | Description |
|------|-------------|
| `door` | Standard door |
| `window` | Standard window |
| `archway` | Open archway |
| `secret_door` | Hidden door |

### Lighting

Ambient lighting and shadow configuration.

```yaml
lighting:
  ambient_light: "ffffffff"              # string — ARGB hex ambient color
  ambient_energy: 1.0                    # f64, optional — brightness multiplier
  shadow_color: "66000000"               # string, optional — ARGB hex shadow color
```

#### Time-of-Day Presets

| Preset | Ambient Color | Energy | Shadow | Use Case |
|--------|--------------|--------|--------|----------|
| `dawn` | Warm gold | 0.7 | Brown tint | Early morning encounters |
| `day` | White | 1.0 | None | Default daylight |
| `dusk` | Orange-red | 0.6 | Purple tint | Evening ambush, sunset |
| `night` | Blue | 0.3 | Dark | Nighttime stealth, camp |
| `underground` | Dark grey | 0.2 | Black | Caves, dungeons |

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
