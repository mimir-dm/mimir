# Tool Parameter Quick Reference

## Response Formats

All MCP tools return standardized JSON responses. The format depends on the operation type:

### List Responses
```json
{ "<collection>": [...], "count": <n> }
```
Examples: `{ "characters": [...], "count": 5 }`, `{ "monsters": [...], "count": 20 }`

### Get/Read Responses
```json
{ "<entity>": { ... } }
```
Examples: `{ "character": { "id": "...", "name": "..." } }`, `{ "document": { ... } }`

### Create Responses
```json
{ "status": "created", "<entity>": { ... } }
```

### Update Responses
```json
{ "status": "updated", "<entity>": { ... } }
```

### Delete Responses
```json
{ "status": "deleted", "id": "<id>" }
```

### Add Responses (adding items to collections)
```json
{ "status": "added", "<entity>": { ... } }
```

### Remove Responses (removing items from collections)
```json
{ "status": "removed", "id": "<id>" }
```

### Success Responses (special operations like level_up)
```json
{ "status": "success", "data": { ... } }
```

### Complex Read Responses
Some tools return composite data and use `ok()` which passes through the JSON directly:
```json
{ "map": { ... }, "tokens": [...] }
```

## Array Formats

### Ability Scores
`ability_scores: [STR, DEX, CON, INT, WIS, CHA]`

Example: `[16, 12, 14, 10, 13, 11]`

### Currency
`currency: [CP, SP, EP, GP, PP]`

Example: `[0, 0, 0, 50, 0]` (50 gold)

### Ability Increases (Level Up — MCP tool)

The MCP `level_up_character` tool uses **individual parameters**, not an array:
- `asi_ability1` — First ability to increase (e.g., "Constitution")
- `asi_increase1` — Amount for first ability (1 or 2)
- `asi_ability2` — Second ability to increase (optional)
- `asi_increase2` — Amount for second ability

Example: +1 CON, +1 CHA → `asi_ability1: "Constitution", asi_increase1: 1, asi_ability2: "Charisma", asi_increase2: 1`

> **Note**: The Tauri frontend command uses a `[STR, DEX, CON, INT, WIS, CHA]` array format instead. The MCP tool uses the named parameters above.

## Enum Values

### character_type
- `"pc"` — Player character
- `"npc"` — Non-player character

### document_type
- `"backstory"` — Background and history
- `"read_aloud"` — Text to read to players
- `"dm_notes"` — DM-only information
- `"description"` — Location or encounter descriptions
- `"custom"` — Custom documents

### module_type
- `"adventure"` — Adventure module (default)
- `"location"` — Location description
- `"organization"` — Organization or faction

### hp_method (level_up_character)
- `"average"` — Use average HP roll
- `"roll"` — Roll for HP
- `"manual"` — Manually specify HP gained

### asi_type (level_up_character)
- `"asi"` — Ability Score Improvement (+2 to one or +1 to two)
- `"feat"` — Take a feat instead

## Common Filter Parameters

### list_characters
- `character_type` — `"pc"` or `"npc"`
- `module_id` — Filter NPCs by module assignment
- `location` — Filter NPCs by location string
- `faction` — Filter NPCs by faction string

### search_monsters
- `name` — Partial name match
- `cr_min` / `cr_max` — Challenge rating range (number)
- `monster_type` — Type filter (e.g., `"undead"`, `"dragon"`)
- `limit` — Max results (default: 20)

### search_items
- `name` — Partial name match
- `rarity` — `"common"`, `"uncommon"`, `"rare"`, `"very rare"`, `"legendary"`, `"artifact"`
- `item_type` — Type filter (e.g., `"weapon"`, `"armor"`, `"wondrous item"`)
- `limit` — Max results (default: 20)

### search_spells
- `name` — Partial name match
- `level` — Spell level (integer, 0 for cantrips)
- `school` — School of magic (e.g., `"evocation"`, `"necromancy"`)
- `class_name` — Filter by class spell list
- `limit` — Max results (default: 20)

## Homebrew Tool Parameters

### create_homebrew_item
- `name` (required) — Item name
- `data` (required) — JSON string with item data
- `item_type` — `"weapon"`, `"armor"`, `"potion"`, `"ring"`, `"rod"`, `"scroll"`, `"staff"`, `"wand"`, `"wondrous item"`, `"adventuring gear"`
- `rarity` — `"common"`, `"uncommon"`, `"rare"`, `"very rare"`, `"legendary"`, `"artifact"`
- `cloned_from_name` — Catalog item this was based on
- `cloned_from_source` — Source book of the base item

### create_homebrew_monster
- `name` (required) — Monster name
- `data` (required) — JSON string with stat block
- `cr` — Challenge rating (e.g., `"1/4"`, `"5"`, `"20"`)
- `creature_type` — `"humanoid"`, `"dragon"`, `"undead"`, `"fiend"`, etc.
- `size` — `"T"`, `"S"`, `"M"`, `"L"`, `"H"`, `"G"`
- `cloned_from_name` — Catalog monster this was based on
- `cloned_from_source` — Source book of the base monster

### create_homebrew_spell
- `name` (required) — Spell name
- `data` (required) — JSON string with spell data
- `level` — Spell level (integer, 0 for cantrip)
- `school` — School of magic (e.g., `"evocation"`, `"necromancy"`)
- `cloned_from_name` — Catalog spell this was based on
- `cloned_from_source` — Source book of the base spell

### update_homebrew_* tools
All update tools accept the same fields as their create counterparts plus `id` (required). Only fields you provide will be updated.

## Character Spell Tools

### add_character_spell
- `character_id` (required) — The character's ID
- `spell_name` (required) — Spell name (e.g., "Fireball")
- `spell_source` (required) — Source book abbreviation (e.g., "PHB", "XGE") or "HB" for homebrew
- `source_class` (required) — Class that grants this spell (e.g., "Wizard", "Cleric")
- `prepared` — Whether the spell starts prepared (default: false)

### remove_character_spell
- `character_id` (required) — The character's ID
- `spell_name` (required) — Name of the spell to remove
- `source_class` — Class that granted the spell. If omitted, removes all instances of the spell.

### list_character_spells
- `character_id` (required) — The character's ID
- `source_class` — Filter by granting class (e.g., "Wizard")
- `prepared_only` — Only return prepared spells (default: false)

## Map Generation Tools

### generate_map
- `config_yaml` — YAML configuration string (mutually exclusive with `preset`)
- `preset` — Biome preset name: `"forest"`, `"grassland"`, `"cave"` (mutually exclusive with `config_yaml`)
- `output_path` (required) — Absolute path for the output `.dungeondraft_map` file
- `seed` — Random seed override (integer) for reproducible generation

### list_map_presets
No parameters. Returns `{ presets: [{ name, description, default_size }] }`.

### validate_map_config
- `config_yaml` (required) — YAML configuration to validate

Returns `{ valid: bool, errors: [{ field, message }] }` or `{ valid: false, parse_error: "..." }` if YAML is malformed.

## Campaign Management Tools

### create_campaign
- `name` (required) — Campaign name
- `description` — Campaign description

### update_campaign
- `name` — New campaign name
- `description` — New description

### export_campaign
- Returns the archive file path

### import_campaign
- `file_path` (required) — Path to the archive file

### preview_archive
- `file_path` (required) — Path to the archive file to inspect before importing
