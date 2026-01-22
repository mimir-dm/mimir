---
id: implement-campaign-and-module
level: task
title: "Implement Campaign and Module Tauri commands"
short_code: "MIMIR-T-0400"
created_at: 2026-01-21T16:34:48.214810+00:00
updated_at: 2026-01-21T17:55:34.363765+00:00
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

# Implement Campaign and Module Tauri commands

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0045]]

## Objective

Implement Tauri commands that wrap `mimir-core` CampaignService and ModuleService. These are the foundational commands needed before frontend can function.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Campaign commands: list, get, create, update, archive, delete
- [x] Module commands: list, get, create, update, delete
- [x] All commands use AppState.db connection
- [x] Commands return types compatible with existing frontend (ApiResponse<T> wrapper)
- [x] Error handling returns user-friendly messages

## Implementation Notes

### Commands to Implement

```rust
// Campaign
#[tauri::command] fn list_campaigns(state, include_archived: bool) -> Result<Vec<Campaign>>
#[tauri::command] fn get_campaign(state, id: String) -> Result<Campaign>
#[tauri::command] fn create_campaign(state, name: String, description: Option<String>) -> Result<Campaign>
#[tauri::command] fn update_campaign(state, id: String, ...) -> Result<Campaign>
#[tauri::command] fn archive_campaign(state, id: String) -> Result<()>
#[tauri::command] fn delete_campaign(state, id: String) -> Result<()>

// Module  
#[tauri::command] fn list_modules(state, campaign_id: String) -> Result<Vec<Module>>
#[tauri::command] fn get_module(state, id: String) -> Result<Module>
#[tauri::command] fn create_module(state, campaign_id: String, name: String, ...) -> Result<Module>
#[tauri::command] fn update_module(state, id: String, ...) -> Result<Module>
#[tauri::command] fn delete_module(state, id: String) -> Result<()>
```

### Dependencies
- Blocked by: [[MIMIR-T-0399]] (Rust backend setup)

## Status Updates

### 2026-01-21

**Commands Module Created:**
- `src/commands/mod.rs` - ApiResponse<T> wrapper for frontend compatibility
- `src/commands/campaign.rs` - 8 campaign commands
- `src/commands/module.rs` - 6 module commands

**Campaign Commands Implemented:**
- `list_campaigns(include_archived)` - List active or all campaigns
- `list_archived_campaigns()` - List only archived campaigns
- `get_campaign(id)` - Get single campaign
- `create_campaign(request)` - Create with name/description
- `update_campaign(id, request)` - Update name/description
- `archive_campaign(campaign_id)` - Soft delete
- `unarchive_campaign(campaign_id)` - Restore from archive
- `delete_campaign(request)` - Permanent delete

**Module Commands Implemented:**
- `list_modules(campaign_id)` - List all modules for campaign
- `get_module(id)` - Get single module
- `get_module_by_number(campaign_id, number)` - Get by position
- `create_module(request)` - Create with type-specific template
- `update_module(id, request)` - Update name/description
- `delete_module(id)` - Permanent delete

**Frontend Type Note:**
Frontend `types/api.ts` expects numeric IDs but mimir-core uses String UUIDs.
Frontend types will need updating in T-0405 (Pinia store migration).