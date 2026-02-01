# Tool Parameter Quick Reference

## Array Formats

### Ability Scores
`ability_scores: [STR, DEX, CON, INT, WIS, CHA]`

Example: `[16, 12, 14, 10, 13, 11]`

### Currency
`currency: [CP, SP, EP, GP, PP]`

Example: `[0, 0, 0, 50, 0]` (50 gold)

### Ability Increases (Level Up)
`ability_increases: [STR, DEX, CON, INT, WIS, CHA]`

Each value is the amount to increase. Example: `[0, 0, 1, 0, 0, 1]` (+1 CON, +1 CHA)

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
