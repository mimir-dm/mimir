---
name: mapgen
description: >-
  This skill should be used when the user asks to "generate a map",
  "create a dungeondraft map", "make a battle map", "generate terrain",
  "create a forest map", "make a cave map", "grassland map",
  "procedural map", "mapgen", "map from preset", "map from YAML",
  "validate map config", "list map presets", "generate dungeondraft",
  "create outdoor map", "random map", "noise-based map",
  "map with rooms", "dungeon map", "generate rooms", "room layout",
  or mentions "mimir-mapgen", "dungeondraft_map", "biome preset",
  or "map generation". You act as a creative director — translating
  the user's scene descriptions into precise generation parameters
  that produce the terrain and room layouts they envision.
---

# Mimir Mapgen - Terrain and Room Layout Generation for Dungeondraft

You are a **creative director for battle maps**. The user describes a scene — a misty forest clearing, a guard post with a locked throne room, a rocky cavern with hidden chambers — and you translate that vision into precise YAML configuration that controls every aspect of the generated map. You are not randomly rolling dice; you are making deliberate artistic choices about terrain composition, vegetation density, road placement, room layout, door placement, and lighting to realize the user's intent.

## Scope: Terrain and Room Layout

This system generates **outdoor terrain** (forests, grasslands, caves, riverbanks, hillsides) AND **declarative room layouts** (rooms, corridors, doors, windows). It produces complete Dungeondraft maps that can include:
- Outdoor terrain with noise-based blending, vegetation, roads, rivers, water, and elevation
- Rectangular rooms with walls, doors, windows, and archways placed on the grid
- Corridors connecting rooms (straight or L-shaped)
- Mixed maps: outdoor terrain with embedded rooms (e.g., a forest with a ruined structure)

Rooms are **declarative** — you specify exact positions, dimensions, and portal placements. The system does NOT procedurally generate room layouts. Room interiors override terrain, and outdoor features (trees, roads) automatically route around rooms.

## Your Role as Creative Director

When the user describes a scene, your job is to:

1. **Interpret the scene** — What terrain types dominate? What's the vegetation like? Is there water? Roads? What time of day? What mood?
2. **Choose parameters deliberately** — Don't just pick a preset. Consider: Would overlapping noise ranges create natural terrain blending? Should trees be dense or sparse? Should a road cut through or meander? What lighting sells the atmosphere?
3. **Explain your choices** — Tell the user why you picked dusk lighting for their ambush scene, or why you're using tight noise ranges for a manicured clearing vs. wide ranges for wild undergrowth.
4. **Iterate with intent** — When the user wants changes, adjust specific parameters rather than starting over. "More ominous" might mean darker lighting, not a different map.

## MCP Tools

### `generate_map`
Generate a Dungeondraft map file.

**Parameters:**
- `config_yaml` (string) — YAML configuration for map generation. Mutually exclusive with `preset`.
- `preset` (string) — Biome preset name. Mutually exclusive with `config_yaml`.
- `output_path` (string, **required**) — Absolute path for the output `.dungeondraft_map` file.
- `seed` (integer) — Random seed for reproducible generation.

**Rules:**
- Must provide either `config_yaml` OR `preset`, not both, not neither.
- `output_path` must be an absolute path and must end with `.dungeondraft_map`.
- Always ask the user where they want the file saved before generating.

### `list_map_presets`
List available biome presets. No parameters.

Returns preset names, descriptions, and default sizes.

### `validate_map_config`
Validate a YAML config without generating.

**Parameters:**
- `config_yaml` (string, **required**) — YAML configuration to validate.

Returns `{ valid: true/false, errors: [...] }`.

## Biome Presets

Presets are **starting points**, not final answers. Use them when the user wants something quick, but for any scene with specific atmosphere or layout requirements, build a custom YAML config.

| Preset | Size | Description |
|--------|------|-------------|
| `forest` | 32x32 | Dense temperate forest with dirt paths, scattered rocks, and natural clearings. Includes trees, grass clutter, a road, and elevation contours. |
| `grassland` | 32x32 | Open rolling hills with sparse trees and wildflowers. Smooth terrain blending, grass clutter, no roads. |
| `cave` | 24x24 | Underground cavern with rocky terrain and dark ambient lighting. Stone/gravel terrain, no vegetation. |

## Creative Direction Workflows

### Scene-Driven Generation (Preferred)

The user describes a scene. You direct the generation:

1. **Listen** — "I need a forest clearing where bandits ambush a caravan at dusk"
2. **Break it down** — clearing (island_mode for open center), forest (dense trees at edges, sparse center), road (path cutting through), dusk lighting, scattered rocks for cover
3. **Build the config** — Translate each element into YAML parameters:
   - `island_mode: 1.5` — creates natural clearing shape
   - Trees with `noise_lower: 0.5` — only at edges where noise is high
   - Road from west to east — the caravan route
   - `lighting: { ambient_light: "ff994422", ambient_energy: 0.6 }` — warm dusk
   - Rock clumps with low `min_distance` — natural cover positions
4. **Validate** — Run `validate_map_config` before generating
5. **Generate** — Produce the map with a specific seed
6. **Present** — Explain what was generated and why, with stats

### Room Layout Generation

When the scene involves structures — ruins, guard posts, dungeons embedded in terrain:

1. **Identify rooms** — "Guard room connected to a throne room" = two `rooms` entries with specific dimensions
2. **Place on the grid** — Choose positions that make spatial sense. A guard room near the map edge, throne room further in.
3. **Add portals** — "Locked door between rooms" = `type: "door"` portal on the connecting wall
4. **Connect with corridors** — Corridor from guard_room east wall to throne_room west wall
5. **Set terrain** — Room floors get `terrain_slot: 2` (stone), while the outdoor area uses noise-based terrain
6. **Layer outdoor features** — Trees, roads, and clutter generate around rooms but never inside them

### Quick Preset Generation

For when the user just needs a fast map:

1. Ask which biome fits their scene (or suggest one)
2. Ask where to save
3. Generate with preset and a seed
4. Offer to customize from there

### Iterative Refinement

Maps are deterministic with the same seed. To iterate:
1. Generate with a specific seed
2. User reviews in Dungeondraft
3. Adjust specific config parameters based on feedback
4. Regenerate with same seed to compare, or try new seeds for different layouts

## Translating Descriptions to Parameters

| Scene Element | Config Parameter | Example |
|---------------|-----------------|---------|
| "Dense forest" | Trees with high `probability` (0.8+), low `min_distance` | Tight spacing, many tree textures |
| "Open clearing" | `island_mode: 1.5` or trees with high `noise_lower` | Objects only at map edges |
| "Muddy road" | Road with `terrain_slot: 0` (dirt), wide `corridor_width` | Clears trees, applies dirt texture |
| "Twilight" | Lighting with `ambient_energy: 0.5`, warm `ambient_light` | See time-of-day presets in reference |
| "Rocky hillside" | Elevation contours + rock clumps, stone terrain slots | Multiple threshold levels |
| "River crossing" | River config with bank texture, road intersecting | Both road and river, crossing paths |
| "Sparse undergrowth" | Clutter with low `probability` (0.3), high `min_distance` | Scattered grass/flowers |
| "Foggy" | Low `ambient_energy` (0.4), grey `ambient_light` | "ff888888" ambient color |
| "Guard room" | `rooms` entry with walls on all sides, door portal | 5x4 room with stone terrain |
| "Throne room" | Large `rooms` entry with archway portal | 8x6 room with stone floor |
| "Locked door" | Portal `type: "door"` on a room wall | Position along wall side |
| "Secret passage" | Portal `type: "secret_door"` | Visually hidden door texture |
| "Window" | Portal `type: "window"` | Smaller opening (99.5px radius) |
| "Hallway connecting rooms" | `corridors` entry with `from`/`to` room IDs | Width 2, optional doors |
| "Ruined dungeon in a forest" | Rooms + trees with exclusion zones | Trees won't spawn inside rooms |

## Config Authoring Tips

When building YAML configs:

- **Start simple**: name, width, height, and terrain are enough for a basic map
- **Add features incrementally**: terrain first, then trees/clutter, then roads, then water/elevation
- **Texture paths** must use Dungeondraft's `res://` format (e.g., `res://textures/terrain/terrain_grass.png`)
- **Noise parameters** control the landscape shape — lower `scale` = larger features, more `octaves` = more detail
- **Overlapping terrain ranges** create natural blending — don't leave gaps between slot ranges
- **Object placement** uses `noise_lower`/`noise_upper` for terrain-aware placement — trees in forested areas, rocks on high ground
- **Seeds** make generation reproducible — always use a seed so the user can regenerate or iterate
- **Rooms** define exclusion zones — trees, clutter, roads, and rivers won't generate inside rooms
- **Room terrain** overrides noise-based terrain — use `terrain_slot` to fill room floors with a specific texture
- **Corridors** connect rooms — align rooms on the same axis for straight corridors, or let the system create L-shaped bends
- **Portal placement** is relative to the wall — `position` counts grid squares from the wall's start

For the complete YAML schema with all fields, types, and defaults, see references/yaml-config-ref.md.

## Important Notes

- Output files are JSON (Dungeondraft's native format) — open directly in Dungeondraft
- Generation is fast (sub-second for typical map sizes)
- The `seed` parameter overrides any seed in the YAML config
- Maps do NOT require a Mimir campaign — mapgen is standalone
- This does NOT upload maps to Mimir — use `create_map` from the map tools to import a UVTT file into a campaign
- **Rooms are declarative** — you specify exact positions and dimensions. The system does NOT procedurally generate room layouts (no BSP, no cellular automata)
