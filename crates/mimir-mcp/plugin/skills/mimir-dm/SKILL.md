---
name: mimir-dm
description: >-
  This skill should be used when the user asks to "create a D&D campaign",
  "make a new module", "add an NPC", "create an encounter", "search for monsters",
  "find D&D items", "list campaigns", "set up a dungeon crawl", "create a mystery adventure",
  "add loot to characters", "search for traps", "manage character inventory",
  "build an adventure", "populate a dungeon", "give items to players", "create a villain",
  "add treasure", "write session notes", "create campaign document", "add world lore",
  "campaign-level document", "upload a map", "place tokens", "level up character",
  "set ability scores", "set currency", "search races", "search classes",
  "search backgrounds", "search feats", "search conditions", "delete module",
  "delete character", "delete document", "create campaign",
  or mentions "Mimir campaign", "D&D 5e authoring",
  or "dungeon master tools". Provides MCP tools for campaign management, module creation,
  NPC authoring, character management, map/token placement, document management,
  and catalog searching.
---

# Mimir DM - D&D 5e Campaign Management

## Getting Started

Before using any module, character, or document tools:

1. List available campaigns with `list_campaigns` (or create one with `create_campaign`)
2. Set an active campaign with `set_active_campaign`

All subsequent operations require an active campaign.

## Important Patterns

### Read Before Edit
Always call `get_character` / `read_document` / `get_module_details` before making edits to confirm current state.

### Catalog Exact-Match
Search tools return partial matches. When adding monsters, items, or equipment:
1. Search the catalog first (`search_monsters`, `search_items`, etc.)
2. Pick the **exact** matching name from results
3. Use that exact name string in `add_monster_to_module` / `add_item_to_character`

### Error Handling
- If a tool call fails, report the error to the user rather than silently continuing
- If a catalog search returns no results, try alternate names or broader search terms
- If `add_monster_to_module` fails, verify the monster name matches the catalog exactly

### Human-in-the-Loop for Creative Decisions

**CRITICAL**: The user is the Dungeon Master. You are their assistant, not a co-author. Never make narrative, creative, or design decisions without explicit user approval. Always present options and let the user choose. Execute mechanically once they have decided.

For the full guidelines on what requires approval vs what you can do autonomously, see references/dm-assistant-guidelines.md.

## Core Workflows

### Create Campaign-Level Documents

Campaign-level documents are not tied to any module — use them for world lore, session notes, or campaign-wide references.

1. Set the active campaign
2. Create a document with `create_document` (title, document_type) — **omit `module_id`**
3. List campaign documents with `list_documents` — **omit `module_id`**

### Create and Populate a Module

1. Set the active campaign
2. Create the module with `create_module` (name, type, description)
3. Add narrative documents with `create_document` (module_id, backstory, read_aloud, dm_notes, description, custom)
4. Search the catalog with `search_monsters` to find exact names
5. Add monsters with `add_monster_to_module` (include count and notes)
6. Search items with `search_items` and add loot with `add_item_to_module`

### Create and Equip an NPC

1. Create the character with `create_character` (name, character_type: "npc", race_name)
2. Set role and location with `edit_character` (npc_role, npc_location)
3. Set ability scores with `edit_character` (ability_scores: [STR,DEX,CON,INT,WIS,CHA])
4. Set currency with `edit_character` (currency: [CP,SP,EP,GP,PP])
5. Equip with `add_item_to_character` (item_name, equipped: true, attuned: true if applicable)
6. Verify with `get_character`

### Create a Full PC

1. Search `search_races`, `search_classes`, `search_backgrounds` to find exact names
2. Create with `create_character` (name, character_type: "pc", race_name, class_name)
3. Set ability scores and currency with `edit_character`
4. Set race/background with `edit_character` (race_name, race_source, background_name, background_source)
5. Add equipment with `add_item_to_character`

### Level Up a Character

1. `get_character` to check current level and class
2. `level_up_character` (character_id, class_name, hp_method: "average"|"roll"|"manual")
3. Optionally include subclass_name, asi_type ("asi"|"feat"), ability_increases, feat_name

### Populate an Encounter

1. Search monsters by name, CR range, or type with `search_monsters`
2. Add each monster to the module with `add_monster_to_module` (specify count)
3. Add treasure with `add_item_to_module`

### Upload and Populate a Map

1. Upload a UVTT map file with `create_map` (module_id, name, file_path)
2. Place monsters with `add_token_to_map` (map_id, monster_name, x, y, label)
3. Place NPCs with `add_token_to_map` (map_id, npc_id, x, y, label)
4. Review with `get_map` to see all token placements

### Manage Character Inventory

- `get_character_inventory` — view full inventory
- `add_item_to_character` — add items (set equipped, attuned flags)
- `update_character_inventory` — change quantity, equipped, or attuned status
- `remove_item_from_character` — remove items by inventory ID

### Review Module Structure

Use `get_module_details` to see the full structure before editing — it returns documents, monsters, and items in one call.

## Document Types

Documents can belong to a **module** (provide `module_id`) or the **campaign** (omit `module_id`).

Supported types for `create_document`:

- `backstory` — Background and history
- `read_aloud` — Text to read to players
- `dm_notes` — DM-only information
- `description` — Location or encounter descriptions
- `custom` — Custom documents

## Source Filtering

When a campaign is active, catalog searches are automatically filtered to only include content from the campaign's enabled source books. This ensures monsters, items, and spells match the campaign's allowed content.

## Additional Tips

- Assign NPCs roles and locations for organization
- Documents support markdown formatting for rich content
- Use `list_characters` with location or faction filters to find NPCs by area
- For array format details (ability_scores, currency, etc.), see references/tool-parameter-reference.md

## Workflow Examples

For step-by-step workflows, refer to:
- examples/create-module-workflow.md — Create and populate an adventure module
- examples/create-npc-workflow.md — Create and equip an NPC
- examples/character-creation-workflow.md — Create a full PC from scratch
- examples/level-up-workflow.md — Level up, multiclass, feats, and ASIs
