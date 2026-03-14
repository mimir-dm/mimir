# Install and Use Mapgen as a Standalone Tool

`mimir-mapgen` is a self-contained procedural map generator that creates [Dungeondraft](https://dungeondraft.net/)-compatible `.dungeondraft_map` files. It has no dependency on Mimir's database, UI, or MCP server — you can build it independently and use it as a command-line tool anywhere.

## Prerequisites

- **Rust toolchain** (1.75+) — install via [rustup](https://rustup.rs/)
- **Dungeondraft** — to open and edit generated maps (or export to UVTT for Foundry VTT / Roll20)

## Installation

### From Source (Recommended)

Clone the repository and build just the mapgen crate:

```bash
git clone https://github.com/mimir-dm/mimir.git
cd mimir
cargo build -p mimir-mapgen --release
```

The binary is at `target/release/mimir-mapgen`. Copy it somewhere on your `$PATH`:

```bash
cp target/release/mimir-mapgen /usr/local/bin/
```

### Verify Installation

```bash
mimir-mapgen --help
```

## Quick Start

Generate a map from a built-in preset:

```bash
mimir-mapgen generate --preset forest -o forest.dungeondraft_map
```

Open `forest.dungeondraft_map` in Dungeondraft. Done.

## Commands

### `generate` — Create a Map

From a biome preset:

```bash
mimir-mapgen generate --preset lake -o lake.dungeondraft_map
```

From a custom YAML config:

```bash
mimir-mapgen generate my-config.yaml -o my-map.dungeondraft_map
```

With a specific seed for reproducible output:

```bash
mimir-mapgen generate --preset forest --seed 42 -o forest.dungeondraft_map
```

| Flag | Short | Description |
|------|-------|-------------|
| `--output` | `-o` | Output file path (default: `output.dungeondraft_map`) |
| `--seed` | `-s` | Random seed (overrides config file seed) |
| `--preset` | `-p` | Use a built-in biome preset (mutually exclusive with config file) |

You must provide either a config file **or** `--preset`, not both.

### `list-presets` — Show Available Presets

```bash
mimir-mapgen list-presets
```

| Preset | Size | Description |
|--------|------|-------------|
| `forest` | 32x32 | Dense temperate forest with dirt paths, scattered rocks, and natural clearings |
| `grassland` | 32x32 | Open rolling hills with sparse trees and wildflowers |
| `cave` | 24x24 | Underground cavern with rocky terrain and dark ambient lighting |
| `desert` | 32x32 | Arid sandy wasteland with rocky outcrops and sparse scrub |
| `lake` | 32x32 | Tranquil woodland pond with grassy shores and scattered trees |
| `ice_lake` | 32x32 | Frozen lake with cracked ice, snow-covered shores, and frigid water |
| `arctic` | 32x32 | Frozen tundra with snow drifts, exposed rock, and harsh conditions |
| `island_tropical` | 32x32 | Tropical island with sandy beaches, palm trees, and warm ocean |
| `island_forest` | 32x32 | Forested island in a lake with dirt shores and dense tree cover |
| `island_arctic` | 32x32 | Snow-covered island surrounded by frigid dark water |
| `swamp` | 32x32 | Dark, murky wetland with stagnant water, dead trees, and dim lighting |
| `forest_river` | 32x32 | Dense forest bisected by a meandering river with rocky banks |

Presets accept aliases: `lake`/`pond`, `swamp`/`marsh`/`bog`, `island_tropical`/`tropical_island`. Hyphens work too (`forest-river`).

### `validate` — Check a Config Without Generating

```bash
mimir-mapgen validate config.yaml
```

Reports field-specific errors so you can fix issues before generation.

## Writing a Custom Config

Configs are YAML files. A minimal example:

```yaml
name: "Forest Clearing"
width: 32
height: 32

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

This produces a 32x32 grid map with blended terrain — no trees, roads, or rooms. Add sections to layer in features.

### Adding Trees and Vegetation

```yaml
trees:
  - tree:
      textures:
        - "res://textures/objects/more_trees/oak_01.png"
        - "res://textures/objects/more_trees/oak_02.png"
      min_distance: 500.0
      noise_lower: 0.4
      noise_upper: 0.75
      probability: 0.5

clutter:
  - textures:
      - "res://textures/objects/vegetation/grass/grass_01.png"
    min_distance: 80.0
    noise_lower: 0.2
    noise_upper: 0.7
    probability: 0.6
```

Trees are placed via Poisson Disc sampling — `min_distance` controls spacing, `noise_lower`/`noise_upper` restrict placement to specific terrain zones.

### Adding a Road

```yaml
roads:
  - from: Left
    to: Right
    width: 512.0
    style: straight
```

All road fields have defaults, so even `roads: [{}]` produces a valid road. Style options: `straight` (noise-following) or `meandering` (sinusoidal S-curves). Roads automatically route around rooms.

### Adding a River

```yaml
rivers:
  - from: Top
    to: Bottom
    width: 120.0
    style: meandering
    deep_color: "ff3aa19a"
    shallow_color: "ff3ac3b2"
```

Rivers generate a water polygon with bank paths on each side. Very wide rivers (1000+ pixels) can simulate coastlines.

### Adding Rooms and Corridors

```yaml
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

  - id: "kitchen"
    x: 19
    y: 14
    width: 4
    height: 3
    terrain_slot: 0

corridors:
  - from: "tavern"
    from_wall: "east"
    to: "kitchen"
    to_wall: "west"
    width: 2
    portals:
      - end: "from"
        type: archway
        width: 1
```

Coordinates are in grid squares. Outdoor features (trees, roads, rivers) automatically avoid room interiors.

### Adding Polygon Shapes

For irregular rooms (L-shapes, ovals, etc.), use polygons:

```yaml
polygons:
  - id: "room_a"
    points:         # clockwise vertex order
      - [3, 6]
      - [9, 6]
      - [9, 14]
      - [3, 14]
    terrain_slot: 3
    portals:
      - edge: 0     # edge 0 = points[0] -> points[1]
        position: 0.5
        type: door
```

Adjacent polygons that share an edge automatically get an open connection (shared wall removed). Overlapping polygons merge into a single outer wall.

### Adding Water Bodies

```yaml
water:
  threshold: 0.75
  deep_color: "ff3aa19a"
  shallow_color: "ff3ac3b2"
  smooth_iterations: 2
```

Water fills areas where noise exceeds the threshold. Combine with `island_mode` for lake-surrounded islands.

### Elevation Contours

```yaml
elevation:
  levels:
    - threshold: 0.65
      texture: "res://textures/paths/path_rocks.png"
      width: 12.0
```

### Island Mode

Add `island_mode` to apply a radial falloff — center stays low, edges rise. Creates natural island or clearing shapes:

```yaml
island_mode: 1.5    # higher = steeper falloff at edges
```

### Lighting

```yaml
lighting:
  ambient_light: "ffffffff"
  ambient_energy: 1.0
  shadow_color: "66000000"
```

Time-of-day presets for reference:

| Preset | Ambient Color | Energy | Mood |
|--------|--------------|--------|------|
| Dawn | Warm gold | 0.7 | Early morning |
| Day | White | 1.0 | Bright daylight |
| Dusk | Orange-red | 0.6 | Sunset |
| Night | Blue | 0.3 | Darkness |
| Underground | Dark grey | 0.2 | Cave / dungeon |

## Full Example: Island Fort

A config combining island mode, rooms, water, trees, and clutter:

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

water:
  threshold: 0.85
  deep_color: "ff1a6b5a"
  shallow_color: "ff30b89a"
  blend_distance: 60.0
  smooth_iterations: 5
```

Generate it:

```bash
mimir-mapgen generate island_fort.yaml -o island_fort.dungeondraft_map
```

## Iterative Workflow

Maps are deterministic — the same seed + config always produces the same result. Use this for rapid iteration:

1. Generate with a fixed seed
2. Open in Dungeondraft and review
3. Tweak config parameters (tree density, road width, room positions)
4. Regenerate with the same seed to see only your changes
5. Try different seeds to explore different terrain layouts

## Noise Tuning

The noise parameters control the foundation of everything — terrain blending, object placement, and path generation all read from the same noise field.

```yaml
noise:
  octaves: 6        # more octaves = more fine detail
  persistence: 0.5  # amplitude decay per octave (lower = smoother)
  lacunarity: 2.0   # frequency multiplier per octave
  scale: 0.03       # base frequency (lower = larger features)
```

**Tips:**
- Lower `scale` (0.01–0.02) for broad, sweeping terrain
- Higher `scale` (0.04–0.06) for choppy, varied terrain
- Reduce `octaves` to 3–4 for softer, less detailed maps
- Increase `persistence` toward 0.7 for rougher textures

## Output Format

Generated `.dungeondraft_map` files are JSON and open directly in Dungeondraft. From there you can:

- Fine-tune placement, add furniture, lighting, etc.
- Export to Universal VTT (`.dd2vtt`) for Foundry VTT, Roll20, or other platforms
- Export to PNG/JPEG images

## Example Configs

The `crates/mimir-mapgen/examples/` directory contains YAML configs and pre-generated maps for every biome preset plus multi-feature configs like `tavern_river.yaml` and `island_fort.yaml`.

## See Also

- [Mapgen Reference](../../reference/mapgen.md) — Full YAML schema with all fields documented
- [Dungeondraft Texture Catalog](../../reference/dd-texture-catalog.md) — Available `res://` texture paths
- [Generate Maps (Mimir Integration)](./generate-map.md) — Using mapgen through Mimir's UI and MCP tools
