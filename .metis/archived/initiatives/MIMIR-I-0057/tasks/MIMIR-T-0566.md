---
id: tauri-commands-mcp-tools-homebrew
level: task
title: "Tauri Commands & MCP Tools: Homebrew monster integration"
short_code: "MIMIR-T-0566"
created_at: 2026-03-11T14:49:23.025969+00:00
updated_at: 2026-03-11T16:24:06.309890+00:00
parent: MIMIR-I-0057
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0057
---

# Tauri Commands & MCP Tools: Homebrew monster integration

## Parent Initiative

[[MIMIR-I-0057]]

## Objective

Update Tauri commands and MCP tools to expose homebrew monster module support to the frontend and MCP clients. Users should be able to add homebrew monsters to modules and find them via search.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Tauri `add_module_monster` command accepts optional `homebrew_monster_id` parameter (alternative to `monster_name`)
- [ ] Tauri `list_module_monsters_with_data` returns homebrew monster data (name, CR, type) alongside catalog data
- [ ] MCP `add_monster_to_module` tool accepts `homebrew_monster_id` as alternative to `monster_name`
- [ ] MCP `add_monster_to_module` tool description updated to document both paths
- [ ] MCP `search_monsters` includes homebrew results (flagged with source indicator) OR new `search_homebrew_monsters` tool added
- [ ] MCP `get_module_details` returns homebrew monster info in the monsters list
- [ ] Error handling: clear message when homebrew_monster_id doesn't exist or doesn't belong to active campaign

## Implementation Notes

### Technical Approach

**Tauri commands** (`crates/mimir-lib/src/commands/module.rs`):
- `add_module_monster` — add optional `homebrew_monster_id: Option<String>` param. If set, skip catalog name/source and use homebrew path. Validate mutual exclusivity.
- `list_module_monsters_with_data` — already exists, needs to include homebrew data in response JSON

**MCP tools** (`crates/mimir-mcp/src/tools/module.rs`):
- `add_monster_to_module_tool()` — add `homebrew_monster_id` to input schema properties
- `add_monster_to_module()` — branch on homebrew_monster_id vs monster_name
- Update tool description to explain both catalog and homebrew paths

**MCP search** (`crates/mimir-mcp/src/tools/catalog.rs`):
- Option: augment `search_monsters` to also query `campaign_homebrew_monsters` and merge results with a `"source": "homebrew"` flag
- Alternative: separate `search_homebrew_monsters` tool (simpler, but less discoverable)
- Recommendation: augment existing search with opt-in `include_homebrew` param (default true when campaign is active)

### Dependencies
- MIMIR-T-0564 (migration)
- MIMIR-T-0565 (DAL/services)

### Key Files
- `crates/mimir-lib/src/commands/module.rs`
- `crates/mimir-mcp/src/tools/module.rs`
- `crates/mimir-mcp/src/tools/catalog.rs`
- `crates/mimir-mcp/src/handler.rs`

## Status Updates

### Completed
- ✅ Tauri `AddModuleMonsterRequest` updated: `monster_name`/`monster_source` now `Option`, added `homebrew_monster_id`
- ✅ Tauri `add_module_monster` handles both catalog and homebrew paths with mutual exclusivity validation
- ✅ Tauri `add_module_monster` validates homebrew monster exists before inserting
- ✅ Tauri `list_module_monsters_with_data` now returns homebrew monster data (name, CR, type, size from JSON blob)
- ✅ MCP `add_monster_to_module` tool: `monster_name` no longer required, added `homebrew_monster_id`, `display_name`, `monster_source` params
- ✅ MCP `add_monster_to_module` impl: branches on catalog vs homebrew path with validation
- ✅ MCP `get_module_details` returns `homebrew_monster_id` and `is_homebrew` flag in monster data
- ✅ MCP `search_monsters` includes homebrew results with `is_homebrew: true` and `homebrew_id` field (opt-in via `include_homebrew`, default true)
- ✅ Error handling: clear messages for missing homebrew monster ID, mutual exclusivity violations
- ✅ All crates compile, all tests pass