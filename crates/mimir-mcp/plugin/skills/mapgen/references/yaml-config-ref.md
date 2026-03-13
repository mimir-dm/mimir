# MapConfig YAML Reference

Complete schema for mimir-mapgen YAML configuration files.

## Top-Level Fields

```yaml
name: "My Forest Map"          # string, required — map display name
width: 32                       # u32, required — width in grid squares
height: 32                      # u32, required — height in grid squares
seed: 42                        # u64, optional — random seed (overridden by CLI --seed or MCP seed param)
```

## Noise Configuration

Controls the Perlin noise that drives terrain, object placement, and feature generation.

```yaml
noise:
  seed: 0              # u32 — noise seed (separate from map seed, usually left at 0)
  octaves: 6           # u32 — number of noise layers (more = more detail, default: 6)
  persistence: 0.5     # f64 — amplitude decay per octave (0-1, default: 0.5)
  lacunarity: 2.0      # f64 — frequency multiplier per octave (default: 2.0)
  scale: 0.03          # f64 — base frequency (lower = larger features, default: 0.03)
```

## Island Mode

Optional. Applies a radial falloff to create island/clearing shapes where the center is low and edges are high.

```yaml
island_mode: 1.5       # f64, optional — falloff strength (higher = steeper falloff)
```

## Terrain Configuration

Defines the 4 terrain texture slots and their noise-based blending.

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
  blend_width: 0.05            # f64 — overlap width for blending between slots
  smooth_blending: false        # bool — use smooth interpolation vs hard boundaries
```

**Notes:** Slots should have overlapping lower/upper ranges for natural blending. Each slot's weight is determined by how well the noise value falls within its range.

## Trees

Array of tree configurations. Each entry places a tree type with optional shadow and canopy layers.

```yaml
trees:
  - tree:
      textures:                          # list of texture paths (randomly selected)
        - "res://textures/objects/trees/tree_01.png"
        - "res://textures/objects/trees/tree_02.png"
      min_distance: 180.0                # f64 — minimum spacing between trees (pixels)
      noise_lower: 0.3                   # f64 — only place where noise > this
      noise_upper: 0.8                   # f64 — only place where noise < this
      probability: 0.8                   # f64 — chance to place at valid position (0-1)
      scale_min: 0.8                     # f64 — minimum random scale
      scale_max: 1.4                     # f64 — maximum random scale
      layer: 300                         # i32 — render layer in Dungeondraft
      random_rotation: true              # bool — randomize rotation
      random_mirror: true                # bool — randomly flip horizontally
      custom_color: null                 # string, optional — ARGB hex color override
    shadow:                              # optional shadow beneath the tree
      texture: "res://textures/objects/trees/tree_shadow.png"
      offset_x: 10.0                     # f64 — shadow offset X
      offset_y: 15.0                     # f64 — shadow offset Y
      scale_factor: 1.2                  # f64 — shadow scale relative to tree
      layer: 50                          # i32 — render layer (below tree)
    canopy:                              # optional canopy on upper level
      texture: "res://textures/objects/trees/tree_canopy.png"
      scale_factor: 1.5                  # f64 — canopy scale relative to tree
      layer: 500                         # i32 — render layer (above tree)
      level: 1                           # i32 — Dungeondraft level (1 = upper)
```

## Clutter

Array of small decorative objects scattered across the map.

```yaml
clutter:
  - textures:
      - "res://textures/objects/grass/grass_01.png"
      - "res://textures/objects/grass/grass_02.png"
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

## Clumps

Groups of objects clustered together (e.g., rock formations, flower patches).

```yaml
clumps:
  - primary:                             # main clump object (placed via Poisson Disc)
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
    scatter_count: 5                     # i32 — number of secondary objects per primary
    scatter_radius: 100.0                # f64 — radius around primary to scatter
```

## Roads

Array of road paths. Roads use either a greedy noise-following walk (`straight`) or sinusoidal meander (`meandering`). All fields have defaults via `#[serde(default)]`, so `roads: [{}]` produces a valid default road.

```yaml
roads:
  - from: Left                           # Edge — start side: Left, Right, Top, Bottom (default: Left)
    to: Right                            # Edge — end side (default: Right)
    texture: "res://textures/paths/path_dirt.png"  # string — road texture
    width: 512.0                         # f64 — road width in pixels (default: 512 = 2 grid squares)
    layer: 100                           # i32 — render layer (default: 100)
    style: straight                      # PathStyle — "straight" or "meandering" (default: straight)
    step_distance: 64.0                  # f64 — step size for walk (pixels, default: 64)
    fov: 1.047                           # f64 — field of view angle in radians (default: π/3)
    noise_weight: 0.5                    # f64 — 0=all progress, 1=all noise (default: 0.5)
    margin: 128.0                        # f64 — margin from map edge in pixels (default: 128)
    smooth_density: 8                    # usize — Bezier smoothing density (default: 8)
    edge_paths:                          # optional — border textures along road sides
      texture: "res://textures/paths/path_edge.png"
      offset: 20.0                       # f64 — perpendicular offset from center
      width: 8.0                         # f64 — edge path width
      layer: 190                         # i32 — render layer
```

**PathStyle:**
- `straight` — greedy walk that follows noise ridges/valleys. Roughly straight with gentle bends.
- `meandering` — sinusoidal S-curves. More natural for rivers, but also usable for winding roads.

**Exclusion zones:** Roads automatically route around rooms and corridors. No configuration needed.

## Rivers

Array of river paths with water polygon and bank paths. Rivers default to `meandering` style.

```yaml
rivers:
  - from: Top                            # Edge — start side (default: Top)
    to: Bottom                           # Edge — end side (default: Bottom)
    width: 120.0                         # f64 — river/water width in pixels (default: 120)
    style: meandering                    # PathStyle — "straight" or "meandering" (default: meandering)
    step_distance: 64.0                  # f64 — step size for walk (default: 64)
    fov: 1.047                           # f64 — FOV for straight style (default: π/3)
    noise_weight: 0.5                    # f64 — noise following weight (default: 0.5)
    margin: 128.0                        # f64 — margin from map edge (default: 128)
    smooth_density: 8                    # usize — Bezier smoothing (default: 8)
    deep_color: "ff3aa19a"               # string — ARGB hex for deep water
    shallow_color: "ff3ac3b2"            # string — ARGB hex for shallow water
    bank_texture: "res://textures/paths/path_rocks.png"  # string — bank path texture
    bank_width: 20.0                     # f64 — bank path width (default: 20)
    bank_layer: 100                      # i32 — bank render layer (default: 100)
```

**Notes:**
- Rivers generate a water polygon (DD water tree) between the two banks, plus bank paths on each side.
- `width` controls how wide the water corridor is. Very wide rivers (1000+) can simulate coastlines.
- Meandering rivers produce natural S-curves. The meander amplitude scales with `width`.
- **Exclusion zones:** Rivers automatically shift their path to avoid rooms and corridors.

## Water Bodies

Noise-threshold water generation (lakes, ponds).

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

## Elevation Contours

Cliff and hill contour lines at noise thresholds.

```yaml
elevation:
  levels:
    - threshold: 0.65                    # f64 — noise threshold for this contour
      texture: "res://textures/paths/path_rocks.png"
      width: 12.0                        # f64 — contour line width
      layer: 100                         # i32 — render layer
      min_points: 8                      # usize — minimum contour length
      smooth_iterations: 2               # usize — smoothing passes
      shadow:                            # optional shadow below contour
        texture: "res://textures/paths/path_rocks.png"
        offset: 8.0                      # f64 — perpendicular offset
        width: 16.0                      # f64 — shadow width
        layer: 50                        # i32 — shadow layer (below contour)
  pixels_per_cell: 64.0                  # f64 — coordinate scaling
```

## Polygons

Arbitrary closed shapes defined by vertices in grid coordinates. Use polygons for irregular rooms, oval chambers, L-shaped areas, or any non-rectangular layout. The system automatically merges overlapping polygons, removes shared edges, and generates walls, shapes (interior fill), and portals.

```yaml
polygons:
  - id: "room_a"                   # string, required — unique identifier
    points:                        # list of [x, y] vertices, clockwise
      - [3, 6]
      - [9, 6]
      - [9, 14]
      - [3, 14]
    terrain_slot: 3                # usize, optional — terrain texture slot (0-3) for floor fill
    wall_texture: "res://textures/walls/battlements.png"  # optional, default: battlements
    portals:                       # optional — doors/windows on polygon edges
      - edge: 0                   # usize — 0-indexed segment (edge 0 = points[0]→points[1])
        position: 0.5             # f64 — fractional position along edge (0.0=start, 1.0=end)
        type: door                # PortalType — door, window, archway, secret_door

  - id: "corridor"
    points: [[9, 10], [13, 10], [13, 11], [9, 11]]
    terrain_slot: 3
    portals:
      - edge: 3                   # left edge (points[3]→points[0])
        position: 0.5
        type: door
      - edge: 1                   # right edge (points[1]→points[2])
        position: 0.5
        type: door

  - id: "room_b"
    points: [[13, 6], [19, 6], [19, 14], [13, 14]]
    terrain_slot: 3
```

**How adjacency works:**
- When two polygons share an edge (e.g., room_a has `[9,6]→[9,14]` and room_b has `[9,14]→[9,6]`), the shared wall segment is automatically removed.
- This creates open connections between adjacent polygons without needing corridors.
- Portals on shared edges become freestanding (floating in the opening). Portals on the outer perimeter become wall-anchored.

**How overlapping polygons work:**
- Polygons that share vertices but have crossing or overlapping edges are merged into a single outer wall using the CW walk algorithm.
- Interior edges are removed; only the outer union boundary becomes a wall.
- Each polygon still gets its own interior fill (DD shape) and terrain override.

**Edge indexing:**
- Edges are 0-indexed segments between consecutive vertices.
- Edge 0 = `points[0]` → `points[1]`, edge 1 = `points[1]` → `points[2]`, etc.
- The last edge wraps: `points[N-1]` → `points[0]`.

**Terrain fill:**
- Unlike rooms (which use bounding-box fill), polygon terrain fill uses per-cell point-in-polygon testing.
- Only cells whose center falls inside the polygon get the terrain override.
- This means octagonal, oval, and irregular polygons get accurate fill boundaries.

## Rooms

Declarative room definitions. Rooms are rectangular areas with walls, doors, and windows placed on the grid. Room interiors override noise-based terrain, and outdoor features (trees, clutter, roads) route around them.

```yaml
rooms:
  - id: "guard_room"              # string, required — unique room identifier
    x: 4                          # u32, required — grid X position (left edge)
    y: 6                          # u32, required — grid Y position (top edge)
    width: 5                      # u32, required — width in grid squares
    height: 4                     # u32, required — height in grid squares
    terrain_slot: 3               # usize, optional — terrain texture slot (0-3) for floor
    walls:                        # optional — per-wall toggles (default: all true)
      north: true
      south: true
      east: true
      west: false                 # open side (e.g., corridor entrance)
    portals:                      # optional — doors/windows on walls
      - wall: "north"            # WallSide — which wall: north, south, east, west
        position: 2              # u32 — grid offset along wall from start
        type: "door"             # PortalType — door, window, archway, secret_door
        width: 1                 # u32 — portal width in grid squares (default: 1)
```

**Notes:**
- All coordinates are in grid squares (256 pixels per grid square internally)
- Walls default to all enabled — only specify walls section to disable specific sides
- Portal position is measured from the left (N/S walls) or top (E/W walls) of the wall
- Portal `position + width` must not exceed the wall length
- `terrain_slot` indexes into the terrain configuration's 4 texture slots
- Rooms must not overlap each other or extend beyond map boundaries

### Portal Types

| Type | Texture | Default Width |
|------|---------|---------------|
| `door` | `res://textures/portals/door_00.png` | 1 grid square (128px radius) |
| `window` | `res://textures/portals/window_03.png` | 1 grid square (99.5px radius) |
| `archway` | `res://textures/portals/archway_00.png` | 1 grid square (128px radius) |
| `secret_door` | `res://textures/portals/secret_00.png` | 1 grid square (128px radius) |

## Corridors

Connections between rooms. Corridors generate parallel wall segments and can have doors at their ends.

```yaml
corridors:
  - from: "guard_room"           # string, required — source room ID
    from_wall: "east"            # WallSide, required — exit wall of source room
    to: "throne_room"            # string, required — destination room ID
    to_wall: "west"              # WallSide, required — entry wall of destination room
    width: 2                     # u32 — corridor width in grid squares (default: 2)
    terrain_slot: 3              # usize, optional — floor terrain slot
    portals:                     # optional — doors at corridor ends
      - end: "from"             # CorridorEnd — "from" or "to"
        type: "door"            # PortalType — door, window, archway, secret_door
        width: 1                # u32 — portal width in grid squares (default: 1)
```

**Notes:**
- Corridors are straight when rooms are aligned on the connecting axis; L-shaped otherwise
- Corridor portals are freestanding (not embedded in walls)
- Both `from` and `to` must reference valid room IDs
- Corridor width should be smaller than the connecting wall length

## Lighting

Ambient lighting and shadow configuration. Can also use time-of-day presets.

```yaml
lighting:
  ambient_light: "ffffffff"              # string — ARGB hex ambient color
  ambient_energy: 1.0                    # f64, optional — brightness multiplier
  shadow_color: "66000000"               # string, optional — ARGB hex shadow color
```

### Time-of-Day Presets

Instead of manually configuring lighting, reference these presets when discussing options with the user:

| Preset | Ambient Color | Energy | Shadow | Use Case |
|--------|--------------|--------|--------|----------|
| `dawn` | warm gold | 0.7 | brown tint | Early morning encounters |
| `day` | white | 1.0 | none | Default daylight |
| `dusk` | orange-red | 0.6 | purple tint | Evening ambush, sunset |
| `night` | blue | 0.3 | dark | Nighttime stealth, camp |
| `underground` | dark grey | 0.2 | black | Caves, dungeons |

## Common Dungeondraft Texture Paths

These `res://` paths are commonly available in Dungeondraft's default asset pack:

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

### Objects
- `res://textures/objects/trees/tree_01.png` through `tree_03.png`
- `res://textures/objects/grass/grass_01.png`, `grass_02.png`

### Paths
- `res://textures/paths/path_cobble.png`
- `res://textures/paths/path_rocks.png`
- `res://textures/paths/path_shadow.png`
- `res://textures/paths/path_cliff.png`

### Walls
- `res://textures/walls/battlements.png`
- `res://textures/walls/stone.png`
- `res://textures/walls/wood.png`
- `res://textures/walls/cave.png`

### Portals
- `res://textures/portals/door_00.png` — standard door
- `res://textures/portals/window_03.png` — standard window
- `res://textures/portals/archway_00.png` — archway
- `res://textures/portals/secret_00.png` — secret door
