---
id: declarative-dungeondraft-map
level: initiative
title: "Declarative Dungeondraft Map Generation"
short_code: "MIMIR-I-0058"
created_at: 2026-03-11T21:06:02.955630+00:00
updated_at: 2026-03-11T21:23:21.932506+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: L
initiative_id: declarative-dungeondraft-map
---

# Declarative Dungeondraft Map Generation Initiative

## Context

Mimir currently imports pre-made Dungeondraft `.dungeondraft_map` files (which are JSON internally) and displays them as map backgrounds for the VTT. Map creation is entirely manual — DMs must use the Dungeondraft GUI editor to build every map from scratch.

The project [dungeondraft-procedural-forest](https://gitlab.com/gull-rock-maps/dungeondraft-procedural-forest) demonstrates that Dungeondraft maps can be generated programmatically. It's a Ruby script that takes a declarative JSON config (biome type, terrain textures, tree assets, road behavior, Perlin noise params, etc.) and produces a complete `.dungeondraft_map` file with terrain, objects, paths, water, lighting, and hills — all driven by procedural algorithms (Perlin noise, Poisson Disc sampling, Marching Squares contouring, Bezier curves).

We want to bring this concept into Mimir as a Rust-native map generation system, allowing DMs to declaratively describe what a map should contain and have it generated automatically. This would be a new `mimir-mapgen` crate.

### Dungeondraft Map Format

See **MIMIR-S-0001** for the full reverse-engineered format specification. Key points: JSON with `header` + `world` top-level keys, Godot-style serialized types (`Vector2`, `PoolByteArray`), 256px per grid square coordinate system, 4-texture terrain splat blending, and a shared `next_node_id` counter for all entities.

### Reference Implementation Key Algorithms
- **Perlin noise** — fractal multi-octave heightmap for terrain, object gating, and contouring
- **Poisson Disc sampling** — spatially uniform random object placement with minimum distance
- **Marching Squares** — contour extraction from heightmap for water edges and hill cliffs
- **Bezier curves** — path smoothing for roads and rivers
- **Greedy pathfinding** — road/river routing that follows heightmap ridges/valleys

### Asset Reference System

See **MIMIR-S-0002** for the full asset catalog and path conventions. Key points:
- Default assets: `res://textures/{category}/{file}.png` (always available)
- Pack assets: `res://packs/{8-char-id}/textures/...` (user-installed)
- 14 asset categories (Terrain, Objects, Paths, Patterns, Walls, etc.)
- Map header `asset_manifest` declares pack dependencies
- `Script.GetAssetList(category)` only works inside DD runtime — external gen needs embedded catalog
- Strategy: ship known default paths, allow user override, future pack file parsing (see **MIMIR-A-0008**)

**API reference**: https://megasploot.github.io/DungeondraftModdingAPI/

### Inspiration Repo
- Source: `git@gitlab.com:gull-rock-maps/dungeondraft-procedural-forest.git`
- Language: Ruby (~4,500 lines across 2 files)
- Approach: procedural generation driven by declarative JSON config
- Takes a baseline `.dungeondraft_map` template and populates it with generated content

## Goals & Non-Goals

**Goals:**
- Create a `mimir-mapgen` crate that generates `.dungeondraft_map` files from declarative YAML/JSON configs
- Implement core algorithms in Rust: Perlin noise, Poisson Disc, Marching Squares, Bezier curves
- Support biome presets (forest, cave, swamp, desert, island) with configurable parameters
- Generate terrain (4-texture blending), object placement, paths (roads/rivers), water bodies, lighting, and elevation contours
- Expose map generation via MCP tool so AI agents can generate maps from natural language descriptions
- Expose via Tauri command for future UI integration
- Generated maps should be importable into both Dungeondraft (for further editing) and Mimir's VTT

**Non-Goals:**
- Real-time map editing UI in Mimir (this is generation, not an editor)
- Mimir frontend UI for map generation (MCP + CLI only)
- Rendering maps or rasterizing images (Dungeondraft is the renderer — see Pipeline below)
- Building/interior generation (outdoor biomes first)
- Asset pack management or distribution (users bring their own Dungeondraft assets)
- Reimplementing Dungeondraft's rendering engine (terrain blending, object compositing, lighting)

## Architecture

See **MIMIR-A-0008** for crate structure, dependency strategy, config format, and asset approach decisions.

### Overview

New crate `mimir-mapgen` with these layers:

```
Config (YAML/JSON) → MapSpec → Generator Pipeline → DungeondraftMap → .dungeondraft_map JSON
```

### Core Modules

1. **`format`** — Dungeondraft map file format serialization/deserialization
   - Parse and write `.dungeondraft_map` JSON
   - Type-safe representations of Vector2, PoolByteArray, PoolVector2Array
   - Object, path, pattern, water, light, terrain structures

2. **`noise`** — Procedural noise generation
   - Perlin noise (2D, fractal/multi-octave)
   - Noise map simplification and caching

3. **`distribution`** — Spatial distribution algorithms
   - Poisson Disc sampling (configurable min distance, density)
   - Noise-gated placement (only place objects where noise > threshold)

4. **`contour`** — Contour extraction
   - Marching Squares implementation
   - Contour line smoothing and simplification

5. **`curves`** — Path mathematics
   - Bezier curve generation and evaluation
   - Path smoothing for roads/rivers

6. **`terrain`** — Terrain generation
   - 4-texture terrain blending from noise map
   - Configurable boundary thresholds and blend widths

7. **`objects`** — Object placement
   - Tree placement with shadows, lights, canopy layers
   - Clump generation (primary + secondary clustered objects)
   - Clutter scattering

8. **`paths`** — Road and river generation
   - Greedy pathfinding along noise ridges/valleys
   - Road/river width, texture, bank generation
   - Object clearing along path corridors

9. **`water`** — Water body generation
   - Contour-based water polygon extraction
   - Shore/bank path generation
   - Swamp/lake/river configurations

10. **`elevation`** — Hill and cliff generation
    - Contour-based cliff path generation
    - Shadow path offsets for depth illusion

11. **`biomes`** — Preset biome configurations
    - Forest, cave, swamp, desert, island templates
    - Sensible defaults for terrain, objects, lighting per biome

12. **`pipeline`** — Orchestration
    - Config parsing and validation
    - Sequential generation pipeline (terrain → objects → paths → water → hills → lights)
    - Random seed management for reproducibility

### Config Format (YAML)

```yaml
map:
  width: 30        # grid squares
  height: 20
  pixels_per_cell: 256
  seed: 42         # optional, random if omitted

biome: forest      # preset, overridable below

terrain:
  textures:
    - res://textures/terrain/terrain_grass.png
    - res://textures/terrain/terrain_dirt.png
    - res://textures/terrain/terrain_stone.png
    - res://textures/terrain/terrain_sand.png
  boundaries: [0.3, 0.5, 0.7]
  blend_width: 0.05

noise:
  octaves: 6
  persistence: 0.5
  lacunarity: 2.0
  scale: 0.02

trees:
  enabled: true
  assets:
    - res://objects/trees/tree_oak_01.png
    - res://objects/trees/tree_oak_02.png
  density: 0.4
  min_distance: 1.5
  noise_threshold: 0.4
  shadows: true
  canopy: true

roads:
  enabled: true
  texture: res://textures/paths/path_dirt.png
  width: 2.0
  from: left
  to: right

water:
  enabled: false

lighting:
  ambient_color: "#FFE4B5"
  tree_lights: true
```

## Detailed Design

### Generation Pipeline

1. Parse config → `MapSpec` struct
2. Apply biome preset defaults, merge with user overrides
3. Generate Perlin noise heightmap at map resolution
4. Apply noise modifiers (island mode, canyon mode)
5. Generate terrain splat data from noise + texture config
6. Place objects (trees, clumps, clutter) via Poisson Disc + noise gating
7. Generate roads/rivers via pathfinding + Bezier smoothing
8. Clear objects from road/river corridors
9. Generate water bodies via Marching Squares contouring
10. Generate elevation contours for hills/cliffs
11. Place lights (tree-top, ambient scatter)
12. Serialize to `.dungeondraft_map` JSON

### End-to-End Pipeline

Dungeondraft remains the **renderer** — it owns the asset PNGs and composites the final image. We generate the map data; DD renders it; Mimir consumes the rendered output.

```
mimir-mapgen (config/AI → .dungeondraft_map JSON)
     ↓
Dungeondraft (loads map → renders with assets → exports)
     ↓
.dd2vtt (rendered PNG image + walls/lights/grid geometry)
     ↓
Mimir VTT (imports UVTT as playable map)
```

**Why we can't skip Dungeondraft**: The `.dungeondraft_map` is just data (asset refs, positions, terrain weights). Rendering requires the actual asset PNGs, splat-based terrain blending, object compositing with layer sorting, and lighting/shadow calculations. That's DD's rendering engine — reimplementing it would be massive scope and has asset licensing issues.

**Stretch goal**: Investigate whether DD supports headless/CLI export to automate the render step (see Investigation Notes below).

### UX Surfaces

No Mimir frontend UI. Two interfaces:

- **MCP tool (`generate_map`)**: Primary interface. AI agents convert natural language map descriptions into config, generate the `.dungeondraft_map` file, write to disk. Returns the file path.
- **Standalone CLI (`mimir-mapgen`)**: `generate --config forest.yaml --output map.dungeondraft_map`, `generate --biome forest --size 30x20 --seed 42`, `list-biomes`, `list-assets`. For batch generation, scripting, and development.

### Investigation: DD Headless Export

**Status**: Investigated. No true headless/CLI mode exists.

The `Exporter` class (`Global.Exporter`) can programmatically trigger UVTT export via `Start(mode=3, ppi, filepath)` on a background thread. It also exposes `ExportForVTT()`, `ExportLOS()`, `ExportPortals()`, `ExportLights()`, and `ExportImageBase64()`. However, this is an **internal modding API** — it only works from within a running Dungeondraft instance via GDScript mods.

**Possible automation path** (stretch goal, not in initial scope):
1. Write a DD mod (GDScript tool script) that watches a folder for new `.dungeondraft_map` files
2. Mod auto-loads the map and triggers `Global.Exporter.Start(3, ppi, output_path)`
3. Mimir watches the output folder and auto-imports the `.dd2vtt`

This requires DD running in the background with the mod loaded — not truly headless, but hands-free once set up. Deferred to a future iteration.

**Reference**: https://megasploot.github.io/DungeondraftModdingAPI/reference/Exporter/

## Alternatives Considered

### Wrapping the Ruby script
Could shell out to the existing Ruby script. Rejected because: adds Ruby runtime dependency, can't integrate with Mimir's type system, harder to extend, and the algorithms are straightforward to reimplement in Rust with better performance.

### Using an existing Rust noise/procgen library
Libraries like `noise-rs` exist for Perlin noise, and `poisson-disk-sampling` for spatial distribution. We should evaluate these as dependencies rather than reimplementing from scratch where mature crates exist. The Dungeondraft format layer and generation pipeline are Mimir-specific regardless.

### Web-based map editor
Building a full map editor in the browser/Tauri frontend. Rejected as a non-goal — this initiative focuses on generation, not editing. A future initiative could add visual config editing.

## Implementation Plan

### Phase 1: Format & Foundation
- Dungeondraft map format parser/writer
- Core math: Perlin noise, Poisson Disc, Marching Squares, Bezier
- Evaluate existing Rust crates vs custom implementation

### Phase 2: Terrain & Objects
- Terrain generation from noise
- Object placement (trees, clutter)
- Basic biome presets

### Phase 3: Paths & Water
- Road/river generation
- Water body generation
- Object clearing along paths

### Phase 4: Polish & Integration
- Elevation/hill contours
- Lighting
- MCP tool and Tauri command integration
- Biome preset library
- Documentation and examples