# Install and Use Mapgen as a Standalone Tool

`mimir-mapgen` is a self-contained procedural map generator that creates [Dungeondraft](https://dungeondraft.net/)-compatible `.dungeondraft_map` files. It has no dependency on Mimir's database, UI, or MCP server — you can build it independently and use it as a command-line tool anywhere.

Note that Mimir itself cannot open `.dungeondraft_map` files directly — Mimir's map upload accepts only UVTT (`.dd2vtt`, `.uvtt`) and image files. To use a generated map in Mimir, open it in Dungeondraft (a paid tool) and export it as Universal VTT.

## Prerequisites

- **Rust toolchain** (1.75+) — install via [rustup](https://rustup.rs/) (only needed when building from source)
- **Dungeondraft** — to open and edit generated maps (or export to UVTT for Mimir, Foundry VTT, or Roll20)

## Installation

### From GitHub Releases

Pre-built `mimir-mapgen` binaries for each platform are attached to [Mimir's GitHub releases](https://github.com/mimir-dm/mimir/releases). Download the binary for your platform and put it on your `$PATH`.

### From Source

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

Other useful commands:

```bash
mimir-mapgen list-presets             # show all 12 biome presets
mimir-mapgen validate config.yaml     # check a YAML config without generating
mimir-mapgen generate --preset forest --seed 42 -o forest.dungeondraft_map   # reproducible output
```

See the [Mapgen Reference](../../reference/mapgen.md) for the full CLI flags, the complete preset table, and every YAML schema field.

## Worked Example: Island Fort

A custom YAML config combining island mode, a walled room, water, and trees:

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
      scale_min: 0.8
      scale_max: 1.2
      layer: 300
      random_rotation: true
      random_mirror: true

water:
  threshold: 0.85
  deep_color: "ff1a6b5a"
  shallow_color: "ff30b89a"
  blend_distance: 60.0
  min_contour_points: 6
  smooth_iterations: 5
  pixels_per_cell: 64.0
```

Generate it:

```bash
mimir-mapgen generate island_fort.yaml -o island_fort.dungeondraft_map
```

How it works:

- `island_mode` applies a radial falloff so the center stays low (land) and edges rise — combined with `water.threshold: 0.85`, the high-noise edges become ocean.
- The four `terrain.slots` blend grass through sand as noise increases, so beaches appear near the water.
- The `fort` room is placed in grid squares; trees and other outdoor features automatically route around it.
- The `trees` entry is a full object config — all placement fields (`min_distance` through `random_mirror`) are required.

For the many other sections you can add (roads, rivers, lakes, lights, polygons, pattern fills, materials, and more), see the [Mapgen Reference](../../reference/mapgen.md).

## Iterative Workflow

Maps are deterministic — the same seed + config always produces the same result. Use this for rapid iteration:

1. Generate with a fixed seed
2. Open in Dungeondraft and review
3. Tweak config parameters (tree density, road width, room positions)
4. Regenerate with the same seed to see only your changes
5. Try different seeds to explore different terrain layouts

Run `mimir-mapgen validate config.yaml` after each edit to catch schema and layout errors before generating.

## Example Configs

The `crates/mimir-mapgen/examples/` directory contains YAML configs and pre-generated maps for every biome preset plus multi-feature configs like `tavern_river.yaml` and `island_fort.yaml`.

## See Also

- [Mapgen Reference](../../reference/mapgen.md) — Full CLI, preset table, and YAML schema with all fields documented
- [Dungeondraft Texture Catalog](../../reference/dd-texture-catalog.md) — Available `res://` texture paths
- [Generate Maps (Mimir Integration)](./generate-map.md) — Using mapgen through Mimir's UI and MCP tools
