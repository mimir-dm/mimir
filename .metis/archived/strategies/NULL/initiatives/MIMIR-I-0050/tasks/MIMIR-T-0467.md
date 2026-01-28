---
id: mcp-catalog-search-tools
level: task
title: "MCP Catalog Search Tools"
short_code: "MIMIR-T-0467"
created_at: 2026-01-28T04:06:34.056101+00:00
updated_at: 2026-01-28T04:44:51.916875+00:00
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

# MCP Catalog Search Tools

## Parent Initiative

[[MIMIR-I-0050]]

## Objective

Implement MCP tools for searching the 5etools catalog: monsters, items, spells, traps, classes, races, and feats. These enable Claude to look up D&D 5e content when authoring modules.

**Reference**: `mimir-dm-bu/mimir-dm-mcp/src/tools/` (catalog handlers)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `search_monsters` - Search with CR, type, name filters
- [ ] `search_items` - Search with rarity, type, name filters
- [ ] `search_spells` - Search with level, school, class filters
- [ ] `search_traps` - Search with level, name filters
- [ ] `search_classes` - List/search classes with features
- [ ] `search_races` - List/search races with traits
- [ ] `search_feats` - Search feats by name, prerequisite
- [ ] All tools respect campaign source filtering
- [ ] Tool schemas registered in ServerHandler

## Tools Specification

### search_monsters
- **Parameters**: `name?, cr_min?, cr_max?, monster_type?, source?`
- **Returns**: Array of monsters with name, CR, type, HP, AC
- **Uses**: `catalog_dal::search_monsters()` with filters

### search_items
- **Parameters**: `name?, rarity?, item_type?, source?`
- **Returns**: Array of items with name, type, rarity, value
- **Uses**: `catalog_dal::search_items()` with filters

### search_spells
- **Parameters**: `name?, level?, school?, class_name?, source?`
- **Returns**: Array of spells with name, level, school, components
- **Uses**: `catalog_dal::search_spells()` with filters

### search_traps
- **Parameters**: `name?, level?, source?`
- **Returns**: Array of traps with name, level, description
- **Uses**: `catalog_dal::search_traps()` with filters

### search_classes
- **Parameters**: `name?`
- **Returns**: Classes with hit die, proficiencies, features
- **Uses**: `catalog_dal::list_classes()` or `get_class()`

### search_races
- **Parameters**: `name?`
- **Returns**: Races with traits, ability bonuses, speed
- **Uses**: `catalog_dal::list_races()` or `get_race()`

### search_feats
- **Parameters**: `name?, prerequisite?`
- **Returns**: Feats with name, prerequisite, description
- **Uses**: `catalog_dal::search_feats()` with filters

## Source Filtering
All catalog searches should filter by campaign sources when an active campaign is set. Use `get_campaign_source_codes()` to get enabled sources.

## Dependencies
- Depends on: MIMIR-T-0461, MIMIR-T-0462
- Independent of other tool tasks (can be done in parallel)

## Status Updates

*To be added during implementation*