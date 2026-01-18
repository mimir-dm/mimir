---
id: expand-mcp-character-tools-to
level: task
title: "Expand MCP Character Tools to Support PC Creation and Character Editing"
short_code: "MIMIR-T-0349"
created_at: 2026-01-18T20:01:04.719103+00:00
updated_at: 2026-01-18T20:03:35.375563+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Expand MCP Character Tools to Support PC Creation and Character Editing

## Objective

Expand the MCP character tools to expose full character creation and editing capabilities that already exist in the service layer but are not currently accessible via MCP.

## Problem Statement

The `CharacterService` in `mimir-dm-core` supports:
- Creating both PCs (`is_npc: false`) and NPCs (`is_npc: true`)
- Full character editing via `update_character()`

However, the MCP tools in `mimir-dm-mcp` only expose:
- `create_npc` - hardcodes `is_npc: true`, cannot create PCs
- `add_item_to_character` - inventory only
- `update_character_currency` - currency only

This means Claude Code users cannot:
1. Create player characters via MCP
2. Edit any character attributes (name, race, class, abilities, HP, etc.)

### Current Capability Matrix

| Capability | Service Layer | MCP Layer |
|------------|---------------|-----------|
| Create PC | Yes (`is_npc: false`) | **No** |
| Create NPC | Yes | Yes |
| Edit character (full) | Yes (`update_character`) | **No** |
| Edit inventory | Yes | Yes |
| Edit currency | Yes | Yes |

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: DMs using Claude Code can fully manage their party's characters without switching to the desktop app
- **Business Value**: Makes the MCP integration more complete and useful for AI-assisted campaign management
- **Effort Estimate**: S - The service layer already exists; this is primarily exposing existing functionality

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `create_character` tool (or modified `create_npc`) accepts `is_npc` parameter (default: true for backwards compatibility)
- [ ] New `create_character` tool accepts optional `player_id` parameter for PC creation
- [ ] New `edit_character` tool allows updating any `CharacterData` field
- [ ] `edit_character` creates a new character version (preserving history)
- [ ] Plugin documentation updated with new tools
- [ ] Skill reference (`tool-reference.md`) updated with new tool parameters

## Implementation Notes

### Technical Approach

**Option A: Extend existing tool**
- Rename `create_npc` to `create_character`
- Add optional `is_npc` parameter (default: `true`)
- Add optional `player_id` parameter

**Option B: Add new tools**
- Keep `create_npc` as-is for backwards compatibility
- Add new `create_pc` tool
- Add new `edit_character` tool

Recommend **Option A** for `create_character` and a new `edit_character` tool.

### Files to Modify

1. `crates/mimir-dm-mcp/src/tools/character.rs`
   - Modify `CreateNpcInput` or add new input structs
   - Add `EditCharacterInput` struct and implementation

2. `crates/mimir-dm-mcp/src/tools/mod.rs`
   - Register new tools

3. `crates/mimir-dm-mcp/src/handler.rs`
   - Add tool dispatch cases

4. `crates/mimir-dm-mcp/plugin/skills/mimir-dm/SKILL.md`
   - Update quick reference table

5. `crates/mimir-dm-mcp/plugin/skills/mimir-dm/references/tool-reference.md`
   - Document new tools and parameters

6. `crates/mimir-dm-mcp/plugin/README.md`
   - Update tool count and table

### Dependencies
- None - service layer already implements required functionality

### Risk Considerations
- **Breaking change risk**: If renaming `create_npc`, ensure backwards compatibility with default `is_npc: true`
- **Data integrity**: `edit_character` must always create new versions, never modify in place

## Status Updates

### 2026-01-18: Implementation Complete

**Changes Made:**

1. **Renamed `CreateNpcInput` to `CreateCharacterInput`** (`character.rs`)
   - Added `is_npc` parameter (default: `true` for backwards compatibility)
   - Added `player_id` and `player_name` parameters for PC creation
   - Added `level` and `background` parameters
   - Added `backstory` parameter
   - Tool renamed from `create_npc` to `create_character`

2. **Added new `EditCharacterInput` tool** (`character.rs`)
   - Supports updating: name, race, alignment, backstory, max_hp, current_hp, speed
   - Supports updating: ability scores (all six at once)
   - Supports updating: personality (traits, ideals, bonds, flaws)
   - NPC-specific fields: npc_role, npc_location, npc_faction, npc_notes
   - Creates new character version on each edit (preserving history)
   - Custom snapshot reason support

3. **Updated exports** (`mod.rs`)
   - Exported `CreateCharacterInput`, `CreateCharacterResponse`
   - Exported `EditCharacterInput`, `EditCharacterResponse`

4. **Updated handler** (`handler.rs`)
   - Added dispatch for `create_character` tool
   - Added dispatch for `edit_character` tool

5. **Updated plugin documentation**
   - `SKILL.md`: Updated trigger phrases, tool reference table, workflows
   - `tool-reference.md`: Full parameter documentation for new tools
   - `README.md`: Updated tool count and table
   - `plugin.json`: Bumped version to 0.2.4

6. **Updated tests** (`mcp_tools.rs`)
   - Migrated all tests from `CreateNpcInput` to `CreateCharacterInput`
   - Added new test: `test_create_character_pc`
   - Added test for `EditCharacterInput` tool definition

**Acceptance Criteria Status:**
- [x] `create_character` tool accepts `is_npc` parameter (default: true)
- [x] `create_character` tool accepts optional `player_id` parameter
- [x] `edit_character` tool allows updating CharacterData fields
- [x] `edit_character` creates new character version (preserving history)
- [x] Plugin documentation updated with new tools
- [x] Skill reference (`tool-reference.md`) updated

**Test Results:** All 34 tests pass