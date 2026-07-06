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

## Ability Scores & Currency (edit_character)

The MCP `edit_character` tool takes each score and coin as its **own named
integer parameter** — there is **no `ability_scores` or `currency` array**.
Passing an array is silently ignored (the call still returns success but sets
nothing), so always use the named parameters below.

### Ability Scores
`strength`, `dexterity`, `constitution`, `intelligence`, `wisdom`, `charisma`

Example: `edit_character(character_id: "...", strength: 16, dexterity: 12, constitution: 14, intelligence: 10, wisdom: 13, charisma: 11)`

Provide only the scores you want to change; the rest keep their current values.
Ability scores cannot be set in `create_character` — create the character first,
then set scores with `edit_character`.

### Currency
`cp`, `sp`, `ep`, `gp`, `pp` (copper, silver, electrum, gold, platinum)

Example: `edit_character(character_id: "...", gp: 50)` (set 50 gold, other coins unchanged)

### Ability Increases (Level Up)

The MCP `level_up_character` tool likewise uses **individual parameters**, not an array:
- `asi_ability1` — First ability to increase (e.g., "Constitution")
- `asi_increase1` — Amount for first ability (1 or 2)
- `asi_ability2` — Second ability to increase (optional)
- `asi_increase2` — Amount for second ability

Example: +1 CON, +1 CHA → `asi_ability1: "Constitution", asi_increase1: 1, asi_ability2: "Charisma", asi_increase2: 1`

> **Note**: The Mimir desktop app's internal (Tauri) commands use `[STR, DEX, CON, INT, WIS, CHA]` and `[CP, SP, EP, GP, PP]` array formats. Those are **not** the MCP interface — through MCP, always use the named parameters above.

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

### search_catalog
- `category` (required) — `"monster"`, `"item"`, `"spell"`, `"race"`, `"class"`, `"background"`, `"feat"`, `"condition"`
- `name` — Partial name match
- `limit` — Max results (default: 20)

**Monster-specific filters** (category: `"monster"`):
- `cr_min` / `cr_max` — Challenge rating range (number)
- `monster_type` — Type filter (e.g., `"undead"`, `"dragon"`)

**Item-specific filters** (category: `"item"`):
- `rarity` — `"common"`, `"uncommon"`, `"rare"`, `"very rare"`, `"legendary"`, `"artifact"`
- `item_type` — Type filter (e.g., `"weapon"`, `"armor"`, `"wondrous item"`)

**Spell-specific filters** (category: `"spell"`):
- `level` — Spell level (integer, 0 for cantrips)
- `school` — School of magic (e.g., `"evocation"`, `"necromancy"`)
- `class_name` — Filter by class spell list

## Homebrew Tool Parameters

### create_homebrew
- `content_type` (required) — `"item"`, `"monster"`, or `"spell"`
- `name` (required) — Name of the homebrew entry
- `data` (required) — JSON string with the entry data
- `cloned_from_name` — Catalog entry this was based on
- `cloned_from_source` — Source book of the base entry

**Item-specific fields** (content_type: `"item"`):
- `item_type` — `"weapon"`, `"armor"`, `"potion"`, `"ring"`, `"rod"`, `"scroll"`, `"staff"`, `"wand"`, `"wondrous item"`, `"adventuring gear"`
- `rarity` — `"common"`, `"uncommon"`, `"rare"`, `"very rare"`, `"legendary"`, `"artifact"`

**Monster-specific fields** (content_type: `"monster"`):
- `cr` — Challenge rating (e.g., `"1/4"`, `"5"`, `"20"`)
- `creature_type` — `"humanoid"`, `"dragon"`, `"undead"`, `"fiend"`, etc.
- `size` — `"T"`, `"S"`, `"M"`, `"L"`, `"H"`, `"G"`

**Spell-specific fields** (content_type: `"spell"`):
- `level` — Spell level (integer, 0 for cantrip)
- `school` — School of magic (e.g., `"evocation"`, `"necromancy"`)

### update_homebrew
Accepts the same fields as `create_homebrew` plus `id` (required). Only fields you provide will be updated.

### get_homebrew
- `content_type` (required) — `"item"`, `"monster"`, or `"spell"`
- `id` (required) — The homebrew entry ID

### list_homebrew
- `content_type` (required) — `"item"`, `"monster"`, or `"spell"`

### delete_homebrew
- `content_type` (required) — `"item"`, `"monster"`, or `"spell"`
- `id` (required) — The homebrew entry ID

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
- `output_path` (required) — Directory (on the server host) where the archive is written
- Returns the archive file path

### import_campaign
- `archive_path` (required) — Path to the archive file to import
- `new_name` — Optional new name for the imported campaign

### preview_archive
- `archive_path` (required) — Path to the archive file to inspect before importing
