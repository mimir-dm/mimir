---
id: mcp-campaign-tools
level: task
title: "MCP Campaign Tools"
short_code: "MIMIR-T-0463"
created_at: 2026-01-28T04:06:31.252463+00:00
updated_at: 2026-01-28T04:36:56.673892+00:00
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

# MCP Campaign Tools

## Parent Initiative

[[MIMIR-I-0050]]

## Objective

Implement MCP tools for campaign management: listing campaigns, setting active campaign, and getting campaign details/sources.

**Reference**: `mimir-dm-bu/mimir-dm-mcp/src/tools/` (campaign handlers)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `list_campaigns` - Returns all campaigns with id, name, description
- [ ] `set_active_campaign` - Sets active campaign in context, returns campaign details
- [ ] `get_campaign_details` - Returns full campaign info including modules and characters
- [ ] `get_campaign_sources` - Returns list of enabled source codes for the campaign
- [ ] Tool schemas registered in ServerHandler
- [ ] Error handling for invalid campaign IDs

## Tools Specification

### list_campaigns
- **Parameters**: None
- **Returns**: Array of `{id, name, description, created_at}`
- **Uses**: `CampaignService::list(false)`

### set_active_campaign
- **Parameters**: `campaign_id: string`
- **Returns**: Campaign details + confirmation
- **Effect**: Updates `McpContext.active_campaign_id`

### get_campaign_details
- **Parameters**: `campaign_id: string` (optional, defaults to active)
- **Returns**: Campaign + modules + characters summary
- **Uses**: `CampaignService::get()`, `ModuleService::list()`, `CharacterService::list()`

### get_campaign_sources
- **Parameters**: `campaign_id: string` (optional, defaults to active)
- **Returns**: Array of source codes
- **Uses**: `list_campaign_source_codes()`

## Dependencies
- Depends on: MIMIR-T-0461, MIMIR-T-0462
- Required for: Module and Character tools (they need active campaign)

## Status Updates

*To be added during implementation*