# Mimir MCP Tool Reference

Complete reference for all available MCP tools (20 tools total).

## Campaign Management

| Tool | Purpose |
|------|---------|
| `list_campaigns` | List all campaigns (use `include_archived: true` to see archived) |
| `set_active_campaign` | Set active campaign by ID (required before other operations) |

## Module Management

| Tool | Purpose |
|------|---------|
| `create_module` | Create a new module with auto-generated documents |
| `list_modules` | List modules (optional `status` filter: planning, active, completed, all) |
| `get_module_details` | Get module info with documents and NPCs |
| `add_monster_to_module` | Add a monster from catalog to module |
| `add_item_to_module` | Add an item from catalog to module |

## Document Authoring

| Tool | Purpose |
|------|---------|
| `list_documents` | List documents (filter by `level`, `module_id`, or `session_id`) |
| `read_document` | Read document content by ID (returns markdown with frontmatter) |
| `edit_document` | Update document content (search/replace within document) |
| `create_user_document` | Create a new custom markdown document |

## Character & NPC Management

| Tool | Purpose |
|------|---------|
| `list_characters` | List characters (filter by `character_type`: pc/npc/all) |
| `get_character` | Get full character details (optionally include version history) |
| `create_npc` | Create an NPC with name, race, class, role, faction, alignment |
| `assign_npc_to_module` | Link NPC to module with role and optional encounter_tag |
| `add_item_to_character` | Add item to character inventory (creates new version) |
| `update_character_currency` | Update character's currency (creates new version) |

## Catalog Search

| Tool | Purpose |
|------|---------|
| `search_monsters` | Find monsters by name, type, CR range, source |
| `search_items` | Find items by name, type, rarity, source |
| `search_traps` | Find traps/hazards by name, category (Trap/Hazard), source |

---

## Detailed Parameter Reference

### Campaign Tools

#### list_campaigns
- `include_archived` (boolean, optional) - Include archived campaigns (default: false)

#### set_active_campaign
- `campaign_id` (integer, required) - ID of the campaign to activate

### Module Tools

#### create_module
- `name` (string, required) - Module name
- `module_type` (string, optional) - Type: "mystery", "dungeon", "heist", "horror", "political"

#### list_modules
- `status` (string, optional) - Filter: "planning", "active", "completed", "all" (default: all)

#### get_module_details
- `module_id` (integer, required) - Module ID

#### add_monster_to_module
- `module_id` (integer, required) - Target module ID
- `monster_name` (string, required) - Exact name from catalog
- `monster_source` (string, required) - Source book (e.g., "MM", "DMG")
- `quantity` (integer, optional) - Number to add (default: 1)
- `encounter_tag` (string, optional) - Tag to group monsters by encounter

#### add_item_to_module
- `module_id` (integer, required) - Target module ID
- `item_name` (string, required) - Exact name from catalog
- `item_source` (string, required) - Source book (e.g., "PHB", "DMG")
- `quantity` (integer, optional) - Number to add (default: 1)
- `location` (string, optional) - Where the item is found
- `notes` (string, optional) - Additional context

### Document Tools

#### list_documents
- `level` (string, optional) - Filter: "campaign", "module", "session", "handout"
- `module_id` (integer, optional) - Filter by module
- `session_id` (integer, optional) - Filter by session

#### read_document
- `document_id` (integer, required) - Document ID

#### edit_document
- `document_id` (integer, required) - Document ID
- `search` (string, required) - Text to find
- `replace` (string, required) - Replacement text
- `replace_all` (boolean, optional) - Replace all occurrences (default: false)

#### create_user_document
- `title` (string, required) - Document title
- `content` (string, optional) - Initial markdown content
- `module_id` (integer, optional) - Associate with a module

### Character Tools

#### list_characters
- `character_type` (string, optional) - Filter: "pc", "npc", "all" (default: all)

#### get_character
- `character_id` (integer, required) - Character ID
- `include_versions` (boolean, optional) - Include version history (default: false)

#### create_npc
- `name` (string, required) - NPC name
- `race` (string, required) - Race (e.g., "Human", "Elf", "Dwarf")
- `class` (string, optional) - Class if applicable
- `role` (string, optional) - Role: quest_giver, ally, antagonist, neutral, merchant, informant
- `location` (string, optional) - Where the NPC can be found
- `faction` (string, optional) - Faction affiliation
- `alignment` (string, optional) - Alignment (e.g., "Lawful Good", "Chaotic Neutral")
- `notes` (string, optional) - Additional details

#### assign_npc_to_module
- `character_id` (integer, required) - NPC's character ID
- `module_id` (integer, required) - Target module ID
- `role` (string, optional) - Role in this module
- `encounter_tag` (string, optional) - Tag for specific encounter
- `notes` (string, optional) - Context for this assignment

#### add_item_to_character
- `character_id` (integer, required) - Character ID
- `item_name` (string, required) - Exact name from catalog
- `item_source` (string, required) - Source book
- `quantity` (integer, optional) - Number to add (default: 1)
- `notes` (string, optional) - Additional context

#### update_character_currency
- `character_id` (integer, required) - Character ID
- `copper` (integer, optional) - Copper pieces to add/subtract
- `silver` (integer, optional) - Silver pieces to add/subtract
- `electrum` (integer, optional) - Electrum pieces to add/subtract
- `gold` (integer, optional) - Gold pieces to add/subtract
- `platinum` (integer, optional) - Platinum pieces to add/subtract

*Use positive values to add, negative to subtract. Creates a new character version.*

### Catalog Tools

#### search_monsters
- `name` (string, optional) - Monster name (partial match)
- `creature_type` (string, optional) - Type: "undead", "dragon", "humanoid", etc.
- `min_cr` (number, optional) - Minimum challenge rating
- `max_cr` (number, optional) - Maximum challenge rating
- `source` (string, optional) - Source book abbreviation
- `limit` (integer, optional) - Max results (default: 20)

#### search_items
- `name` (string, optional) - Item name (partial match)
- `item_type` (string, optional) - Type: "weapon", "armor", "potion", etc.
- `rarity` (string, optional) - Rarity: common, uncommon, rare, very rare, legendary
- `source` (string, optional) - Source book abbreviation
- `limit` (integer, optional) - Max results (default: 20)

#### search_traps
- `name` (string, optional) - Trap/hazard name (partial match)
- `category` (string, optional) - "Trap" or "Hazard"
- `source` (string, optional) - Source book abbreviation
- `limit` (integer, optional) - Max results (default: 20)
