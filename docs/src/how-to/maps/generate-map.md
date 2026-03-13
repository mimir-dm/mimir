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

Output files open directly in Dungeondraft for further editing or UVTT export.

## Quick Start: Using a Preset

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

Some presets have aliases (e.g., `tropical_island` for `island_tropical`, `pond` for `lake`).

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

trees:
  - tree:
      textures:
        - "res://textures/objects/more_trees/oak_01.png"
        - "res://textures/objects/more_trees/oak_02.png"
      min_distance: 500.0
      noise_lower: 0.4
      noise_upper: 0.75
      probability: 0.5

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

- [Upload a Map](./upload-map.md) — Import maps into Mimir modules
- [Mapgen Reference](../../reference/mapgen.md) — Full YAML schema and configuration details
