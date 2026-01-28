---
id: migrate-mcp-server-to-v0-5
level: initiative
title: "Migrate MCP Server to v0.5"
short_code: "MIMIR-I-0050"
created_at: 2026-01-28T03:56:17.629668+00:00
updated_at: 2026-01-28T05:19:45.887066+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: migrate-mcp-server-to-v0-5
---

# Migrate MCP Server to v0.5 Initiative

## Context

The v0.4 codebase had a standalone MCP server (`mimir-dm-mcp`) that exposed 21 tools for D&D campaign management via the Model Context Protocol. This allowed Claude Code and Claude Desktop to interact with Mimir campaigns.

The v0.5 rewrite has all the backend functionality (Tauri commands, services, DAL) but no MCP server. We need to migrate the MCP functionality to v0.5, wrapping the existing backend in the MCP protocol.

## Goals & Non-Goals

**Goals:**
- Create `mimir-mcp` crate with MCP server using `rust-mcp-sdk`
- Expose 28 tools covering campaigns, modules, documents, characters, and catalog search
- Create Claude Code plugin with commands, skills, and documentation
- Support both Claude Code (CLI) and Claude Desktop (standalone app)

**Non-Goals:**
- New business logic (all logic exists in mimir-core services)
- UI changes to the Tauri app
- Real-time sync between MCP and Tauri app

## MCP Tools (28 Total)

### Campaign Tools (4)

| Tool | Description |
|------|-------------|
| `list_campaigns` | List all campaigns (with optional include_archived) |
| `set_active_campaign` | Set the active campaign for subsequent operations |
| `get_campaign_details` | Get details about the active campaign |
| `get_campaign_sources` | Get source books enabled for active campaign |

### Module Tools (6)

| Tool | Description |
|------|-------------|
| `create_module` | Create a new module in the active campaign |
| `list_modules` | List modules in the active campaign |
| `get_module_details` | Get full details about a module (monsters, items, NPCs) |
| `add_monster_to_module` | Add a monster from catalog to a module |
| `update_module_monster` | Update monster quantity, notes, encounter tag |
| `add_item_to_module` | Add an item from catalog to a module |

### Document Tools (4)

| Tool | Description |
|------|-------------|
| `list_documents` | List documents in the active campaign |
| `read_document` | Read document content |
| `edit_document` | Edit document using search and replace |
| `create_document` | Create a new document |

### Character Tools (7)

| Tool | Description |
|------|-------------|
| `list_characters` | List characters (filter by PC/NPC) |
| `get_character` | Get full character details |
| `create_character` | Create a new character (PC or NPC) |
| `edit_character` | Update character attributes |
| `assign_npc_to_module` | Assign an NPC to a module |
| `add_item_to_character` | Add item to character inventory |
| `update_character_currency` | Update character currency (cp/sp/ep/gp/pp) |

### Catalog Search Tools (7)

| Tool | Description |
|------|-------------|
| `search_monsters` | Search monsters by name, CR, type, source |
| `search_items` | Search items by name, type, rarity, source |
| `search_traps` | Search traps/hazards by name, category, source |
| `search_spells` | Search spells by name, level, school, class, concentration, ritual |
| `search_classes` | Search classes by name, source |
| `search_races` | Search races by name, size, source |
| `search_feats` | Search feats by name, prerequisite, source |

## Architecture

```
mimir-mcp/
├── Cargo.toml
├── src/
│   ├── main.rs              # MCP server entry point
│   ├── lib.rs
│   ├── context.rs           # McpContext (DB connection, active campaign)
│   ├── handler.rs           # ServerHandler implementation
│   ├── error.rs             # Error types
│   └── tools/
│       ├── mod.rs
│       ├── campaign.rs      # Campaign tool implementations
│       ├── module.rs        # Module tool implementations
│       ├── document.rs      # Document tool implementations
│       ├── character.rs     # Character tool implementations
│       └── catalog.rs       # Catalog search tool implementations
└── plugin/                  # Claude Code plugin
    ├── .claude-plugin/
    │   └── plugin.json
    ├── .mcp.json
    ├── README.md
    ├── commands/
    │   ├── mimir-campaigns.md
    │   ├── create-module.md
    │   └── search-monsters.md
    └── skills/
        └── mimir-dm/
            └── SKILL.md
```

## Dependencies

- `rust-mcp-sdk` - MCP protocol implementation
- `mimir-core` - Database, models, services, DAL
- `tokio` - Async runtime
- `serde` / `serde_json` - Serialization

## Implementation Plan

1. **Crate Setup** - Create mimir-mcp crate with dependencies
2. **Context & Handler** - MCP context and ServerHandler implementation
3. **Campaign Tools** - list, set_active, get_details, get_sources
4. **Module Tools** - create, list, get_details, add_monster, update_monster, add_item
5. **Document Tools** - list, read, edit, create
6. **Character Tools** - list, get, create, edit, assign_npc, add_item, update_currency
7. **Catalog Tools** - search_monsters, items, traps, spells, classes, races, feats
8. **Plugin Structure** - Claude Code plugin with commands and skills
9. **Testing & Documentation** - Integration tests and README