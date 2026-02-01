---
name: mimir-dm
description: >-
  This skill should be used when the user asks to "create a D&D campaign",
  "make a new module", "add an NPC", "create an encounter", "search for monsters",
  "find D&D items", "search for spells", "list campaigns", "set up a dungeon crawl",
  "create a mystery adventure", "add loot to characters", "search for traps",
  "manage character inventory", "build an adventure", "populate a dungeon",
  "give items to players", "create a villain", "add treasure", "write session notes",
  "create campaign document", "add world lore", "campaign-level document",
  "upload a map", "place tokens", "level up character", "set ability scores",
  "set currency", "search races", "search classes", "search backgrounds",
  "search feats", "search conditions", "delete module", "delete character",
  "delete document", "create campaign", "export campaign", "import campaign",
  "homebrew item", "homebrew monster", "homebrew spell", "create custom monster",
  "create custom item", "create custom spell", "clone monster", "clone item",
  "clone spell", or mentions "Mimir campaign", "D&D 5e authoring",
  or "dungeon master tools". Provides MCP tools for campaign management, module
  creation, NPC authoring, character management, map/token placement, document
  management, homebrew content creation, and catalog searching.
---

# Mimir DM - D&D 5e Campaign Management

## Getting Started

Before using any module, character, or document tools:

1. List available campaigns with `list_campaigns` (or create one with `create_campaign`)
2. Set an active campaign with `set_active_campaign`

All subsequent operations require an active campaign.

## Important Patterns

### Read Before Edit
Always call `get_character` / `read_document` / `get_module_details` / `get_homebrew_item` / `get_homebrew_monster` / `get_homebrew_spell` before making edits to confirm current state. Never edit blind.

### Catalog Exact-Match
Search tools return partial matches. When adding monsters, items, or equipment:
1. Search the catalog first (`search_monsters`, `search_items`, `search_spells`, etc.)
2. Pick the **exact** matching name from results
3. Use that exact name string in `add_monster_to_module` / `add_item_to_character`

### Clone From Catalog Before Creating Homebrew
When a user wants a custom monster, item, or spell, **always try cloning from the catalog first** rather than building from scratch. A cloned entry preserves correct 5etools JSON structure and only needs targeted edits.

1. Search the catalog for the closest match (`search_monsters`, `search_items`, `search_spells`)
2. Present options and let the user pick the base creature/item/spell
3. Clone it with `create_homebrew_*` using `cloned_from_name` and `cloned_from_source`
4. Edit the `data` JSON to apply the user's requested changes

Building homebrew JSON from scratch is error-prone — the data blobs follow 5etools format which has many nested structures. Cloning and modifying is almost always safer.

### Error Handling
- If a tool call fails, report the error to the user rather than silently continuing
- If a catalog search returns no results, try alternate names or broader search terms
- If `add_monster_to_module` fails, verify the monster name matches the catalog exactly

### Ask When Unsure
If you are uncertain about any creative decision — monster choice, encounter composition, loot selection, NPC details, homebrew modifications — **ask the user**. Present 2-3 options with brief rationale and let them choose. It is always better to ask than to guess wrong and have to undo work.

### Human-in-the-Loop for Creative Decisions

**CRITICAL**: The user is the Dungeon Master. You are their assistant, not a co-author. Never make narrative, creative, or design decisions without explicit user approval. Always present options and let the user choose. Execute mechanically once they have decided.

For the full guidelines on what requires approval vs what you can do autonomously, see references/dm-assistant-guidelines.md.

## Core Workflows

### Campaign Management

- `list_campaigns` — List all campaigns
- `create_campaign` — Create a new campaign (name, description)
- `get_campaign_details` — View active campaign details
- `update_campaign` — Update campaign name or description
- `set_active_campaign` — Set the working campaign
- `get_campaign_sources` — View enabled source books
- `export_campaign` — Export campaign to archive file
- `import_campaign` — Import a campaign archive
- `preview_archive` — Preview archive contents before importing
- `delete_campaign` — Delete a campaign (irreversible)

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
7. Search spells with `search_spells` if the module involves spellcasting NPCs or traps

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
3. Search spells with `search_spells` if encounter involves spellcasters
4. Add treasure with `add_item_to_module`

### Upload and Populate a Map

1. Upload a UVTT map file with `create_map` (module_id, name, file_path)
2. Place monsters with `add_token_to_map` (map_id, monster_name, x, y, label)
3. Place NPCs with `add_token_to_map` (map_id, npc_id, x, y, label)
4. Review with `get_map` to see all token placements
5. Remove misplaced tokens with `remove_token`

### Manage Character Inventory

- `get_character_inventory` — view full inventory
- `add_item_to_character` — add items (set equipped, attuned flags)
- `update_character_inventory` — change quantity, equipped, or attuned status
- `remove_item_from_character` — remove items by inventory ID

### Review Module Structure

Use `get_module_details` to see the full structure before editing — it returns documents, monsters, and items in one call.

### Create Homebrew Content

Homebrew content lets the DM create custom items, monsters, and spells that don't exist in the catalog. The recommended workflow is **clone and edit**, not build from scratch.

#### Homebrew Items
1. Search the catalog with `search_items` to find a similar base item
2. Clone with `create_homebrew_item` (name, data, item_type, rarity, cloned_from_name, cloned_from_source)
3. Review with `get_homebrew_item`
4. Edit with `update_homebrew_item` to refine the data JSON
5. List all with `list_homebrew_items`

#### Homebrew Monsters
1. Search the catalog with `search_monsters` to find a similar base creature
2. Clone with `create_homebrew_monster` (name, data, cr, creature_type, size, cloned_from_name, cloned_from_source)
3. Review with `get_homebrew_monster`
4. Edit with `update_homebrew_monster` to adjust the stat block JSON
5. List all with `list_homebrew_monsters`

#### Homebrew Spells
1. Search the catalog with `search_spells` to find a similar base spell
2. Clone with `create_homebrew_spell` (name, data, level, school, cloned_from_name, cloned_from_source)
3. Review with `get_homebrew_spell`
4. Edit with `update_homebrew_spell` to adjust the spell data JSON
5. List all with `list_homebrew_spells`

**Important**: The `data` field is a JSON string following 5etools format. Always clone from catalog to get the correct structure, then make targeted edits. If you must build from scratch, validate the JSON is well-formed before saving.

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
- When deleting homebrew content, check if any modules or characters reference it first
- Homebrew monsters appear with source "HB" in module monster lists
