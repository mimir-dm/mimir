---
id: lighting-biome-presets-and-config
level: task
title: "Lighting, biome presets, and config parsing pipeline"
short_code: "MIMIR-T-0576"
created_at: 2026-03-11T21:23:35.838231+00:00
updated_at: 2026-03-11T23:01:03.341322+00:00
parent: MIMIR-I-0058
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0058
---

# Lighting, biome presets, and config parsing pipeline

## Parent Initiative

[[MIMIR-I-0058]]

## Objective

Implement the YAML configuration parsing pipeline, biome preset system, and Dungeondraft lighting configuration. This task ties together the generation modules (terrain, objects, paths, water) into a cohesive config-driven pipeline where a single YAML file produces a complete `.dungeondraft_map`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] YAML config schema defined with `serde` deserialization structs covering all generation parameters
- [ ] Config validation: detect missing required fields, invalid asset paths, out-of-range values with clear error messages
- [ ] Biome preset system: at least 3 built-in presets (forest, grassland, cave) that provide sensible defaults for terrain textures, object palettes, and noise parameters
- [ ] Presets can be overridden partially — user specifies only what they want to change
- [ ] Lighting section generates correct DD `environment` block (ambient color, ambient energy, shadow color)
- [ ] Time-of-day presets (dawn, day, dusk, night, underground) map to appropriate lighting values
- [ ] End-to-end pipeline: `Config -> generate terrain -> place objects -> route paths -> generate water -> write .dungeondraft_map`
- [ ] Pipeline stages are independent and optional — config can omit sections to skip stages
- [ ] Config supports seeded RNG (`seed` field) for reproducible generation
- [ ] Unit tests for config parsing, validation, preset merging, and lighting generation

## Implementation Notes

### Technical Approach

**Config schema** (`config.rs`):
```
MapConfig {
  meta: MetaConfig { name, width, height, grid_size, seed? },
  biome: Option<String>,        // preset name, overridden by explicit sections
  terrain: Option<TerrainConfig>,
  objects: Option<ObjectsConfig>,
  paths: Option<PathsConfig>,
  water: Option<WaterConfig>,
  lighting: Option<LightingConfig>,
  contours: Option<ContourConfig>,
}
```

**Biome presets**: Embedded YAML files or const structs keyed by name. A preset fills in defaults for all optional sections. User-provided sections merge on top (user wins).

**Lighting model**: DD stores lighting as `environment` in the world block with `ambient_light` (Color), `ambient_energy` (float), and `shadow_color` (Color). Map time-of-day strings to these tuples. Allow raw override for custom lighting.

**Pipeline orchestrator** (`pipeline.rs`):
1. Parse YAML config
2. Resolve biome preset + user overrides
3. Initialize seeded RNG
4. Run enabled stages in order (terrain → objects → paths → water → contours → lighting)
5. Assemble `DungeondraftMap` struct
6. Serialize to `.dungeondraft_map` via writer (MIMIR-T-0570)

### Dependencies
- MIMIR-T-0571 (core algorithms — noise, Poisson Disc used by pipeline stages)
- MIMIR-T-0572 (terrain generation — first pipeline stage)
- MIMIR-T-0573 (object placement — consumes terrain output)
- MIMIR-T-0574 (road/river generation)
- MIMIR-T-0575 (water body generation)
- MIMIR-T-0570 (map format writer — final serialization)

### Risk Considerations
- Config schema needs to be stable before other tasks can finalize their parameter interfaces; define the schema early and iterate
- Biome presets are opinionated — start minimal and expand based on real Dungeondraft testing

## Status Updates

### Session 2 - 2026-03-11
- Fixed `test_generate_deterministic` failure: root cause was `HashMap<String, String>` for `layers`, `levels`, `materials`, and `embedded` fields in world.rs — HashMap has non-deterministic iteration order causing JSON serialization mismatches
- Fix: replaced all `HashMap` with `BTreeMap` in `format/world.rs` for deterministic key ordering
- All 78 unit tests + 2 integration tests now pass
- Task complete: biomes.rs (3 presets), pipeline.rs (MapConfig, LightingConfig, validate, generate orchestrator), all serde derives across all config structs

*To be added during implementation*