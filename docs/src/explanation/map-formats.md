# Map Formats and Generation

How Mimir handles battle maps — from uploaded files to procedurally generated terrain.

## Two Ways to Get Maps

Mimir supports two approaches to maps:

1. **Upload** an existing map file (UVTT or image)
2. **Generate** a map procedurally from a biome preset or YAML configuration

Both produce maps you can use in play mode with tokens, fog of war, and lighting.

## The UVTT Format

UVTT (Universal Virtual Tabletop) is the preferred map format. Created by [Dungeondraft](https://dungeondraft.net/) and adopted by other tools, a `.dd2vtt` or `.uvtt` file bundles:

- **Map image** — The rendered map as a PNG
- **Grid data** — Cell size, offset, and dimensions so tokens snap correctly
- **Walls** — Line segments that block line of sight for fog of war
- **Doors** — Interactive wall segments that can be opened/closed
- **Lights** — Light source positions with color and intensity

When you upload a UVTT file, Mimir extracts all of this automatically. Fog of war, line-of-sight calculations, and lighting all work out of the box because the wall data is embedded in the file.

## Image Files

You can also upload plain images (PNG, JPG, WebP) as maps. These work for basic token placement but lack wall data, so fog of war operates as a simple reveal grid rather than true line-of-sight.

Mimir defaults to 70 pixels per grid square for image files. If your map image uses a different grid size, token snapping may not align perfectly. For best results, use UVTT format.

## Procedural Map Generation

Mimir includes a map generator (`mimir-mapgen`) that creates Dungeondraft-compatible maps from a declarative configuration. Rather than hand-drawing every map, you describe what you want and the generator builds it.

### Biome Presets

The fastest path: choose a biome like "forest," "cave," "desert," or "arctic" and the generator produces terrain with appropriate textures, vegetation, and features. Presets encode the decisions a map artist would make — which materials to blend, how terrain should transition, what vegetation density looks natural.

### YAML Configuration

For full control, write a YAML configuration that describes the map you want declaratively — dimensions, terrain layers, room layouts, features, and lighting. The generator translates this into the output format. See the [Mapgen Reference](../reference/mapgen.md) for the complete YAML schema.

### Output Format

The generator outputs `.dungeondraft_map` files — the same format Dungeondraft uses natively. This means generated maps can be:

- Used directly in Mimir with full fog of war and lighting support
- Opened in Dungeondraft for hand-editing and polish
- Shared with other tools that support the format

### Standalone CLI

The map generator also works as a standalone command-line tool, independent of the main Mimir application. This is useful for batch generation or scripting.

## Why Dungeondraft Format?

Mimir chose the Dungeondraft format for several reasons:

- **Rich metadata.** Walls, doors, lights, and grid data are all part of the format — exactly what a VTT needs.
- **Ecosystem.** Dungeondraft is widely used in the TTRPG community, so maps created there work in Mimir and vice versa.
- **Editability.** Generated maps can be refined in Dungeondraft before use, bridging procedural and hand-crafted approaches.

## The Trade-Offs

Each approach has different strengths. UVTT files give the richest play experience (fog of war, lighting, doors) but require a map-making tool. Images are the most accessible — any map image works — but lose advanced features. Procedural generation is fastest for outdoor terrain and random encounters, but produces maps that may need hand-editing in Dungeondraft for complex interior layouts.

The choice often depends on the encounter: a carefully planned boss fight deserves a hand-crafted UVTT map; a random wilderness encounter can use a generated biome; a map from an online artist works fine as an image with basic fog of war.

## See Also

- [Upload a Map](../how-to/maps/upload-map.md) — Step-by-step upload guide
- [Generate Maps](../how-to/maps/generate-map.md) — Using the map generator
- [Mapgen Reference](../reference/mapgen.md) — Full YAML configuration reference
- [Vision & Lighting](../reference/vision-and-lighting.md) — How fog of war and light sources work
