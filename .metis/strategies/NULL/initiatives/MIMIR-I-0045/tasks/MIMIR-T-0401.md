---
id: implement-character-tauri-commands
level: task
title: "Implement Character Tauri commands"
short_code: "MIMIR-T-0401"
created_at: 2026-01-21T16:34:48.441891+00:00
updated_at: 2026-01-21T18:00:13.558687+00:00
parent: MIMIR-I-0045
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0045
---

# Implement Character Tauri commands

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0045]]

## Objective

Implement Tauri commands wrapping `mimir-core` CharacterService for PC and NPC management.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] List commands: list_pcs, list_npcs, list_campaign_characters
- [x] CRUD commands: get, create, update, delete
- [x] Inventory commands: add_item, remove_item, update_quantity
- [x] Currency command: update_currency (via update_character with currency field)
- [x] NPC-specific: list_by_location, list_by_faction

## Implementation Notes

### Commands to Implement

```rust
#[tauri::command] fn list_pcs(state, campaign_id: String) -> Result<Vec<Character>>
#[tauri::command] fn list_npcs(state, campaign_id: String) -> Result<Vec<Character>>
#[tauri::command] fn get_character(state, id: String) -> Result<Character>
#[tauri::command] fn create_character(state, input: CreateCharacterInput) -> Result<Character>
#[tauri::command] fn update_character(state, id: String, input: UpdateCharacterInput) -> Result<Character>
#[tauri::command] fn delete_character(state, id: String) -> Result<()>
#[tauri::command] fn add_inventory_item(state, character_id: String, ...) -> Result<InventoryItem>
#[tauri::command] fn update_character_currency(state, id: String, ...) -> Result<Character>
```

### Dependencies
- Blocked by: [[MIMIR-T-0399]] (Rust backend setup)

## Status Updates

### 2026-01-21: Implementation Complete

Created `crates/mimir/src/commands/character.rs` with 17 Tauri commands:

**List Commands (5)**:
- `list_characters` - All characters for a campaign
- `list_pcs` - Player characters only
- `list_npcs` - NPCs only
- `list_npcs_by_location` - NPCs filtered by location
- `list_npcs_by_faction` - NPCs filtered by faction

**CRUD Commands (5)**:
- `get_character` - Get by ID
- `create_pc` - Create player character with race/background/ability scores
- `create_npc` - Create NPC with role/location/faction
- `update_character` - Full update including currency, roleplay elements
- `delete_character` - Permanent delete

**Inventory Commands (7)**:
- `get_character_inventory` - All items
- `get_equipped_items` - Equipped items only
- `get_attuned_items` - Attuned items only
- `add_inventory_item` - Add item with quantity/equipped/attuned/notes
- `remove_inventory_item` - Remove by inventory ID
- `update_inventory_item` - Update quantity/equipped/attuned
- `count_attuned_items` - Count for D&D 5e 3-item limit

All commands registered in `main.rs`, build passes.