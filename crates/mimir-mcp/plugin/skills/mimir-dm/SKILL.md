# Mimir DM - D&D 5e Campaign Management

This skill should be used when the user asks to "create a D&D campaign", "make a new module", "add an NPC", "create an encounter", "search for monsters", "find D&D items", "list campaigns", "set up a dungeon crawl", "create a mystery adventure", "add loot to characters", "search for traps", "manage character inventory", "build an adventure", "populate a dungeon", "give items to players", "create a villain", "add treasure", "write session notes", or mentions "Mimir campaign", "D&D 5e authoring", or "dungeon master tools". Provides MCP tools for campaign management, module creation, NPC authoring, and catalog searching.

## Getting Started

Before using Mimir tools, you must:
1. List available campaigns with `list_campaigns`
2. Set an active campaign with `set_active_campaign`

All module, character, and document operations require an active campaign.

## Available MCP Tools

### Campaign Management

| Tool | Purpose |
|------|---------|
| `list_campaigns` | List all available campaigns |
| `set_active_campaign` | Set the active campaign for subsequent operations |
| `get_campaign_details` | Get full campaign info including modules and characters |
| `get_campaign_sources` | Get enabled source books for the campaign |

### Module Management

| Tool | Purpose |
|------|---------|
| `create_module` | Create a new module (adventure chapter) |
| `list_modules` | List all modules in the active campaign |
| `get_module_details` | Get module with documents, monsters, and items |
| `add_monster_to_module` | Add a monster from the catalog to a module |
| `add_item_to_module` | Add an item from the catalog as module loot |

### Document Management

| Tool | Purpose |
|------|---------|
| `list_documents` | List all documents in a module |
| `read_document` | Read the full content of a document |
| `create_document` | Create a new document in a module |
| `edit_document` | Edit a document using search and replace |

### Character Management

| Tool | Purpose |
|------|---------|
| `list_characters` | List characters (filter by PC/NPC) |
| `get_character` | Get full character details with classes and inventory |
| `create_character` | Create a new PC or NPC |
| `edit_character` | Update character fields (name, role, location, etc.) |
| `add_item_to_character` | Add an item to a character's inventory |

### Catalog Search

| Tool | Purpose |
|------|---------|
| `search_monsters` | Search monsters by name, CR, type |
| `search_items` | Search items by name, rarity, type |
| `search_spells` | Search spells by name, level, school |

## Source Filtering

When a campaign is active, catalog searches are automatically filtered to only include content from the campaign's enabled source books. This ensures monsters, items, and spells match the campaign's allowed content.

## Document Types

Modules support various document types for organizing content:
- `module_overview` - Main module description and structure
- `play_notes` - Session tracking and notes
- `backstory` - Background and history
- `read_aloud` - Text to read to players
- `dm_notes` - DM-only information
- `description` - Location or encounter descriptions
- `user_document` - Custom documents

## Best Practices

1. **Always set an active campaign** before using module/character tools
2. **Use `get_module_details`** to see the full structure before editing
3. **Search the catalog first** to find exact monster/item names before adding
4. **NPCs can be assigned roles and locations** for organization
5. **Documents support markdown formatting** for rich content
