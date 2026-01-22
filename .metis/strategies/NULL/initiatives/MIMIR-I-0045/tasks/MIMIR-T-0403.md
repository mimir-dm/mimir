---
id: implement-map-and-asset-tauri
level: task
title: "Implement Map and Asset Tauri commands"
short_code: "MIMIR-T-0403"
created_at: 2026-01-21T16:34:48.859476+00:00
updated_at: 2026-01-21T18:14:28.565431+00:00
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

# Implement Map and Asset Tauri commands

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0045]]

## Objective

Implement Tauri commands wrapping `mimir-core` MapService and AssetService for map and binary asset management.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Map CRUD commands: list, get, create, update, delete
- [x] Asset commands: upload, get, delete, list
- [x] Read UVTT file data for map viewer
- [x] Handle binary file uploads from frontend (via base64 encoding)

## Implementation Notes

### Map Commands

```rust
#[tauri::command] fn list_maps(state, campaign_id: String, module_id: Option<String>) -> Result<Vec<Map>>
#[tauri::command] fn get_map(state, id: String) -> Result<Map>
#[tauri::command] fn create_map(state, input: CreateMapInput, uvtt_data: Vec<u8>) -> Result<Map>
#[tauri::command] fn update_map(state, id: String, input: UpdateMapInput) -> Result<Map>
#[tauri::command] fn delete_map(state, id: String) -> Result<()>
#[tauri::command] fn read_map_uvtt(state, map_id: String) -> Result<Vec<u8>>
```

### Asset Commands

```rust
#[tauri::command] fn upload_asset(state, input: UploadAssetInput, data: Vec<u8>) -> Result<Asset>
#[tauri::command] fn get_asset(state, id: String) -> Result<Asset>
#[tauri::command] fn delete_asset(state, id: String) -> Result<()>
#[tauri::command] fn list_assets(state, campaign_id: Option<String>, module_id: Option<String>) -> Result<Vec<Asset>>
```

### Key Notes
- Maps use UVTT format internally (per ADR-0007)
- Binary assets stored on filesystem, metadata in DB
- Asset deletion cascades to map deletion

### Dependencies
- Blocked by: [[MIMIR-T-0399]] (Rust backend setup)

## Status Updates

### 2026-01-21: Implementation Complete

Created two command modules:

**Map Commands (`crates/mimir/src/commands/map.rs`) - 8 commands:**
- `list_campaign_maps` - All maps for a campaign (including module maps)
- `list_campaign_level_maps` - Campaign-level maps only
- `list_module_maps` - Maps for a specific module
- `get_map` - Get map by ID
- `create_map` - Upload UVTT file and create map record
- `update_map` - Update name, description, sort_order, lighting_mode
- `delete_map` - Delete map and associated UVTT asset
- `read_map_uvtt` - Read UVTT file data (returns base64)

**Asset Commands (`crates/mimir/src/commands/asset.rs`) - 7 commands:**
- `list_campaign_assets` - All assets for a campaign
- `list_module_assets` - All assets for a module
- `list_campaign_images` - Image assets only for a campaign
- `get_asset` - Get asset metadata by ID
- `upload_asset` - Upload file (base64 input)
- `delete_asset` - Delete asset from disk and database
- `read_asset_file` - Read file data (returns base64)

**Notes:**
- Binary data handled via base64 encoding for JSON transport
- Added `base64 = "0.22"` dependency
- LightingMode options: bright (default), dim, dark

All commands registered in `main.rs`, build passes.