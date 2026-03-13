# MCP Server Reference

Technical reference for Mimir's Model Context Protocol (MCP) server — the sidecar that enables AI assistant integration.

## Architecture

The MCP server (`mimir-mcp`) runs as a Tauri sidecar process. It connects to the same SQLite database as the main app and exposes 71 tools across 10 categories.

### Components

- **Binary**: `mimir-mcp` (built to `crates/mimir/binaries/mimir-mcp-{target-triple}`)
- **Protocol**: MCP over stdio
- **Database**: Shared SQLite database at `~/Library/Application Support/com.mimir.app/data/mimir.db`
- **Plugin**: Claude Code plugin at `crates/mimir-mcp/plugin/`

### Configuration

The database path is auto-detected. Override with:

```bash
export MIMIR_DATABASE_PATH=/path/to/mimir.db
```

## Tool Reference

### Campaign Management (10 tools)

| Tool | Description |
|------|-------------|
| `list_campaigns` | List all available campaigns |
| `set_active_campaign` | Set the active campaign (required by most tools) |
| `get_campaign_details` | Get campaign info including modules and characters |
| `get_campaign_sources` | Get enabled source books for a campaign |
| `create_campaign` | Create a new campaign |
| `update_campaign` | Update campaign name or description |
| `delete_campaign` | Delete a campaign and all its data |
| `export_campaign` | Export campaign as shareable archive |
| `import_campaign` | Import campaign from archive |
| `preview_archive` | Preview archive contents without importing |

### Module Management (8 tools)

| Tool | Description |
|------|-------------|
| `create_module` | Create new module in active campaign |
| `list_modules` | List all modules in active campaign |
| `get_module_details` | Get module info including documents, monsters, items |
| `update_module` | Update module name or description |
| `delete_module` | Delete module and all contents |
| `add_monster_to_module` | Add monster from catalog or homebrew to module |
| `remove_monster_from_module` | Remove monster from module |
| `add_item_to_module` | Add catalog item as loot to module |

### Document Management (6 tools)

| Tool | Description |
|------|-------------|
| `list_documents` | List campaign-level or module-level documents |
| `read_document` | Read full content of a document |
| `create_document` | Create document (backstory, read_aloud, dm_notes, description, custom) |
| `edit_document` | Edit document using search and replace |
| `delete_document` | Delete a document |
| `reorder_document` | Reorder documents by swapping sort positions |

### Character Management (13 tools)

| Tool | Description |
|------|-------------|
| `list_characters` | List characters with optional filters (type, module, location, faction) |
| `get_character` | Get detailed character info including classes and inventory |
| `create_character` | Create NPC or PC (auto-populates proficiencies from catalog) |
| `edit_character` | Update ability scores, currency, race, background, traits |
| `delete_character` | Delete character and all associated data |
| `level_up_character` | Level up character (handles HP, multiclass, ASI/feats, spells) |
| `add_item_to_character` | Add catalog item to character inventory |
| `remove_item_from_character` | Remove item from character inventory |
| `update_character_inventory` | Update item quantity, equipped, or attuned state |
| `get_character_inventory` | Get inventory (filterable by equipped/attuned) |
| `add_character_spell` | Add spell to character's known spells |
| `remove_character_spell` | Remove spell from character |
| `list_character_spells` | List character's known spells (filterable by class/prepared) |

### Map Management (8 tools)

| Tool | Description |
|------|-------------|
| `create_map` | Upload UVTT file to create new map |
| `list_maps` | List maps (optionally filtered by module) |
| `get_map` | Get map details including token placements |
| `update_map` | Update map metadata (name, description, lighting) |
| `delete_map` | Delete map and associated UVTT asset |
| `add_token_to_map` | Add monster or NPC token to map |
| `list_tokens_on_map` | List all tokens (optionally visible only) |
| `remove_token` | Remove token placement from map |

### Map Generation (3 tools)

| Tool | Description |
|------|-------------|
| `generate_map` | Generate Dungeondraft map from YAML config or biome preset |
| `list_map_presets` | List available biome presets |
| `validate_map_config` | Validate YAML config without generating |

### Catalog Search (8 tools)

| Tool | Description |
|------|-------------|
| `search_monsters` | Search monster catalog (includes homebrew from active campaign) |
| `search_items` | Search item catalog |
| `search_spells` | Search spell catalog |
| `search_races` | Search race catalog |
| `search_classes` | Search class catalog |
| `search_backgrounds` | Search background catalog |
| `search_feats` | Search feat catalog |
| `search_conditions` | Search condition catalog |

### Homebrew Items (5 tools)

| Tool | Description |
|------|-------------|
| `list_homebrew_items` | List all homebrew items in active campaign |
| `get_homebrew_item` | Get homebrew item by ID |
| `create_homebrew_item` | Create new or clone from catalog |
| `update_homebrew_item` | Update homebrew item |
| `delete_homebrew_item` | Delete homebrew item |

### Homebrew Monsters (5 tools)

| Tool | Description |
|------|-------------|
| `list_homebrew_monsters` | List all homebrew monsters in active campaign |
| `get_homebrew_monster` | Get homebrew monster by ID |
| `create_homebrew_monster` | Create new or clone from catalog |
| `update_homebrew_monster` | Update homebrew monster |
| `delete_homebrew_monster` | Delete homebrew monster |

### Homebrew Spells (5 tools)

| Tool | Description |
|------|-------------|
| `list_homebrew_spells` | List all homebrew spells in active campaign |
| `get_homebrew_spell` | Get homebrew spell by ID |
| `create_homebrew_spell` | Create new or clone from catalog |
| `update_homebrew_spell` | Update homebrew spell |
| `delete_homebrew_spell` | Delete homebrew spell |

## Claude Code Plugin

### Installation

```bash
claude plugin add /path/to/mimir/crates/mimir-mcp/plugin
```

### Slash Commands

| Command | Description |
|---------|-------------|
| `/mimir-campaigns` | List all available campaigns |
| `/create-module` | Create a new module in the active campaign |
| `/search-monsters` | Search the D&D monster catalog |
| `/search-spells` | Search the D&D spell catalog |
| `/generate-map` | Generate a procedural Dungeondraft map |

### Skills

| Skill | Description |
|-------|-------------|
| `/mimir-dm` | General campaign management |
| `/encounter-balance` | Analyze encounter difficulty |
| `/loot-audit` | Audit treasure distribution |
| `/session-prep` | Pre-session readiness review |
| `/continuity-check` | Find contradictions across documents |
| `/npc-network` | Map NPC relationships |
| `/pressure-test` | Stress-test scenarios |
| `/mapgen` | Creative direction for map generation |

## See Also

- [AI Assistant How-To](../how-to/ai-assistant/)
- [Mapgen Reference](./mapgen.md)
