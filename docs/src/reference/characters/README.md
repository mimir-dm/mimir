# Character and NPC Management

Mimir provides a comprehensive character management system for tracking player characters and non-player characters within campaigns. The system handles the full lifecycle of D&D 5e characters, from creation through leveling, spell slot tracking, and version history.

## Data Model

Players represent the real people at your table. Each player record stores a name and optional email address, and can be associated with multiple characters across different campaigns through the campaign_players relationship table.

Characters are the in-game personas controlled by players or the DM. A character record contains the character name, race, class, and level alongside the six ability scores. The system tracks current, maximum, and temporary hit points separately to support combat scenarios. Each character also maintains armor class, movement speed, proficiency selections, class features, known and prepared spells with slot tracking, and a complete inventory.

The distinction between player characters and NPCs is handled through the player_id field. When this field is null, the character is treated as an NPC. When it references a valid player record, the character appears in player-specific views and receives full character sheet treatment.

## Spell Slot Calculation

The spell slot system automatically calculates available slots based on class and level. Full casters such as Wizard, Cleric, Druid, Bard, and Sorcerer follow the standard progression table. Half casters including Paladin and Ranger gain slots at half the rate, while third casters like Eldritch Knight and Arcane Trickster progress even more slowly. Warlocks use the separate Pact Magic system with fewer slots that recover on short rest.

Multiclass characters have their slots calculated according to the multiclassing rules, summing effective caster levels from each class before consulting the appropriate progression table. The system stores both the calculated slot maximums and current availability, updating the latter when spells are cast or rests are taken.

## Character Versioning

Significant character changes create version snapshots in the database. These snapshots capture the complete character state at moments like level advancement, major stat modifications, or class feature acquisitions. The version history allows reviewing a character's progression over time and can help resolve questions about when particular changes occurred.

## Tauri Commands

The character system exposes its functionality through Tauri commands that the frontend invokes via IPC. These commands follow a consistent pattern of accepting structured parameters and returning ApiResponse wrappers containing either the requested data or an error message.

### Player Commands

The `create_player` command accepts a name and optional email, returning the newly created player record. The `get_player` command retrieves a single player by ID, while `list_players` returns all registered players. Updates are handled through `update_player` which accepts the player ID and a partial update object. The `delete_player` command removes a player and cascades to remove their campaign associations.

### Character Commands

Character creation is handled by `create_character` for full character data or `create_minimal_character` for quick stubs that can be fleshed out later. The `create_npc` command provides a streamlined path for NPCs that skips player association.

Retrieval commands include `get_character` for a single character by ID and three listing commands scoped to campaigns: `list_characters_for_campaign` returns all characters, `list_pcs_for_campaign` filters to player characters only, and `list_npcs_for_campaign` returns NPCs exclusively.

The `update_character` command accepts a character ID and partial update object for modifying existing characters. The `level_up_character` command handles level advancement including hit point recalculation, proficiency bonus updates, and spell slot adjustments. Characters are linked to campaigns through `assign_character_to_campaign`.

### Spell Commands

The `get_character_spell_slots` command returns current slot status including both maximums and remaining uses. The `add_spell_to_known` command adds a spell to the character's repertoire by spell ID. Prepared spell selection is handled through `prepare_spells` which accepts a list of spell IDs from the known spells.

Casting uses the `cast_spell` command with character ID, spell ID, and the slot level being expended. The command validates slot availability before marking the slot as used. Resource recovery happens through `take_rest` which accepts the rest type (short or long) and restores appropriate slots and features.

## Frontend Integration

The frontend provides a character creation wizard that guides users through race and class selection, ability score assignment, and initial equipment choices. The character sheet view displays all statistics, abilities, and equipment in a format familiar to tabletop players. Spell management includes a preparation interface supporting drag-and-drop organization of known spells into prepared slots.

## MCP Integration

Character data is accessible through the Mimir MCP server for Claude Code integration. The MCP tools support listing characters, viewing details, creating PCs and NPCs, and managing inventory. See the [MCP Plugin README](https://github.com/mimir-dm/mimir/tree/main/crates/mimir-mcp/plugin) for setup instructions.
