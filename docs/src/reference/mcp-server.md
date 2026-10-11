# MCP Server Reference

Technical reference for Mimir's Model Context Protocol (MCP) endpoint, which lets an AI assistant work with your campaigns.

## Architecture

The Mimir server serves MCP at `/mcp` (MCP over streamable HTTP). The tools
use the same database and services as the web app. The tool list is in
`crates/mimir-mcp` (its registry); the server serves it.

### Components

- **Endpoint**: `<server>/mcp`, for example `http://127.0.0.1:8740/mcp`
- **Protocol**: MCP, streamable HTTP (sessions: the `Mcp-Session-Id` header)
- **Access**: the DM token (`Authorization: Bearer <MIMIR_API_TOKEN>`). A
  player link is refused (403); no token or a wrong token is refused (401).
  A server in open mode (no `MIMIR_API_TOKEN`) lets every caller in.
- **Plugin**: Claude Code plugin at `crates/mimir-mcp/plugin/` (it reads
  `MIMIR_URL` and `MIMIR_API_TOKEN`)

### Errors

Most tools operate on an active campaign. Until a campaign is selected with `set_active_campaign`, campaign-scoped tools return a `NoActiveCampaign` error (`No active campaign. Use set_active_campaign first.`). Each MCP session has its own active campaign; it is not stored, so a new session starts with no campaign selected.

A tool's failure comes back as a tool result with `isError: true` and the text `Error: …`; an unknown tool is a JSON-RPC error.

## Tool Reference

### Campaign Management (16 tools)

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
| `preview_adventure_campaign` | Show how an adventure in the reader becomes a campaign |
| `create_campaign_from_adventure` | Make a campaign from an adventure; it becomes the active campaign |
| `export_adventure_file` | Export the active campaign as a 5etools homebrew adventure file |
| `export_adventure_pack` | Write the active campaign as a Mimir adventure pack in a folder |
| `import_adventure_pack` | Import a Mimir adventure pack as a new campaign |
| `print_adventure_book` | Write the active campaign as one PDF book in a folder |
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
| `edit_document` | Rename a document, edit its content with search and replace, or both |
| `delete_document` | Delete a document |
| `reorder_document` | Reorder documents by swapping sort positions |

### Character Management (16 tools)

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
| `add_character_feat` | Record a catalog feat for a character outside level-up |
| `list_character_feats` | List character's feats with source and how each was gained |
| `remove_character_feat` | Remove a feat from a character by name |

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

### Catalog Search (1 tool)

A single `search_catalog` tool searches all catalog categories.

| Tool | Description |
|------|-------------|
| `search_catalog` | Search the D&D 5e catalog by category |

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `category` | string (required) | One of: `monster`, `item`, `spell`, `race`, `class`, `background`, `feat`, `condition` |
| `name` | string | Search by name (partial match) |
| `limit` | integer | Maximum results (default: 20) |
| `cr_min` / `cr_max` | number | Challenge rating range (monsters only) |
| `monster_type` | string | Creature type, e.g. `undead`, `dragon` (monsters only) |
| `include_homebrew` | boolean | Include homebrew monsters from the active campaign (monsters only, default: true) |
| `rarity` | string | `common`, `uncommon`, `rare`, `very rare`, `legendary`, `artifact` (items only) |
| `item_type` | string | e.g. `weapon`, `armor`, `wondrous item` (items only) |
| `level` | integer | Spell level, 0 for cantrips (spells only) |
| `school` | string | School of magic, e.g. `evocation` (spells only) |
| `class_name` | string | Filter by class spell list (spells only) |

Results are filtered to the active campaign's enabled sources when a campaign is set.

### Homebrew Content (5 tools)

Homebrew items, monsters, and spells share a unified tool set. Every call takes a
`content_type` parameter: `item`, `monster`, or `spell`.

| Tool | Description |
|------|-------------|
| `list_homebrew` | List homebrew content of one type in the active campaign |
| `get_homebrew` | Get a homebrew entry by ID |
| `create_homebrew` | Create new homebrew, or clone from the catalog with `cloned_from_name` + `cloned_from_source` |
| `update_homebrew` | Update a homebrew entry |
| `delete_homebrew` | Delete a homebrew entry by ID |

**`create_homebrew` parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `content_type` | string (required) | `item`, `monster`, or `spell` |
| `name` | string (required) | Name of the homebrew content |
| `data` | string | JSON string with content data. Required when not cloning; when cloning, fields here override the catalog data |
| `item_type` | string | Items only: `weapon`, `armor`, `potion`, `ring`, `rod`, `scroll`, `staff`, `wand`, `wondrous item`, `adventuring gear` |
| `rarity` | string | Items only: `common`, `uncommon`, `rare`, `very rare`, `legendary`, `artifact` |
| `cr` | string | Monsters only, e.g. `1/4`, `1`, `5`, `20` |
| `creature_type` | string | Monsters only, e.g. `humanoid`, `dragon`, `undead` |
| `size` | string | Monsters only: `T`, `S`, `M`, `L`, `H`, `G` |
| `level` | integer | Spells only: 0 for cantrip, 1–9 |
| `school` | string | Spells only, e.g. `evocation`, `necromancy` |
| `cloned_from_name` | string | Catalog entry to clone from; must be used with `cloned_from_source` |
| `cloned_from_source` | string | Source book of the cloned entry (e.g. `PHB`, `DMG`, `MM`) |

## Claude Code Plugin

For installation and setup, see the [AI Assistant How-To](../how-to/ai-assistant/README.md).

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
