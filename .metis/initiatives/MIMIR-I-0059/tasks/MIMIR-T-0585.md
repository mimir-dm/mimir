---
id: gap-analysis-mcp-server-claude
level: task
title: "Gap analysis: MCP server, Claude Code plugin, and mapgen CLI docs"
short_code: "MIMIR-T-0585"
created_at: 2026-03-11T23:13:29.924247+00:00
updated_at: 2026-03-11T23:13:29.924247+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

*To be added during implementation*