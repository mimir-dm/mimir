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
- `cr_min` / `cr_max` — Challenge rating range
- `monster_type` — Type filter (e.g., `"undead"`, `"dragon"`)

### search_items
- `name` — Partial name match
- `rarity` — `"common"`, `"uncommon"`, `"rare"`, `"very rare"`, `"legendary"`, `"artifact"`
- `item_type` — Type filter (e.g., `"weapon"`, `"armor"`, `"wondrous item"`)
