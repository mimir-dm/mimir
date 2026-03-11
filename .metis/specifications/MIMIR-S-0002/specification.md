---
id: dungeondraft-asset-reference
level: specification
title: "Dungeondraft Asset Reference Catalog"
short_code: "MIMIR-S-0002"
created_at: 2026-03-11T21:11:42.832369+00:00
updated_at: 2026-03-11T21:11:42.832369+00:00
parent: MIMIR-I-0058
blocked_by: []
archived: false

tags:
  - "#specification"
  - "#phase/discovery"


exit_criteria_met: false
initiative_id: NULL
---

# Dungeondraft Asset Reference Catalog

Catalog of known Dungeondraft default asset paths and third-party asset pack conventions. Used by `mimir-mapgen` biome presets to reference correct asset paths without requiring users to know exact paths.

## Overview

Dungeondraft assets are referenced via Godot `res://` resource paths. There are two asset sources:

1. **Default assets** — shipped with Dungeondraft, always available
2. **Third-party packs** — installed by users, referenced via pack ID

This catalog documents known default asset paths extracted from working configs and maps. It will be embedded in `mimir-mapgen` as a lookup table for biome presets.

## Asset Path Conventions

### Default Assets
```
res://textures/{category}/{subcategory}/{filename}.png
```

### Third-Party Pack Assets
```
res://packs/{packId}/textures/{category}/{filename}.png
```

Where `{packId}` is the 8-character alphanumeric ID from the pack manifest (e.g., `XpMDQfBe`).

## 14 Asset Categories

As defined by the Dungeondraft modding API:

| Category | Description | Used In |
|----------|-------------|---------|
| Terrain | Ground textures | Terrain splat map slots |
| Objects | Props, trees, clutter | Object placement |
| Paths | Road/river/cliff textures | Path drawing |
| Patterns | Repeating fill textures | Pattern polygons |
| Patterns Colorable | Tintable pattern textures | Pattern polygons |
| Walls | Wall segment textures | Wall drawing |
| Portals | Door/window textures | Portal placement |
| Roofs | Roof textures | Roof drawing |
| Caves | Cave ground textures | Cave brush |
| Materials | Surface materials | Material brush |
| Lights | Light textures/cookies | Light placement |
| Simple Tiles | Single-tile floor textures | Tile placement |
| Smart Tiles | Auto-tiling floor textures | Smart tile placement |
| Smart Tiles Double | Double-layer smart tiles | Smart tile placement |

## Default Terrain Textures

```
res://textures/terrain/terrain_dirt.png
res://textures/terrain/terrain_dry_grass.png
res://textures/terrain/terrain_grass.png
res://textures/terrain/terrain_gravel.png
res://textures/terrain/terrain_moss.png
res://textures/terrain/terrain_rocky.png
res://textures/terrain/terrain_sand.png
res://textures/terrain/terrain_snow.png
res://textures/terrain/terrain_stone.png
res://textures/terrain/terrain_water.png
```

## Default Object Textures

### Trees
```
res://textures/objects/vegetation/trees/tree_big_green_01.png
res://textures/objects/vegetation/trees/tree_big_green_02.png
res://textures/objects/vegetation/trees/tree_big_green_03.png
res://textures/objects/vegetation/trees/tree_branch_01.png
res://textures/objects/vegetation/trees/tree_branch_02.png
res://textures/objects/vegetation/trees/tree_branch_03.png
res://textures/objects/vegetation/trees/tree_green_simple_01.png
res://textures/objects/vegetation/trees/tree_green_simple_02.png
res://textures/objects/vegetation/trees/tree_green_simple_03.png
res://textures/objects/vegetation/trees/tree_green_simple_04.png
res://textures/objects/vegetation/trees/pine_tree_01.png
res://textures/objects/vegetation/trees/pine_tree_02.png
res://textures/objects/vegetation/trees/pine_tree_03.png
```

### Ferns
```
res://textures/objects/vegetation/ferns/fern_01.png
res://textures/objects/vegetation/ferns/fern_02.png
res://textures/objects/vegetation/ferns/fern_03.png
res://textures/objects/vegetation/ferns/fern_04.png
res://textures/objects/vegetation/ferns/fern_05.png
res://textures/objects/vegetation/ferns/fern_06.png
```

### Grass
```
res://textures/objects/vegetation/grass/grass_14.png
res://textures/objects/vegetation/grass/grass_15.png
res://textures/objects/vegetation/grass/grass_16.png
res://textures/objects/vegetation/grass/grass_17.png
res://textures/objects/vegetation/grass/grass_18.png
res://textures/objects/vegetation/grass/grass_19.png
res://textures/objects/vegetation/grass/grass_20.png
res://textures/objects/vegetation/grass/grass_21.png
res://textures/objects/vegetation/grass/grass_24.png
res://textures/objects/vegetation/grass/grass_25.png
res://textures/objects/vegetation/grass/grass_26.png
```

### Flowers
```
res://textures/objects/vegetation/flowers/flowers_01.png
res://textures/objects/vegetation/flowers/flowers_02.png
res://textures/objects/vegetation/flowers/flowers_03.png
res://textures/objects/vegetation/flowers/flowers_04.png
res://textures/objects/vegetation/flowers/flowers_05.png
```

### Boulders
```
res://textures/objects/clutter/boulders/boulder_01.png
res://textures/objects/clutter/boulders/boulder_04.png
res://textures/objects/clutter/boulders/boulder_06.png
res://textures/objects/clutter/boulders/boulder_07.png
res://textures/objects/clutter/boulders/boulder_08.png
res://textures/objects/clutter/boulders/boulder_09.png
res://textures/objects/clutter/boulders/boulder_10.png
```

### Rubble
```
res://textures/objects/clutter/rubble/rubble_07.png
res://textures/objects/clutter/rubble/rubble_08.png
res://textures/objects/clutter/rubble/rubble_09.png
res://textures/objects/clutter/rubble/rubble_10.png
res://textures/objects/clutter/rubble/rubble_11.png
res://textures/objects/clutter/rubble/rubble_12.png
res://textures/objects/clutter/rubble/rubble_13.png
res://textures/objects/clutter/rubble/rubble_14.png
res://textures/objects/clutter/rubble/rubble_15.png
```

## Default Path Textures

```
res://textures/paths/cliff.png
res://textures/paths/dirt_path.png
res://textures/paths/stone_path.png
```

## Default Grid Textures

```
res://textures/grid/dotted_line.png
res://textures/grid/solid_line.png
```

## Known Third-Party Packs

Commonly used packs referenced in community configs:

| Pack Name | ID | Author | Notable Assets |
|-----------|-----|--------|---------------|
| Krager's Shadow & Light Pack | `XpMDQfBe` | Krager | Colorable gradients, shadow paths, double shadow paths |
| AoA_FX_CloudsAndWater | `H0hDYdyG` | AoA | Whitecaps water paths, cloud effects |
| (Various plant packs) | `jeeh3kgP` | Various | Additional plant/vegetation objects |

### Notable Pack Asset Paths

**Krager's Shadow & Light (`XpMDQfBe`)**:
```
res://packs/XpMDQfBe/textures/objects/ColorableGradient/Circle/Large/ColorableGradientFullCircleLarge75.png
res://packs/XpMDQfBe/textures/paths/Shadow/DoubleShadowPath/DoubleShadowPathMed25.png
```

**AoA Clouds & Water (`H0hDYdyG`)**:
```
res://packs/H0hDYdyG/textures/paths/WhitecapsPathB_2.png
```

## Biome Preset Asset Mapping

Recommended default asset selections per biome:

### Forest
- **Terrain**: terrain_moss (primary), terrain_dirt (under trees), terrain_dry_grass (transition), terrain_rocky (low)
- **Trees**: tree_big_green_03, tree_branch_01-03, tree_green_simple_04, pine_tree_02
- **Undergrowth**: fern_01-06, grass_14-21, grass_24-26
- **Clutter**: boulder_01-10, rubble_07-15, flowers_01-05
- **Paths**: cliff.png (cliffs/hills)

### Desert (TODO — needs asset discovery)
- **Terrain**: terrain_sand, terrain_rocky, terrain_stone, terrain_dirt

### Swamp (TODO — needs asset discovery)
- **Terrain**: terrain_moss, terrain_water, terrain_dirt, terrain_grass

### Cave (TODO — needs asset discovery)
- **Terrain**: terrain_stone, terrain_rocky, terrain_gravel, terrain_dirt

## Constraints

- Default asset paths are stable across Dungeondraft versions but not officially documented
- Third-party pack IDs are random and cannot be predicted — must be declared in config
- The `asset_manifest` in the map header must list ALL third-party packs referenced by any asset in the map
- Biome presets should default to only default assets, with optional pack asset overrides
- This catalog is incomplete — additional paths should be discovered by examining more community maps and configs