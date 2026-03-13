# AI Assistant Integration

Use Mimir with Claude Code for AI-assisted campaign management via natural language.

## What It Does

Mimir includes an MCP (Model Context Protocol) server that exposes 70+ tools to Claude Code, enabling you to manage campaigns, search catalogs, generate maps, and prep sessions through conversation. Ask "create a forest map" or "find CR 5 undead monsters" and the assistant handles it.

## Prerequisites

- Mimir desktop app installed
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed
- The `mimir-mcp` binary (built with the app)

## Setup

### For App Users

The MCP server runs as a Tauri sidecar — it starts automatically when needed. Install the Claude Code plugin:

```bash
claude plugin add /path/to/mimir/crates/mimir-mcp/plugin
```

### Database Path

The server auto-detects the Mimir database at `~/Library/Application Support/com.mimir.app/data/mimir.db`. Override with:

```bash
export MIMIR_DATABASE_PATH=/path/to/mimir.db
```

## Slash Commands

| Command | Description |
|---------|-------------|
| `/mimir-campaigns` | List all available campaigns |
| `/create-module` | Create a new module in the active campaign |
| `/search-monsters` | Search the D&D monster catalog by name, CR, or type |
| `/search-spells` | Search the D&D spell catalog |
| `/generate-map` | Generate a procedural Dungeondraft map |

## Skills

Specialized analysis tools that run multi-step workflows:

| Skill | Description |
|-------|-------------|
| `/mimir-dm` | General campaign management — create campaigns, modules, NPCs, encounters |
| `/encounter-balance` | Analyze encounter difficulty against party level and size |
| `/loot-audit` | Audit treasure distribution across modules |
| `/session-prep` | Pre-session checklist and readiness review |
| `/continuity-check` | Find contradictions and plot holes across campaign documents |
| `/npc-network` | Map NPC relationships and faction dynamics |
| `/pressure-test` | Stress-test scenarios — "what if the players do X?" |
| `/mapgen` | Creative direction for procedural map generation |

## Example Workflows

**Campaign setup:**
> "Create a new campaign called 'Curse of Strahd' and add a module for Death House"

**Monster search:**
> "Find all CR 3-5 undead monsters from the Monster Manual"

**Map generation:**
> "Generate a dark swamp map with standing water and dim lighting"

**Session prep:**
> "Run a session prep check on the Goblin Hideout module"

**Character management:**
> "Add a +1 longsword to Aldric's inventory"

## Tool Categories

The MCP server provides tools across these areas:

- **Campaign Management** — Create, update, export/import campaigns
- **Module Management** — Create modules, manage module content
- **Document Management** — Create and edit campaign/module documents
- **Character Management** — Create characters, manage inventory and spells
- **Map Management** — Upload maps, place tokens, manage light sources
- **Map Generation** — Generate Dungeondraft maps from presets or YAML configs
- **Homebrew Management** — Create and edit homebrew items, monsters, and spells
- **Catalog Search** — Search monsters, spells, items, classes, races, backgrounds, feats, conditions

## Important Notes

- MCP tools modify the database directly — there is no undo
- [Export your campaign](../campaigns/export-campaign.md) regularly as a backup
- The assistant is the DM's tool — it prompts for choices rather than making creative decisions autonomously
- Set the active campaign with `set_active_campaign` before using campaign-specific tools
