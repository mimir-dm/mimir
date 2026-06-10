# Generate a Map

Use Mimir's mapgen tool to procedurally generate Dungeondraft-format maps from biome presets or custom YAML configurations.

## What Mapgen Creates

Mapgen generates `.dungeondraft_map` files containing:
- Noise-based terrain with blended textures
- Trees, grass, and other vegetation
- Roads and rivers that follow the terrain
- Rooms with walls, doors, and windows
- Water bodies, elevation contours, and lighting
- Polygon-based layouts for irregular room shapes

Output files open directly in Dungeondraft for further editing or UVTT export. Mimir cannot open `.dungeondraft_map` files directly — to use a generated map in Mimir, open it in Dungeondraft (a paid tool) and export it as Universal VTT.

## Quick Start: Using a Preset

**Prerequisite:** the `mimir-mapgen` binary is not bundled with the Mimir app. Either download a pre-built binary from [GitHub Releases](https://github.com/mimir-dm/mimir/releases) (attached to each release for every platform) or build it from source with `cargo build -p mimir-mapgen --release`. See [Install and Use Mapgen as a Standalone Tool](./mapgen-standalone.md) for details.

Generate a map from one of 12 built-in biome presets:

```bash
mimir-mapgen generate --preset forest --output my-forest.dungeondraft_map
```

Use a specific seed for reproducible results:

```bash
mimir-mapgen generate --preset forest --seed 42 --output my-forest.dungeondraft_map
```

## List Available Presets

```bash
mimir-mapgen list-presets
```

Example presets:

| Preset | Size | Description |
|--------|------|-------------|
| `forest` | 32x32 | Dense temperate forest with dirt paths, scattered rocks, and natural clearings |
| `cave` | 24x24 | Underground cavern with rocky terrain and dark ambient lighting |
| `forest_river` | 32x32 | Dense forest bisected by a meandering river with rocky banks |

See the [Mapgen Reference](../../reference/mapgen.md#biome-presets) for the full table of all 12 presets and their aliases.

## Custom YAML Configs

For full control, write a YAML configuration file:

```yaml
name: "Forest Clearing"
width: 32
height: 32
seed: 42

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
  blend_width: 0.05
  smooth_blending: true

trees:
  - tree:
      textures:
        - "res://textures/objects/more_trees/oak_01.png"
        - "res://textures/objects/more_trees/oak_02.png"
      min_distance: 500.0
      noise_lower: 0.4
      noise_upper: 0.75
      probability: 0.5
      scale_min: 0.8
      scale_max: 1.2
      layer: 300
      random_rotation: true
      random_mirror: true

roads:
  - {}
```

Generate from the config:

```bash
mimir-mapgen generate config.yaml --output clearing.dungeondraft_map
```

## Validate a Config

Check a config for errors without generating:

```bash
mimir-mapgen validate config.yaml
```

## Iterating on Maps

Maps are deterministic — the same seed and config produce the same map. To iterate:

1. Generate with a specific seed
2. Open in Dungeondraft and review
3. Adjust config parameters
4. Regenerate with the same seed to compare changes
5. Try different seeds for different terrain layouts

## Using via AI Assistant

Mapgen is also available as an MCP tool (`generate_map`) through Mimir's AI assistant integration. Describe the scene you want — "a misty forest clearing with a road" — and the assistant will build the YAML config and generate the map for you.

## See Also

- [Install and Use Mapgen as a Standalone Tool](./mapgen-standalone.md) — Installation and a full worked example
- [Upload a Map](./upload-map.md) — Import maps into Mimir modules (UVTT or image files)
- [Mapgen Reference](../../reference/mapgen.md) — Full YAML schema and configuration details
