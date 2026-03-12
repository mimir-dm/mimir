---
description: Generate a procedural Dungeondraft map from a biome preset or custom YAML config
arguments:
  - name: preset_or_description
    description: "Biome preset name (forest, grassland, cave) or a description of the desired map"
    required: false
allow_override: false
---

# Generate a Dungeondraft Map

## Step 1: Determine Map Type

If the user provided a preset name (forest, grassland, cave), use it directly.

If they provided a description or no arguments, help them choose:
1. Call `list_map_presets` to show available presets
2. Ask if a preset fits or if they want a custom config
3. For custom configs, discuss: size, terrain types, vegetation density, roads/rivers, lighting, rooms/corridors

## Step 2: Choose Output Location

Ask the user where to save the `.dungeondraft_map` file. Suggest a reasonable default like their Desktop or current directory.

## Step 3: Generate

**From preset:**
```
generate_map(preset: "<name>", output_path: "<path>", seed: <random or user-specified>)
```

**From custom config:**
1. Build the YAML config based on the discussion
2. Validate first: `validate_map_config(config_yaml: "...")`
3. If valid, generate: `generate_map(config_yaml: "...", output_path: "<path>")`
4. If errors, fix and re-validate

## Step 4: Report Results

Show the generation stats:
- Walls generated (room boundaries, corridors)
- Portals generated (doors, windows)
- Objects placed (trees, clutter, clumps)
- Paths generated (roads, rivers)
- Water polygons
- Contour paths
- Output file location

Remind the user they can open the file in Dungeondraft to view and edit the map.

Offer to regenerate with a different seed or adjusted config if they want variations.
