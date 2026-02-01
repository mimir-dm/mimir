---
id: mcp-tool-coverage-gaps-new-and
level: task
title: "MCP Tool Coverage Gaps - New and Updated Interfaces"
short_code: "MIMIR-T-0499"
created_at: 2026-01-30T19:51:18.335114+00:00
updated_at: 2026-01-31T03:04:53.574097+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# MCP Tool Coverage Gaps - New and Updated Interfaces

## Objective

Close the gap between the mimir-core service layer and the MCP tool layer. The service/DAL already supports most operations; they just need MCP tool wrappers in `crates/mimir-mcp/src/tools/`.

## Updated Tools (Existing)

### `edit_character` — Add missing fields
Currently only exposes: name, npc_role, npc_location, faction, traits, ideals, bonds, flaws.

**Add parameters:**
- `strength`, `dexterity`, `constitution`, `intelligence`, `wisdom`, `charisma` (ability scores)
- `cp`, `sp`, `ep`, `gp`, `pp` (currency)
- `player_name`, `race_name`, `race_source`, `background_name`, `background_source`

Service method: `CharacterService::update()` with `UpdateCharacterInput` — already supports all these fields.

### `list_characters` — Add location/faction filters
**Add optional parameters:** `location`, `faction`

Service methods: `list_npcs_by_location()`, `list_npcs_by_faction()`

### `add_item_to_character` — Add attuned parameter
**Add optional parameter:** `attuned` (bool)

Already supported by `AddInventoryInput`.

## New Tools — Characters

### `delete_character`
- **Params:** `character_id` (required)
- **Service:** `CharacterService::delete()`

### `level_up_character`
- **Params:** `character_id`, `class_name`, `subclass_name` (optional), `hp_method` (roll/average), `hp_roll` (if roll), `asi_choices` (ability score pairs or feat name), `new_spells`, `removed_spells`, `feature_choices`
- **Service:** `CharacterService::level_up()` with `LevelUpRequest`
- Note: This is the most complex tool. Consider breaking into sub-operations if needed.

### `remove_item_from_character`
- **Params:** `character_id`, `inventory_id` (required)
- **Service:** `CharacterService::remove_from_inventory()`

### `update_character_inventory`
- **Params:** `inventory_id` (required), `quantity`, `equipped`, `attuned` (all optional)
- **Service:** `CharacterService::update_inventory_item()`

### `get_character_inventory`
- **Params:** `character_id` (required), `filter` (optional: all/equipped/attuned)
- **Service:** `get_inventory()`, `get_equipped_items()`, `get_attuned_items()`

## New Tools — Campaigns

### `create_campaign`
- **Params:** `name` (required), `description`, `setting`, `source_books` (array)
- **Service:** `CampaignService::create()`

### `update_campaign`
- **Params:** `campaign_id` (required), `name`, `description`, `setting`
- **Service:** `CampaignService::update()`

### `delete_campaign`
- **Params:** `campaign_id` (required)
- **Service:** `CampaignService::delete()`

### `archive_campaign`
- **Params:** `campaign_id` (required)
- **Service:** Check if available; may need DAL addition

## New Tools — Modules

### `update_module`
- **Params:** `module_id` (required), `name`, `description`, `module_type`
- **Service:** `ModuleService::update()`

### `delete_module`
- **Params:** `module_id` (required)
- **Service:** `ModuleService::delete()`

## New Tools — Documents

### `delete_document`
- **Params:** `document_id` (required)
- **Service:** `DocumentService::delete()`

### `search_documents`
- **Params:** `campaign_id` or active campaign, `query` (required), `document_type` filter
- **Service:** Check DAL for search support

### `move_document`
- **Params:** `document_id` (required), `target_module_id` (optional, null = campaign-level)
- **Service:** `DocumentService::update()` changing module_id

## New Tools — Catalog (Priority Additions)

### `search_races`
- **Params:** `name`, `source`, `limit`

### `search_classes`
- **Params:** `name`, `source`, `limit`

### `search_backgrounds`
- **Params:** `name`, `source`, `limit`

### `search_feats`
- **Params:** `name`, `source`, `limit`

### `search_conditions`
- **Params:** `name`, `limit`

## New Tools — Maps

### `list_maps`
- **Params:** `module_id` (optional — without it, lists campaign-level maps)
- **Service:** `MapService::list_for_module()`, `list_campaign_level()`

### `get_map`
- **Params:** `map_id` (required)
- **Service:** `MapService::get_required()`

### `update_map`
- **Params:** `map_id` (required), `name`, `description`, `lighting_mode` (bright/dim/dark), `fog_enabled`
- **Service:** `MapService::update()` with `UpdateMapInput`

### `delete_map`
- **Params:** `map_id` (required)
- **Service:** `MapService::delete()`

Note: `create_map` requires uploading a UVTT file — may need special handling or deferral.

## New Tools — Token Placements

### `add_token_to_map`
- **Params:** `map_id` (required), `module_monster_id` or `module_npc_id` (one required), `grid_x`, `grid_y` (default 0,0), `label` (optional), `faction_color` (hex), `hidden` (bool), `vision_bright_ft`, `vision_dim_ft`, `vision_dark_ft`, `light_radius_ft`
- **DAL:** `insert_token_placement()` with `NewTokenPlacement`
- Note: Coordinates are best-effort without a vision model. Agent can place at 0,0 and user adjusts in UI.

### `list_tokens_on_map`
- **Params:** `map_id` (required), `visible_only` (optional bool)
- **DAL:** `list_token_placements()`, `list_visible_token_placements()`

### `update_token`
- **Params:** `token_id` (required), `grid_x`, `grid_y`, `label`, `faction_color`, `hidden`, vision/light fields
- **DAL:** `update_token_placement()` with `UpdateTokenPlacement`

### `remove_token`
- **Params:** `token_id` (required)
- **DAL:** `delete_token_placement()`

## Deferred — Assets

Asset management (blob storage for UVTT files, token images) requires file upload handling that may not fit the MCP stdio model cleanly. Track separately.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `edit_character` supports ability scores, currency, player_name, race, background
- [ ] `list_characters` supports location and faction filters
- [ ] `add_item_to_character` supports attuned flag
- [ ] New character tools: delete, level_up, remove_item, update_inventory, get_inventory
- [ ] New campaign tools: create, update, delete
- [ ] New module tools: update, delete
- [ ] New document tools: delete, search, move
- [ ] New catalog tools: races, classes, backgrounds, feats, conditions
- [ ] All new tools compile and are registered in the tool registry
- [ ] Existing tools continue to work unchanged

## Status Updates

*To be added during implementation*