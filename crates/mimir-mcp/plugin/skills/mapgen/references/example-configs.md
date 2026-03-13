# Example Configurations

Curated YAML configs demonstrating key mapgen patterns. Use these as templates when building custom configs — copy the structure and adjust parameters to match the user's scene.

## Pattern: Basic Forest with Road and Elevation

The simplest "full-featured" outdoor map. Default road (`roads: [{}]`) gives you a dirt path from left to right. Elevation contours add rocky cliff lines at high-noise areas.

```yaml
name: "Forest"
width: 32
height: 32
seed: 42

noise:
  octaves: 6
  persistence: 0.5
  lacunarity: 2.0
  scale: 0.03

terrain:
  slots:
    - texture: "res://textures/terrain/terrain_dirt.png"
      lower: 0.0
      upper: 0.3
    - texture: "res://textures/terrain/terrain_dry_grass.png"
      lower: 0.25
      upper: 0.55
    - texture: "res://textures/terrain/terrain_moss.png"
      lower: 0.5
      upper: 0.8
    - texture: "res://textures/terrain/terrain_gravel.png"
      lower: 0.75
      upper: 1.0
  blend_width: 0.05
  smooth_blending: false

trees:
  - tree:
      textures:
        - "res://textures/objects/more_trees/oak_01.png"
        - "res://textures/objects/more_trees/oak_02.png"
        - "res://textures/objects/more_trees/oak_03.png"
      min_distance: 500.0
      noise_lower: 0.4
      noise_upper: 0.75
      probability: 0.5
      scale_min: 0.8
      scale_max: 1.2
      layer: 300
      random_rotation: true
      random_mirror: true

clutter:
  - textures:
      - "res://textures/objects/vegetation/grass/grass_01.png"
      - "res://textures/objects/vegetation/grass/grass_02.png"
    min_distance: 80.0
    noise_lower: 0.2
    noise_upper: 0.7
    probability: 0.6
    scale_min: 0.5
    scale_max: 1.0
    layer: 100
    random_rotation: true
    random_mirror: false

roads:
  - {}

elevation:
  levels:
    - threshold: 0.65
      texture: "res://textures/paths/path_rocks.png"
      width: 12.0
      layer: 100
      min_points: 8
      smooth_iterations: 2
      shadow:
        texture: "res://textures/paths/path_rocks.png"
        offset: 8.0
        width: 16.0
        layer: 50
  pixels_per_cell: 64.0
```

**Key takeaways:**
- `roads: [{}]` uses all defaults — Left→Right, dirt texture, 512px wide, straight style
- Overlapping terrain slot ranges (0.25-0.55 overlaps 0.0-0.3) create natural blending
- Trees only spawn where noise is 0.4–0.75 (mid-range), leaving clearings at extremes
- Elevation shadow sits below (layer 50) the contour line (layer 100)

## Pattern: Forest with Meandering River

Same forest terrain but with a river instead of a road. Rivers default to `meandering` style (sinusoidal S-curves).

```yaml
name: "Forest River"
width: 32
height: 32
seed: 42

noise:
  octaves: 6
  persistence: 0.5
  lacunarity: 2.0
  scale: 0.03

terrain:
  slots:
    - texture: "res://textures/terrain/terrain_dirt.png"
      lower: 0.0
      upper: 0.3
    - texture: "res://textures/terrain/terrain_dry_grass.png"
      lower: 0.25
      upper: 0.55
    - texture: "res://textures/terrain/terrain_moss.png"
      lower: 0.5
      upper: 0.8
    - texture: "res://textures/terrain/terrain_gravel.png"
      lower: 0.75
      upper: 1.0
  blend_width: 0.05
  smooth_blending: false

trees:
  - tree:
      textures:
        - "res://textures/objects/more_trees/oak_01.png"
        - "res://textures/objects/more_trees/oak_02.png"
        - "res://textures/objects/more_trees/oak_03.png"
      min_distance: 500.0
      noise_lower: 0.4
      noise_upper: 0.75
      probability: 0.5
      scale_min: 0.8
      scale_max: 1.2
      layer: 300
      random_rotation: true
      random_mirror: true

clutter:
  - textures:
      - "res://textures/objects/vegetation/grass/grass_01.png"
      - "res://textures/objects/vegetation/grass/grass_02.png"
    min_distance: 80.0
    noise_lower: 0.2
    noise_upper: 0.7
    probability: 0.6
    scale_min: 0.5
    scale_max: 1.0
    layer: 100
    random_rotation: true
    random_mirror: false

rivers:
  - from: Top
    to: Bottom
    width: 768.0
    step_distance: 48.0
    margin: 384.0
    smooth_density: 8
    deep_color: "ff45b1cd"
    shallow_color: "ff7dcade"
    bank_texture: "res://textures/paths/path_rocks.png"
    bank_width: 32.0
    bank_layer: 100
```

**Key takeaways:**
- Rivers default to `style: meandering` — no need to specify it
- `width: 768.0` = 3 grid squares of water. Wider rivers have more pronounced meander curves.
- `margin: 384.0` keeps the river path away from map edges for cleaner entry/exit
- Blue water colors (`ff45b1cd`/`ff7dcade`) instead of green — river feels different from swamp
- Bank paths (`path_rocks.png`) create a rocky shore on both sides of the river

## Pattern: Mixed Map — Building + River + Road (Exclusion Zones)

A tavern building next to a river with a road crossing the map. **Roads and rivers automatically route around rooms** — no extra configuration needed. This is the key pattern for "structure in nature" maps.

```yaml
name: "Tavern by the River"
width: 32
height: 32
seed: 42

noise:
  octaves: 6
  persistence: 0.5
  lacunarity: 2.0
  scale: 0.03

terrain:
  slots:
    - texture: "res://textures/terrain/terrain_dirt.png"
      lower: 0.0
      upper: 0.3
    - texture: "res://textures/terrain/terrain_dry_grass.png"
      lower: 0.25
      upper: 0.55
    - texture: "res://textures/terrain/terrain_moss.png"
      lower: 0.5
      upper: 0.8
    - texture: "res://textures/terrain/terrain_gravel.png"
      lower: 0.75
      upper: 1.0
  blend_width: 0.05
  smooth_blending: false

rooms:
  - id: "tavern"
    x: 13
    y: 13
    width: 6
    height: 5
    terrain_slot: 0
    portals:
      - wall: south
        position: 2
        type: door

trees:
  - tree:
      textures:
        - "res://textures/objects/more_trees/oak_01.png"
        - "res://textures/objects/more_trees/oak_02.png"
        - "res://textures/objects/more_trees/oak_03.png"
      min_distance: 500.0
      noise_lower: 0.4
      noise_upper: 0.75
      probability: 0.5
      scale_min: 0.8
      scale_max: 1.2
      layer: 300
      random_rotation: true
      random_mirror: true

clutter:
  - textures:
      - "res://textures/objects/vegetation/grass/grass_01.png"
      - "res://textures/objects/vegetation/grass/grass_02.png"
    min_distance: 80.0
    noise_lower: 0.2
    noise_upper: 0.7
    probability: 0.6
    scale_min: 0.5
    scale_max: 1.0
    layer: 100
    random_rotation: true
    random_mirror: false

roads:
  - from: Left
    to: Right
    texture: "res://textures/paths/path_dirt.png"
    width: 512.0
    layer: 100
    style: straight
    step_distance: 48.0
    margin: 384.0
    smooth_density: 8

rivers:
  - from: Top
    to: Bottom
    width: 1024.0
    step_distance: 48.0
    fov: 1.5
    noise_weight: 0.7
    margin: 384.0
    smooth_density: 8
    deep_color: "ff3aa19a"
    shallow_color: "ff3ac3b2"
    bank_texture: "res://textures/paths/path_rocks.png"
    bank_width: 32.0
    bank_layer: 100
```

**Key takeaways:**
- The room creates an **exclusion zone** — the river shifts its entire path to avoid the tavern, and the road routes around it
- `style: straight` on the road gives a direct noise-following path (good for a trade road)
- The river is very wide (`width: 1024.0` = 4 grid squares) — gives it visual prominence
- Room `terrain_slot: 0` fills the tavern floor with dirt texture (slot 0 in the terrain config)
- Trees, clutter, roads, and rivers all respect room exclusion zones automatically

## Pattern: Swamp — Dark Atmosphere with Standing Water

Low water threshold floods much of the map with murky water pools. Dark ambient lighting creates an oppressive atmosphere. `island_mode: -1.0` inverts the radial falloff so water appears at the edges.

```yaml
name: "Swamp"
width: 32
height: 32
seed: 42

noise:
  octaves: 6
  persistence: 0.55
  lacunarity: 2.0
  scale: 0.035

island_mode: -1.0

terrain:
  slots:
    - texture: "res://textures/terrain/terrain_moss.png"
      lower: 0.0
      upper: 0.35
    - texture: "res://textures/terrain/terrain_dirt.png"
      lower: 0.3
      upper: 0.55
    - texture: "res://textures/terrain/terrain_dry_grass.png"
      lower: 0.5
      upper: 0.75
    - texture: "res://textures/terrain/terrain_grass.png"
      lower: 0.7
      upper: 1.0
  blend_width: 0.07
  smooth_blending: true

trees:
  - tree:
      textures:
        - "res://textures/objects/more_trees/oak_01.png"
        - "res://textures/objects/more_trees/oak_02.png"
      min_distance: 600.0
      noise_lower: 0.3
      noise_upper: 0.6
      probability: 0.35
      scale_min: 0.7
      scale_max: 1.0
      layer: 300
      random_rotation: true
      random_mirror: true

clutter:
  - textures:
      - "res://textures/objects/vegetation/grass/grass_01.png"
      - "res://textures/objects/vegetation/grass/grass_02.png"
    min_distance: 90.0
    noise_lower: 0.15
    noise_upper: 0.55
    probability: 0.5
    scale_min: 0.4
    scale_max: 0.8
    layer: 100
    random_rotation: true
    random_mirror: false

water:
  threshold: 0.5
  deep_color: "ff2e6d27"
  shallow_color: "c898d49a"
  blend_distance: 45.0
  min_contour_points: 15
  smooth_iterations: 3
  pixels_per_cell: 64.0

lighting:
  ambient_light: "ff3c3c3c"
  ambient_energy: 0.4
  shadow_color: "cc111111"
```

**Key takeaways:**
- `water.threshold: 0.5` is very low — half the noise range becomes water, creating lots of pools
- Water colors are dark green (`ff2e6d27`) and murky (`c898d49a`) — the `c8` alpha makes shallow water semi-transparent
- Trees are sparse (`probability: 0.35`, `min_distance: 600.0`) and slightly smaller (`scale_max: 1.0`)
- `lighting.ambient_energy: 0.4` dims the map significantly. Dark shadow color adds to the gloom.
- Moss is the base terrain (slot 0), giving a soggy green floor

## Pattern: Island with Building — Water Surrounds Land

An island with a fort structure. `island_mode: 1.0` creates a land mass in the center with water around the edges. The water threshold is high so only the very edge of the map floods.

```yaml
name: "Island Fort"
width: 32
height: 32
seed: 42

noise:
  octaves: 5
  persistence: 0.5
  lacunarity: 2.0
  scale: 0.035

island_mode: 1.0

terrain:
  slots:
    - texture: "res://textures/terrain/terrain_grass.png"
      lower: 0.0
      upper: 0.35
    - texture: "res://textures/terrain/terrain_moss.png"
      lower: 0.3
      upper: 0.55
    - texture: "res://textures/terrain/terrain_dry_grass.png"
      lower: 0.5
      upper: 0.75
    - texture: "res://textures/terrain/terrain_sand.png"
      lower: 0.7
      upper: 1.0
  blend_width: 0.06
  smooth_blending: true

rooms:
  - id: "fort"
    x: 12
    y: 12
    width: 8
    height: 8
    terrain_slot: 0
    portals:
      - wall: south
        position: 3
        type: door

trees:
  - tree:
      textures:
        - "res://textures/objects/more_trees/oak_01.png"
        - "res://textures/objects/more_trees/oak_02.png"
      min_distance: 350.0
      noise_lower: 0.0
      noise_upper: 0.5
      probability: 0.6
      scale_min: 0.7
      scale_max: 1.1
      layer: 300
      random_rotation: true
      random_mirror: true

clutter:
  - textures:
      - "res://textures/objects/vegetation/grass/grass_01.png"
      - "res://textures/objects/vegetation/grass/grass_02.png"
    min_distance: 70.0
    noise_lower: 0.25
    noise_upper: 0.6
    probability: 0.6
    scale_min: 0.4
    scale_max: 0.9
    layer: 100
    random_rotation: true
    random_mirror: false

water:
  threshold: 0.85
  deep_color: "ff1a6b5a"
  shallow_color: "ff30b89a"
  blend_distance: 60.0
  min_contour_points: 20
  smooth_iterations: 5
  pixels_per_cell: 64.0
```

**Key takeaways:**
- `island_mode: 1.0` — positive value = land in center, water at edges
- `island_mode: -1.0` — negative value = water in center, land at edges (used for lakes/swamps)
- `water.threshold: 0.85` — high threshold means only the extreme noise values (outer ring) become water
- Sand texture at the highest slot creates a natural beach transition at the water's edge
- Trees have `noise_upper: 0.5` — they only grow in the center of the island, not near the water
- `smooth_iterations: 5` gives the water contour very smooth, organic-looking edges

## Minimal Configs

### Bare minimum (terrain only)
```yaml
name: "Simple"
width: 24
height: 24
terrain:
  slots:
    - texture: "res://textures/terrain/terrain_grass.png"
      lower: 0.0
      upper: 0.5
    - texture: "res://textures/terrain/terrain_dirt.png"
      lower: 0.4
      upper: 0.7
    - texture: "res://textures/terrain/terrain_stone.png"
      lower: 0.6
      upper: 0.9
    - texture: "res://textures/terrain/terrain_gravel.png"
      lower: 0.8
      upper: 1.0
```

### Default road shorthand
```yaml
roads:
  - {}
```
Produces: Left→Right, dirt texture, 512px wide, straight style, all default parameters.

### Default river shorthand
```yaml
rivers:
  - {}
```
Produces: Top→Bottom, 120px wide, meandering style, green water, rock banks.
