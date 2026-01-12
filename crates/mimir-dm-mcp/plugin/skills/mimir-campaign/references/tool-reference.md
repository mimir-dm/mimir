# Mimir MCP Tool Reference

Complete reference for all available MCP tools.

## Campaign Management

| Tool | Purpose |
|------|---------|
| `list_campaigns` | List all campaigns (use `include_archived: true` to see archived) |
| `set_active_campaign` | Set active campaign by ID (required before other operations) |

## Module Management

| Tool | Purpose |
|------|---------|
| `create_module` | Create a new module with auto-generated documents |
| `list_modules` | List modules (optional `status` filter: planning, active, completed) |
| `get_module_details` | Get module info with documents and NPCs |
| `add_monster_to_module` | Add a monster from catalog to module |
| `add_item_to_module` | Add an item from catalog to module |

## Document Authoring

| Tool | Purpose |
|------|---------|
| `list_documents` | List documents (filter by `level`: campaign/module, `module_id`) |
| `read_document` | Read document content by ID |
| `edit_document` | Update document content (search/replace within document) |

## Character & NPC Management

| Tool | Purpose |
|------|---------|
| `list_characters` | List characters (filter by `character_type`: pc/npc) |
| `get_character` | Get full character details |
| `create_npc` | Create an NPC with name, race, role, notes |
| `assign_npc_to_module` | Link NPC to module with role and optional encounter_tag |
| `add_item_to_character` | Add item to character inventory |
| `update_character_currency` | Update character's gold/silver/copper |

## Catalog Search

| Tool | Purpose |
|------|---------|
| `search_monsters` | Find monsters by name, type, CR range, source |
| `search_items` | Find items by name, type, rarity, source |
| `search_traps` | Find traps/hazards by name, category (Trap/Hazard), source |

## Tool Parameters

### search_monsters
- `name` - Monster name (partial match)
- `creature_type` - e.g., "undead", "dragon", "humanoid"
- `min_cr`, `max_cr` - Challenge rating range
- `source` - Source book abbreviation

### search_items
- `name` - Item name (partial match)
- `item_type` - e.g., "weapon", "armor", "potion"
- `rarity` - common, uncommon, rare, very rare, legendary
- `source` - Source book abbreviation

### search_traps
- `name` - Trap/hazard name (partial match)
- `category` - "Trap" or "Hazard"
- `source` - Source book abbreviation

### create_npc
- `name` - NPC name (required)
- `race` - e.g., "Human", "Elf", "Dwarf"
- `role` - See NPC roles reference
- `location` - Where the NPC can be found
- `notes` - Additional details about the NPC

### add_monster_to_module / add_item_to_module
- `module_id` - Target module ID (required)
- `monster_name` / `item_name` - Exact name from catalog (required)
- `source` - Source book (required)
- `quantity` - Number to add (default: 1)
- `notes` - Context for this addition
