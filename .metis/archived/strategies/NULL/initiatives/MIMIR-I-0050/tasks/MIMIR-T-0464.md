---
id: mcp-module-tools
level: task
title: "MCP Module Tools"
short_code: "MIMIR-T-0464"
created_at: 2026-01-28T04:06:31.893418+00:00
updated_at: 2026-01-28T04:38:31.853660+00:00
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

# MCP Module Tools

## Parent Initiative

[[MIMIR-I-0050]]

## Objective

Implement MCP tools for module management: creating modules, listing modules, viewing details, and managing module monsters/items.

**Reference**: `mimir-dm-bu/mimir-dm-mcp/src/tools/` (module handlers)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `create_module` - Creates new module with name, description, module_type
- [ ] `list_modules` - Returns all modules in active campaign
- [ ] `get_module_details` - Returns module with documents, monsters, items, maps
- [ ] `add_monster_to_module` - Adds monster from catalog with count and notes
- [ ] `update_module_monster` - Updates count, notes for existing module monster
- [ ] `add_item_to_module` - Adds item from catalog as module loot
- [ ] Tool schemas registered in ServerHandler
- [ ] Requires active campaign

## Tools Specification

### create_module
- **Parameters**: `name, description?, module_type? (adventure|location|organization)`
- **Returns**: Created module
- **Uses**: `ModuleService::create()`

### list_modules
- **Parameters**: None (uses active campaign)
- **Returns**: Array of modules with id, name, type, summary
- **Uses**: `ModuleService::list(campaign_id)`

### get_module_details
- **Parameters**: `module_id: string`
- **Returns**: Full module with documents, monsters, items, maps
- **Uses**: `ModuleService::get_with_relations()`

### add_monster_to_module
- **Parameters**: `module_id, monster_name, count?, notes?`
- **Returns**: Created module_monster record
- **Uses**: Catalog lookup + `ModuleMonsterService::create()`

### update_module_monster
- **Parameters**: `module_monster_id, count?, notes?`
- **Returns**: Updated record
- **Uses**: `ModuleMonsterService::update()`

### add_item_to_module
- **Parameters**: `module_id, item_name, quantity?, notes?`
- **Returns**: Created module_item record
- **Uses**: Catalog lookup + `ModuleItemService::create()`

## Dependencies
- Depends on: MIMIR-T-0461, MIMIR-T-0462, MIMIR-T-0463 (needs active campaign)

## Status Updates

*To be added during implementation*