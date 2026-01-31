# Mimir DM - D&D 5e Campaign Management

Mimir is a campaign management tool for Dungeon Masters running D&D 5e games. This Claude Code plugin provides MCP tools for authoring modules, managing NPCs, and searching the 5etools catalog.

## Installation

### Prerequisites

1. **Mimir Desktop App**: Install from [mimir releases](https://github.com/mimir-dm/mimir/releases)
2. **mimir-mcp binary**: Build from source or install via the Mimir installer

### Building from Source

```bash
cargo build --release -p mimir-mcp
```

The binary will be at `target/release/mimir-mcp`.

### Environment Setup

The MCP server automatically detects the default database location. You can optionally override it with the `MIMIR_DATABASE_PATH` environment variable:

```bash
# macOS (production)
export MIMIR_DATABASE_PATH="$HOME/Library/Application Support/com.mimir.app/data/mimir.db"

# macOS (dev mode)
export MIMIR_DATABASE_PATH="$HOME/Library/Application Support/com.mimir.app/dev/data/mimir.db"
```

You can find your exact database path in Mimir's Settings > Integrations tab.

### Claude Code Plugin Installation

```bash
claude plugin add /path/to/mimir/crates/mimir-mcp/plugin
```

Or add the MCP server directly:
```bash
claude mcp add mimir \
  -e MIMIR_DATABASE_PATH="$HOME/Library/Application Support/com.mimir.app/data/mimir.db" \
  -- mimir-mcp
```

## Available Commands

- `/mimir-campaigns` - List all available campaigns
- `/create-module <name>` - Create a new module in the active campaign
- `/search-monsters [query] [--cr <rating>]` - Search the monster catalog

## Getting Started

1. **List campaigns**: Use `list_campaigns` to see available campaigns
2. **Set active campaign**: Use `set_active_campaign` with the campaign ID
3. **Start authoring**: Create modules, add NPCs, populate encounters

## Tool Categories

### Campaign Management
- `list_campaigns` - List all available campaigns
- `set_active_campaign` - Set the active campaign for subsequent operations
- `get_campaign_details` - Get full campaign info including modules and characters
- `get_campaign_sources` - Get enabled source books for the campaign
- `create_campaign` - Create a new campaign (name, description)
- `update_campaign` - Update campaign name or description
- `delete_campaign` - Delete a campaign and all its data
- `export_campaign` - Export campaign as a shareable archive
- `import_campaign` - Import a campaign from an archive
- `preview_archive` - Preview archive contents without importing

### Module Management
- `create_module` - Create a new module (adventure chapter)
- `list_modules` - List all modules in the active campaign
- `get_module_details` - Get module with documents, monsters, and items
- `update_module` - Update module name or description
- `delete_module` - Delete a module and all its contents
- `add_monster_to_module` - Add a monster from the catalog to a module
- `add_item_to_module` - Add an item from the catalog as module loot

### Document Management
- `list_documents` - List documents in a module, or campaign-level documents (omit `module_id`)
- `read_document` - Read the full content of a document
- `create_document` - Create a document in a module or at the campaign level (omit `module_id`)
- `edit_document` - Edit a document using search and replace
- `delete_document` - Delete a document

### Character Management
- `list_characters` - List characters (filter by PC/NPC, location, faction)
- `get_character` - Get full character details with classes and inventory
- `create_character` - Create a new PC or NPC
- `edit_character` - Update character fields (name, role, location, ability scores, currency, race, background, etc.)
- `add_item_to_character` - Add an item to a character's inventory (with equipped/attuned flags)
- `delete_character` - Delete a character
- `level_up_character` - Level up a character (class, HP method, ASI/feat choice)
- `remove_item_from_character` - Remove an item from inventory
- `update_character_inventory` - Update quantity, equipped, or attuned status of an inventory item
- `get_character_inventory` - Get a character's full inventory

### Map Management
- `create_map` - Upload a UVTT map file to a module
- `list_maps` - List all maps in a module
- `get_map` - Get map details including token placements
- `update_map` - Update map name or notes
- `delete_map` - Delete a map
- `add_token_to_map` - Place a monster or NPC token on a map
- `list_tokens_on_map` - List all tokens placed on a map
- `remove_token` - Remove a token from a map

### Catalog Search
- `search_monsters` - Search monsters by name, CR, type
- `search_items` - Search items by name, rarity, type
- `search_spells` - Search spells by name, level, school
- `search_races` - Search races by name
- `search_classes` - Search classes by name
- `search_backgrounds` - Search backgrounds by name
- `search_feats` - Search feats by name
- `search_conditions` - Search conditions by name

## Common Workflows

### Creating a New Module

```
1. set_active_campaign(campaign_id)
2. create_module(name="The Haunted Manor", module_type="horror")
3. edit_document(document_id, search="# Module Overview", replace="...")
```

### Populating an Encounter

```
1. search_monsters(name="goblin", cr="1/4")
2. add_monster_to_module(module_id, monster_name="Goblin", count=6)
3. add_monster_to_module(module_id, monster_name="Bugbear", count=1, notes="Leader")
```

### Creating an NPC

```
1. create_character(name="Sildar Hallwinter", character_type="npc", race_name="Human")
2. edit_character(character_id, npc_role="Quest Giver", npc_location="Phandalin")
3. edit_character(character_id, ability_scores=[14,12,13,10,11,8], currency=[50,0,0,0,0])
4. add_item_to_character(character_id, item_name="Longsword", equipped=true)
```

### Creating a Full PC

```
1. search_races(name="Elf") — find exact race name/source
2. search_classes(name="Ranger") — find exact class name
3. search_backgrounds(name="Outlander") — find exact background name
4. create_character(name="Thalion", character_type="pc", race_name="Elf", class_name="Ranger")
5. edit_character(character_id, ability_scores=[10,16,14,12,14,8], currency=[0,15,0,0,0])
6. add_item_to_character(character_id, item_name="Longbow", equipped=true)
7. add_item_to_character(character_id, item_name="Studded Leather Armor", equipped=true)
```

### Leveling Up a Character

```
1. get_character(character_id) — check current level and class
2. level_up_character(character_id, class_name="Ranger", hp_method="average")
```

### Adding Treasure to a Module

```
1. search_items(rarity="rare", item_type="wondrous item")
2. add_item_to_character(npc_id, item_name="Cloak of Protection")
```

### Uploading a Map

```
1. create_map(module_id, name="Goblin Cave", file_path="/path/to/cave.uvtt")
2. search_monsters(name="Goblin")
3. add_token_to_map(map_id, monster_name="Goblin", x=5.0, y=3.0, label="Guard 1")
4. add_token_to_map(map_id, monster_name="Goblin", x=7.0, y=3.0, label="Guard 2")
```

## Source Filtering

When a campaign is active, catalog searches are automatically filtered to only include content from the campaign's enabled source books. This ensures monsters, items, and spells match the campaign's allowed content.

## Document Types

Documents can belong to a **module** or directly to the **campaign** (omit `module_id`). Campaign-level documents are useful for world lore, session notes, and campaign-wide references.

Supported document types:
- `backstory` - Background and history
- `read_aloud` - Text to read to players
- `dm_notes` - DM-only information
- `description` - Location or encounter descriptions
- `custom` - Custom documents

## Important: The User is the Dungeon Master

Never make narrative, creative, or design decisions autonomously. Always ask the user before choosing monsters, writing backstory, naming NPCs, deciding encounter composition, or making any world-building choices. Present options and let the user decide. You execute mechanically once they have chosen.

## Tips

- Always set an active campaign before using module/character tools
- Use `get_module_details` to see the full structure before editing
- Monsters and items reference the 5etools catalog - search first to find exact names
- NPCs can be assigned roles and locations for organization
- Documents support markdown formatting
