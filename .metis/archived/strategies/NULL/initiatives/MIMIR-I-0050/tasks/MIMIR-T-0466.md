---
id: mcp-character-tools
level: task
title: "MCP Character Tools"
short_code: "MIMIR-T-0466"
created_at: 2026-01-28T04:06:33.296652+00:00
updated_at: 2026-01-28T04:42:49.080129+00:00
parent: MIMIR-I-0050
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0050
---

# MCP Character Tools

## Parent Initiative

[[MIMIR-I-0050]]

## Objective

Implement MCP tools for character management: listing, creating, editing characters (NPCs and PCs), managing inventory, and assigning NPCs to modules.

**Reference**: `mimir-dm-bu/mimir-dm-mcp/src/tools/` (character handlers)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `list_characters` - Returns characters in campaign with filtering
- [ ] `get_character` - Returns full character details with inventory
- [ ] `create_character` - Creates NPC or PC with basic info
- [ ] `edit_character` - Updates character fields
- [ ] `assign_npc_to_module` - Links NPC to module with role
- [ ] `add_item_to_character` - Adds item from catalog to inventory
- [ ] `update_character_currency` - Updates gold/silver/copper/platinum
- [ ] Tool schemas registered in ServerHandler

## Tools Specification

### list_characters
- **Parameters**: `character_type?: "pc" | "npc"`, `module_id?`
- **Returns**: Array of characters with basic info
- **Uses**: `CharacterService::list()` with filters

### get_character
- **Parameters**: `character_id: string`
- **Returns**: Full character with classes, inventory, proficiencies
- **Uses**: `CharacterService::get_full()`

### create_character
- **Parameters**: `name, character_type, race_name?, class_name?, level?`
- **Returns**: Created character
- **Uses**: `CharacterService::create()`

### edit_character
- **Parameters**: `character_id, fields to update...`
- **Returns**: Updated character
- **Uses**: `CharacterService::update()`

### assign_npc_to_module
- **Parameters**: `character_id, module_id, role?, location?`
- **Returns**: Updated character with module assignment
- **Uses**: `CharacterService::update()` (module_id, npc_role, npc_location)

### add_item_to_character
- **Parameters**: `character_id, item_name, quantity?, equipped?`
- **Returns**: Created inventory record
- **Uses**: Catalog lookup + `CharacterInventoryService::create()`

### update_character_currency
- **Parameters**: `character_id, copper?, silver?, gold?, platinum?`
- **Returns**: Updated character
- **Uses**: `CharacterService::update()` currency fields

## Dependencies
- Depends on: MIMIR-T-0461, MIMIR-T-0462, MIMIR-T-0463 (needs active campaign)

## Status Updates

*To be added during implementation*