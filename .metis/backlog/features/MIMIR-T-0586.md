---
id: add-mapgen-skill-and-commands-to
level: task
title: "Add mapgen skill and commands to Claude Code plugin"
short_code: "MIMIR-T-0586"
created_at: 2026-03-11T23:20:21.849847+00:00
updated_at: 2026-03-13T02:38:35.947928+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Add mapgen skill and commands to Claude Code plugin

## Objective

Add a mapgen skill and slash commands to the Claude Code plugin so users can generate Dungeondraft maps via natural language. The 3 MCP tools (`generate_map`, `list_map_presets`, `validate_map_config`) are already registered in the server but the plugin has no skill or commands to guide Claude in using them effectively.

## Priority
- P1 — The mapgen tools exist but are undiscoverable without plugin guidance

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `mapgen` skill in `plugin/skills/mapgen/SKILL.md` that teaches Claude how to use the 3 mapgen MCP tools
- [ ] Skill includes guidance on YAML config schema (all MapConfig fields with types and descriptions)
- [ ] Skill includes biome preset reference (forest, grassland, cave — descriptions and default sizes)
- [ ] Skill includes example workflows: generate from preset, generate from custom config, validate config
- [ ] Slash command `/generate-map` that triggers a guided map generation flow (preset selection or config authoring)
- [ ] Tool parameter reference updated in `plugin/skills/mimir-dm/references/tool-parameter-reference.md` with the 3 new tools
- [ ] Plugin README updated to mention mapgen capabilities

## Implementation Notes

### Plugin Structure (existing pattern to follow)
```
plugin/
  skills/
    mapgen/
      SKILL.md              # Main skill — mapgen guidance
      references/
        yaml-config-ref.md  # Full MapConfig YAML schema reference
  commands/
    generate-map.md         # /generate-map slash command
```

### Key Context for the Skill

The skill should teach Claude:
- `generate_map` requires either `config_yaml` (YAML string) or `preset` (name), plus `output_path` (required) and optional `seed`
- `list_map_presets` takes no args, returns preset names/descriptions/sizes
- `validate_map_config` takes `config_yaml`, returns structured errors
- MapConfig YAML schema: name, width, height, seed, noise, island_mode, terrain, trees, clutter, clumps, roads, rivers, water, elevation, lighting
- Presets: forest (32x32, trees/clutter/road/elevation), grassland (32x32, clutter/smooth terrain), cave (24x24, dark lighting/rocky terrain)
- Output is a `.dungeondraft_map` file (JSON) that can be opened in Dungeondraft

### Reference Files
- `crates/mimir-mapgen/src/pipeline.rs` — MapConfig struct (lines 22-64)
- `crates/mimir-mapgen/src/biomes.rs` — preset definitions
- `crates/mimir-mcp/src/tools/mapgen.rs` — MCP tool implementations
- Existing skill pattern: `plugin/skills/mimir-dm/SKILL.md`

## Status Updates

### Session 1 - 2026-03-11
- Created `plugin/skills/mapgen/SKILL.md` — full skill with tool docs, preset reference, workflows (generate from preset, generate from custom YAML, iterate on a map), config authoring tips
- Created `plugin/skills/mapgen/references/yaml-config-ref.md` — complete MapConfig YAML schema reference covering all fields: noise, terrain (4 slots), trees (with shadow/canopy), clutter, clumps, roads (with edge paths), rivers, water bodies, elevation contours, lighting (with time-of-day presets), plus common Dungeondraft texture paths
- Created `plugin/commands/generate-map.md` — `/generate-map` slash command with 4-step guided flow (determine type → choose output → generate → report results)
- Updated `plugin/skills/mimir-dm/references/tool-parameter-reference.md` — added Map Generation Tools section (generate_map, list_map_presets, validate_map_config)
- Updated `plugin/README.md` — added `/generate-map` to commands list, added Map Generation tool category, added "Generating a Map" common workflow
- All 32 mimir-mcp tests pass

### Session 2 - 2026-03-11
- User feedback: skill reads like "random map picker" rather than "creative director". User wants the agent to translate scene descriptions into deliberate generation parameters, not just roll dice on presets.
- User asked about dungeon/room support — investigated `MapWall`, `MapPortal` in entities.rs and the generation pipeline. Finding: the format supports walls/portals but the pipeline produces none. Current system is **outdoor terrain only**.
- **Rewrote SKILL.md** with creative director framing:
  - New intro: "You are a creative director for outdoor battle maps"
  - Added "Scope: Outdoor Terrain Only" section — honest about no dungeon/interior support
  - Added "Your Role as Creative Director" section — interpret scene, choose parameters deliberately, explain choices, iterate with intent
  - Added "Scene-Driven Generation" workflow — listen to scene description, break it down, build config, validate, generate, present with rationale
  - Added "Translating Descriptions to Parameters" table — maps scene elements (dense forest, twilight, foggy, rocky hillside) to specific config parameters
  - Reframed presets as "starting points, not final answers"
  - Kept all MCP tool docs and technical reference intact