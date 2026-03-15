---
id: pass-1-7-write-generate-map-md-how
level: task
title: "Pass 1.7: Write generate-map.md how-to page"
short_code: "MIMIR-T-0603"
created_at: 2026-03-13T13:50:16.651940+00:00
updated_at: 2026-03-13T14:11:04.678740+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.7: Write generate-map.md how-to page

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Write new page `how-to/maps/generate-map.md` — a user-facing guide for generating Dungeondraft maps using mimir-mapgen.

## Scope

Write `docs/src/how-to/maps/generate-map.md` — a user-facing how-to for generating Dungeondraft maps.

### Page Content

1. **What mapgen does** — procedurally generates terrain, vegetation, roads, rivers, rooms, and lighting into a `.dungeondraft_map` file
2. **Using presets** — quick generation with `mimir-mapgen generate --preset forest --output map.dungeondraft_map`
3. **Writing custom YAML configs** — basic example with terrain + trees + road
4. **Opening in Dungeondraft** — output is native DD format, open directly
5. **Iterating with seeds** — same seed = same map, change config and re-generate
6. **Available presets** — table of 12 biome presets with descriptions
7. **Using via MCP/Claude Code** — mention that mapgen is also available as an MCP tool (`generate_map`)

### Style Guide
- This is a USER how-to, not a developer reference. Focus on practical steps.
- Link to developer mapgen reference (T-0614) for YAML schema details
- Keep examples short — one basic config, one with rooms
- Match tone of existing how-to pages (step-numbered, action-oriented)

### Verification Sources
- `crates/mimir-mapgen/src/main.rs` — CLI interface and arguments
- `crates/mimir-mapgen/src/biomes.rs` — preset names and descriptions
- `crates/mimir-mapgen/examples/` — example YAML configs
- `crates/mimir-mcp/plugin/skills/mapgen/` — existing skill references

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Page created at `docs/src/how-to/maps/generate-map.md`
- [ ] CLI usage matches actual `mimir-mapgen` binary interface
- [ ] All 12 preset names and descriptions verified against `biomes.rs`
- [ ] Example YAML config is valid (run through `validate_map_config` or verified against schema)
- [ ] Links to developer reference for full YAML schema
- [ ] Mentions MCP tool availability

## Status Updates

### 2026-03-13: Completed
Created `docs/src/how-to/maps/generate-map.md`.

**Content:**
- Quick start with preset usage
- Full preset table (12 presets with aliases, verified against `biomes.rs`)
- Custom YAML config example (basic terrain + trees + road)
- Validate subcommand
- Iterating with seeds
- MCP tool mention (`generate_map`)
- Links to mapgen reference page

**CLI verified against `main.rs`:** generate (config/--preset/--output/--seed), validate, list-presets.
**Presets verified against `biomes.rs`:** all 12 names, descriptions, and default sizes confirmed.