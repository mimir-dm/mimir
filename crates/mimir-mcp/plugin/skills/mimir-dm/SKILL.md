---
name: mimir-dm
description: >-
  This skill should be used when the user asks to "create a D&D campaign",
  "make a new module", "add an NPC", "create an encounter", "search for monsters",
  "find D&D items", "list campaigns", "set up a dungeon crawl", "create a mystery adventure",
  "add loot to characters", "search for traps", "manage character inventory",
  "build an adventure", "populate a dungeon", "give items to players", "create a villain",
  "add treasure", "write session notes", or mentions "Mimir campaign", "D&D 5e authoring",
  or "dungeon master tools". Provides MCP tools for campaign management, module creation,
  NPC authoring, and catalog searching.
---

# Mimir DM - D&D 5e Campaign Management

## Getting Started

Before using any module, character, or document tools:

1. List available campaigns with `list_campaigns`
2. Set an active campaign with `set_active_campaign`

All subsequent operations require an active campaign.

## Core Workflows

### Create and Populate a Module

1. Set the active campaign
2. Create the module with `create_module` (name, type, description)
3. Add narrative documents with `create_document` (backstory, read_aloud, dm_notes, description, custom)
4. Search the catalog with `search_monsters` to find exact names
5. Add monsters with `add_monster_to_module` (include count and notes)
6. Search items with `search_items` and add loot with `add_item_to_module`

### Create and Equip an NPC

1. Create the character with `create_character` (name, character_type: "npc", race_name)
2. Set role and location with `edit_character` (npc_role, npc_location)
3. Equip with `add_item_to_character` (item_name, equipped: true)
4. Verify with `get_character`

### Populate an Encounter

1. Search monsters by name, CR range, or type with `search_monsters`
2. Add each monster to the module with `add_monster_to_module` (specify count)
3. Add treasure with `add_item_to_module`

### Review Module Structure

Use `get_module_details` to see the full structure before editing — it returns documents, monsters, and items in one call.

## Document Types

Modules support these document types for `create_document`:

- `backstory` — Background and history
- `read_aloud` — Text to read to players
- `dm_notes` — DM-only information
- `description` — Location or encounter descriptions
- `custom` — Custom documents

## Source Filtering

When a campaign is active, catalog searches are automatically filtered to only include content from the campaign's enabled source books. This ensures monsters, items, and spells match the campaign's allowed content.

## Best Practices

1. Always set an active campaign before using module/character tools
2. Use `get_module_details` to see the full structure before editing
3. Search the catalog first to find exact monster/item names before adding
4. Assign NPCs roles and locations for organization
5. Documents support markdown formatting for rich content

## Workflow Examples

For step-by-step workflows, refer to:
- examples/create-module-workflow.md — Create and populate an adventure module
- examples/create-npc-workflow.md — Create and equip an NPC
