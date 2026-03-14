---
id: gap-analysis-mcp-server-claude
level: task
title: "Gap analysis: MCP server, Claude Code plugin, and mapgen CLI docs"
short_code: "MIMIR-T-0585"
created_at: 2026-03-11T23:13:29.924247+00:00
updated_at: 2026-03-13T12:52:08.434389+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Gap analysis: MCP server, Claude Code plugin, and mapgen CLI docs

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

Three major systems have no user-facing documentation: the MCP server (71 tools), the Claude Code plugin configuration, and the new mimir-mapgen CLI. Analyze each system's feature surface and produce a documentation plan specifying what pages to write, where they belong, and what screenshots or examples are needed.

## Scope

Analyze three undocumented systems and produce documentation plans for each.

### 1. MCP Server (mimir-mcp)
- 71 MCP tools across 9 categories (campaign, module, document, character, map, homebrew items/monsters/spells, catalog, mapgen)
- Sidecar binary architecture — how it's built, configured in tauri.conf.json, and launched
- Claude Code plugin configuration (`.mcp.json` or Tauri sidecar setup)
- Tool reference: what each tool does, required/optional parameters, example usage

### 2. Claude Code Plugin
- How to install and configure the Mimir MCP server in Claude Code
- Available tools and what they enable (campaign management via natural language)
- Example workflows: "create a campaign", "search for monsters", "add items to character"

### 3. Mapgen CLI (mimir-mapgen)
- `mimir-mapgen generate` — from YAML config or biome preset
- `mimir-mapgen validate` — config validation
- `mimir-mapgen list-presets` — available biome presets
- YAML config reference — all fields, types, defaults
- Example configs for common map types

### Code to Review
- `crates/mimir-mcp/src/tools/` — all tool modules for MCP reference
- `crates/mimir-mcp/src/main.rs` — server setup and configuration
- `crates/mimir-mapgen/src/main.rs` — CLI interface
- `crates/mimir-mapgen/src/pipeline.rs` — MapConfig schema (YAML config reference)
- `crates/mimir-mapgen/src/biomes.rs` — preset definitions
- Root `README.md` and any existing MCP/plugin docs

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Catalog all 71 MCP tools by category with brief descriptions
- [ ] Document the MCP server setup flow (sidecar build, config, launch)
- [ ] Map the mapgen CLI interface: all subcommands, flags, and options
- [ ] Document the YAML config schema (every field in MapConfig with type and description)
- [ ] Propose doc structure: where MCP/plugin/mapgen docs should live in SUMMARY.md
- [ ] For MCP tools: determine whether a full tool reference page or a grouped-by-category approach works better
- [ ] For mapgen: determine if this needs its own section or fits under developer docs
- [ ] Identify screenshots or terminal output examples needed
- [ ] Produce documentation plan as task output

## Expected Output

A documentation plan listing:
1. New pages needed (with proposed filenames and SUMMARY.md placement)
2. Whether MCP tools need individual pages or a single reference page with categories
3. Example YAML configs to include in mapgen docs
4. Terminal output examples (CLI help text, generation output)
5. Screenshot requests (if any UI is involved)

## Status Updates

### Gap Analysis Completed 2026-03-13

---

## Current State

The MCP server, Claude Code plugin, and mapgen CLI have **zero user-facing documentation** in the `docs/` site. However, extensive documentation exists inside the plugin source (`crates/mimir-mcp/plugin/`) — it's just not exposed to users browsing the docs.

### What EXISTS (in plugin source)
- `plugin/README.md` — Comprehensive overview with tool listing and workflows
- `plugin/skills/` — 7 specialized skills with examples and references
- `plugin/skills/mapgen/references/` — YAML config reference and example configs
- `plugin/skills/mimir-dm/references/` — Tool parameter reference, DM guidelines
- `plugin/skills/mimir-dm/examples/` — 4 workflow examples (character creation, module, NPC, level-up)

### What's MISSING (in docs/ site)
- MCP server setup and architecture
- Claude Code plugin installation guide
- Mapgen CLI usage guide
- Tool reference documentation
- Environment configuration (`MIMIR_DATABASE_PATH`)

## System Inventory

### MCP Tools (60+ across 8 categories)

| Category | Tools | Key Operations |
|----------|-------|---------------|
| Campaign (10) | list_campaigns, set_active_campaign, get_campaign_details, create_campaign, update_campaign, delete_campaign, export/import_campaign, get_campaign_sources, preview_archive | Full campaign lifecycle |
| Module (7) | create/list/get/update/delete_module, add/remove_monster_to_module | Module CRUD + monster assignment |
| Document (6) | list/read/create/edit/reorder/delete_document | Document management |
| Character (13) | list/get/create/edit/delete_character, add/remove_item, update_inventory, get_inventory, level_up, add/remove/list_character_spells | Full character lifecycle |
| Map (8) | create/list/get/update/delete_map, add_token, list_tokens, remove_token | Map and token management |
| Mapgen (3) | generate_map, list_map_presets, validate_map_config | Procedural map generation |
| Catalog (8) | search_monsters/items/spells/races/classes/backgrounds/feats/conditions | D&D 5e reference search |
| Homebrew (15) | CRUD × 3 types (items, monsters, spells) | Custom content management |

### Claude Code Plugin Components

- **Name:** `mimir-dm` (v0.6.0)
- **Commands (5):** mimir-campaigns, create-module, search-monsters, search-spells, generate-map
- **Skills (7):** mimir-dm, mapgen, encounter-balance, loot-audit, npc-network, session-prep, continuity-check, pressure-test
- **Design:** Human-in-the-loop — plugin guides but DM decides

### Mapgen CLI

```
mimir-mapgen generate --config <PATH> | --preset <NAME> --output <PATH> [--seed N]
mimir-mapgen validate <CONFIG>
mimir-mapgen list-presets
```

12 biome presets: forest, grassland, cave, desert, lake, ice_lake, arctic, island_tropical, island_forest, island_arctic, swamp, forest_river

## Documentation Plan

### New Pages Needed

#### 1. `docs/src/developer/reference/mcp-server/README.md` — MCP Server Overview
- What the MCP server is and why it exists
- Architecture: sidecar binary launched by Tauri externalBin
- Binary location: `crates/mimir/binaries/mimir-mcp-{target-triple}`
- Build process: `scripts/build-sidecar.{sh,ps1,mjs}`
- Database sharing: same SQLite file as Tauri app
- Environment: `MIMIR_DATABASE_PATH` configuration

#### 2. `docs/src/developer/reference/mcp-server/tool-reference.md` — MCP Tool Reference
- All 60+ tools grouped by category with parameters
- Decision: grouped-by-category single page (not individual pages per tool)
- For each tool: name, description, required/optional parameters, return type
- Link to plugin source for detailed examples

#### 3. `docs/src/how-to/ai-assistant/README.md` — Using Mimir with Claude Code
- Installing the Claude Code plugin
- Setting `MIMIR_DATABASE_PATH` for standalone use
- Available commands and what they enable
- Example workflows: "create a campaign", "search for monsters", "generate a map"
- Link to plugin skills for detailed guidance

#### 4. `docs/src/developer/reference/mapgen/README.md` — Mapgen Reference
- CLI usage: subcommands, flags, output
- YAML configuration schema: all fields with types and defaults
- Biome presets: names, descriptions, what they configure
- Example configurations (link to or embed from plugin references)
- Integration with MCP server (generate_map tool)

#### 5. `docs/src/how-to/maps/generate-map.md` — Generate Maps with Mapgen
- User-facing guide for generating Dungeondraft maps
- Using presets for quick generation
- Writing custom YAML configs
- Opening generated maps in Dungeondraft
- Iterating with seeds

### SUMMARY.md Placement

```markdown
- [How-To Guides](./how-to/README.md)
  - [Maps](./how-to/maps/README.md)
    - ...existing pages...
    - [Generate Maps](./how-to/maps/generate-map.md)          ← NEW
  - [AI Assistant](./how-to/ai-assistant/README.md)            ← NEW SECTION
    - [Setup Claude Code Plugin](./how-to/ai-assistant/setup.md)
- [Developer](./developer/README.md)
  - [Architecture](./developer/ARCHITECTURE.md)
  - ...existing pages...
  - [MCP Server](./developer/reference/mcp-server/README.md)   ← NEW
    - [Tool Reference](./developer/reference/mcp-server/tool-reference.md)
  - [Map Generator](./developer/reference/mapgen/README.md)     ← NEW
```

### Approach: Grouped Tool Reference vs Individual Pages

**Recommendation: Single grouped page.** 60+ tools are too many for individual pages, and most are simple CRUD operations. A single `tool-reference.md` with sections per category (Campaign, Module, Character, etc.) is more scannable and maintainable.

### Terminal Output Examples Needed

| Example | What to capture |
|---------|----------------|
| `mimir-mapgen --help` | Top-level CLI help text |
| `mimir-mapgen generate --help` | Generate subcommand help |
| `mimir-mapgen list-presets` | Preset listing output |
| `mimir-mapgen validate bad-config.yaml` | Validation error output |
| `mimir-mapgen generate --preset forest` | Successful generation stats |

### Existing Pages Needing MCP/Mapgen Mentions

| Page | What to add |
|------|-------------|
| `reference/glossary.md` | Add "MCP", "Sidecar", "Mapgen" terms |
| `reference/file-formats.md` | Add `.dungeondraft_map` format |
| `developer/ARCHITECTURE.md` | Already updated with MCP sidecar details ✅ |
| `developer/README.md` | Add links to new MCP and mapgen reference pages |