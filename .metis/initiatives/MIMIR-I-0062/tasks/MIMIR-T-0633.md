---
id: biome-preset-updates-evaluate-and
level: task
title: "Biome preset updates: evaluate and enhance all 12 presets with lights, paths, patterns, and materials"
short_code: "MIMIR-T-0633"
created_at: 2026-03-15T00:42:49.076561+00:00
updated_at: 2026-03-15T00:42:49.076561+00:00
parent: MIMIR-I-0062
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0062
---

# Biome preset updates: evaluate and enhance all 12 presets with lights, paths, patterns, and materials

**Depends on:** MIMIR-T-0629 (lights), MIMIR-T-0630 (paths), MIMIR-T-0631 (patterns), MIMIR-T-0632 (materials)

## Objective

Evaluate all 12 biome presets and enhance them with lights, paths, patterns, and materials where appropriate. Use the Gull Rock procedural forest configs as an aesthetic reference for tuning values. The presets should demonstrate the new capabilities and produce maps that are visually competitive without manual DD finishing.

## Acceptance Criteria

- [ ] All 12 presets evaluated: Forest, Grassland, Cave, Desert, Lake/Pond, Ice Lake, Arctic, Island Tropical, Island Forest, Island Arctic, Swamp/Marsh, Forest River
- [ ] Each preset that has roads/rivers gets along-path lights
- [ ] Each preset that has trees gets with-objects lights (tree-top glow)
- [ ] Presets with elevation contours get offset shadow paths
- [ ] River/lake presets get water overlay patterns
- [ ] At minimum: Forest, Forest River, Swamp, and one Island preset fully enhanced as showcase examples
- [ ] Light density, color, intensity values tuned using Gull Rock configs as reference
- [ ] All presets still generate valid maps that open in DD
- [ ] `cargo test -p mimir-mapgen` passes
- [ ] Regenerate all example `.dungeondraft_map` files in `examples/`

## Implementation Notes

### File to modify
- `crates/mimir-mapgen/src/biomes.rs`
- `crates/mimir-mapgen/examples/*.dungeondraft_map` — regenerate

### Gull Rock reference configs (cloned at `/tmp/dd-procedural-forest/`)
- `config.json` — forest with canyon hills, lights, shadows
- `config-river.json` — forest river with water patterns, flow paths, bank edges
- `3rd Party Asset Examples/config-DM-swamp.json` — swamp with water edges
- `3rd Party Asset Examples/config-DM-island.json` — island variant
- `3rd Party Asset Examples/config-DM-autumn-river.json` — autumn river variant

### Mapping Gull Rock → our presets
| Gull Rock config | Our preset | Key features to add |
|---|---|---|
| config.json (forest) | Forest | ambient scatter lights, tree-top lights, cliff shadow offset paths |
| config-river.json | Forest River | all above + water pattern overlay, bank edge paths, flow paths |
| config-DM-swamp.json | Swamp/Marsh | water edge paths, scatter lights, water patterns |
| config-DM-island.json | Island presets | perimeter lights, water patterns |

## Status Updates

*To be added during implementation*