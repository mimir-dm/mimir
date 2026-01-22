---
id: implement-asset-services
level: task
title: "Implement asset services (MapService, image upload)"
short_code: "MIMIR-T-0398"
created_at: 2026-01-21T03:02:31.021931+00:00
updated_at: 2026-01-21T15:33:22.246380+00:00
parent: MIMIR-I-0044
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0044
---

# Implement asset services (MapService, image upload)

## Parent Initiative

[[MIMIR-I-0044]] - Implement Service Layer for mimir-core

## Objective

Implement services for managing binary assets: UVTT map files and uploaded images. Assets are stored on disk in the app data directory with database records tracking their metadata and paths.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### AssetService (General)
- [x] `AssetService` struct with connection + app_data_dir
- [x] `upload()` - save file to disk, create DB record (renamed from upload_image for generality)
- [x] `get()` - get asset metadata by ID
- [x] `delete()` - remove from disk and DB
- [x] `list_for_campaign()` - list assets for a campaign
- [x] `list_for_module()` - list assets for a module

### MapService
- [x] `MapService` struct with connection + app_data_dir
- [x] `create()` - upload UVTT file, create asset and map records
- [x] `get()` / `get_required()` - get map by ID
- [x] `list_for_campaign()` / `list_campaign_level()` / `list_for_module()` - list maps
- [x] `delete()` - delete map and associated asset

### Storage
- [x] Assets stored at `{app_data_dir}/assets/{uuid}.{ext}` (unified path for all assets)
- [x] Unit tests using tempfile::TempDir for storage (13 AssetService + 20 MapService tests)

## Implementation Notes

### Files to Create

```
crates/mimir-core/src/services/
├── mod.rs              # Add asset, map module exports
├── asset.rs            # AssetService implementation
├── map.rs              # MapService implementation
```

### Directory Structure

```
{app_data_dir}/
├── images/
│   ├── {uuid}.png
│   ├── {uuid}.jpg
│   └── {uuid}.webp
└── maps/
    └── {uuid}.dd2vtt
```

### AssetService API

```rust
pub struct AssetService<'a> {
    conn: &'a mut SqliteConnection,
    app_data_dir: PathBuf,
}

pub struct UploadImage {
    pub campaign_id: i32,
    pub module_id: Option<i32>,
    pub name: String,
    pub data: Vec<u8>,
    pub content_type: String,  // "image/png", "image/jpeg", etc.
}

impl<'a> AssetService<'a> {
    pub fn new(conn: &'a mut SqliteConnection, app_data_dir: PathBuf) -> Self;
    
    /// Upload image, save to disk, create DB record
    pub fn upload_image(&mut self, input: UploadImage) -> ServiceResult<CampaignAsset>;
    
    /// Get asset by ID
    pub fn get_asset(&mut self, id: i32) -> ServiceResult<Option<CampaignAsset>>;
    
    /// Get asset file path on disk
    pub fn get_asset_path(&self, asset: &CampaignAsset) -> PathBuf;
    
    /// Delete asset (disk + DB)
    pub fn delete_asset(&mut self, id: i32) -> ServiceResult<()>;
    
    pub fn list_for_campaign(&mut self, campaign_id: i32) -> ServiceResult<Vec<CampaignAsset>>;
    pub fn list_for_module(&mut self, module_id: i32) -> ServiceResult<Vec<CampaignAsset>>;
}
```

### MapService API

```rust
pub struct MapService<'a> {
    conn: &'a mut SqliteConnection,
    app_data_dir: PathBuf,
}

pub struct UploadMap {
    pub campaign_id: i32,
    pub module_id: i32,
    pub name: String,
    pub uvtt_data: Vec<u8>,  // Raw .dd2vtt file contents
}

impl<'a> MapService<'a> {
    pub fn new(conn: &'a mut SqliteConnection, app_data_dir: PathBuf) -> Self;
    
    /// Upload UVTT map file
    pub fn upload_map(&mut self, input: UploadMap) -> ServiceResult<Map>;
    
    /// Get map with metadata
    pub fn get_map(&mut self, id: i32) -> ServiceResult<Option<Map>>;
    
    /// Get raw UVTT data for a map
    pub fn get_uvtt_data(&self, map: &Map) -> ServiceResult<Vec<u8>>;
    
    pub fn list_for_module(&mut self, module_id: i32) -> ServiceResult<Vec<Map>>;
    pub fn delete_map(&mut self, id: i32) -> ServiceResult<()>;
}
```

### Upload Flow

```rust
pub fn upload_image(&mut self, input: UploadImage) -> ServiceResult<CampaignAsset> {
    // 1. Generate UUID for filename
    let uuid = Uuid::new_v4();
    let ext = match input.content_type.as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        _ => return Err(ServiceError::Validation("Unsupported image type".into())),
    };
    
    // 2. Write to disk
    let filename = format!("{}.{}", uuid, ext);
    let path = self.app_data_dir.join("images").join(&filename);
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, &input.data)?;
    
    // 3. Create DB record
    let asset = NewCampaignAsset {
        campaign_id: input.campaign_id,
        module_id: input.module_id,
        name: input.name,
        asset_type: "image".to_string(),
        blob_path: format!("images/{}", filename),
        content_type: Some(input.content_type),
    };
    
    dal::campaign_asset::insert(self.conn, &asset)
        .map_err(ServiceError::from)
}
```

### Database Schema Reference

```rust
// campaign_assets table
pub struct CampaignAsset {
    pub id: i32,
    pub campaign_id: i32,
    pub module_id: Option<i32>,
    pub name: String,
    pub asset_type: String,      // "image", "map"
    pub blob_path: String,       // Relative path from app_data_dir
    pub content_type: Option<String>,
    pub created_at: String,
}

// maps table
pub struct Map {
    pub id: i32,
    pub module_id: i32,
    pub name: String,
    pub uvtt_asset_id: Option<i32>,  // FK to campaign_assets
    pub grid_size: Option<i32>,
    pub lighting_mode: String,
    pub created_at: String,
}
```

### Dependencies

- MIMIR-T-0390 (ServiceError type)
- Existing `dal::campaign_asset` module
- Existing `dal::map` module
- `uuid` crate for generating filenames

## Status Updates

### 2026-01-21 - Completed
- Created `AssetService` at `crates/mimir-core/src/services/asset.rs`:
  - `UploadAssetInput` with `for_campaign()` and `for_module()` constructors
  - `AssetService` with upload, get, get_file_path, read_file, delete, list_for_campaign, list_for_module, list_images_for_campaign, exists, count methods
  - MIME type validation with `is_allowed_mime_type()`
  - Files stored at `{app_data_dir}/assets/{uuid}.{ext}`
  - 13 passing tests

- Created `MapService` at `crates/mimir-core/src/services/map.rs`:
  - `CreateMapInput` with `for_campaign()` and `for_module()` constructors, lighting mode support
  - `UpdateMapInput` with set_name, set_description, set_sort_order, set_lighting_mode, move_to_module, move_to_campaign methods
  - `MapService` with create, get, get_required, list_for_campaign, list_campaign_level, list_for_module, update, delete, exists, count, get_uvtt_asset, read_uvtt_file methods
  - Auto-incrementing sort order within campaign/module
  - Campaign and module validation on create/update
  - Cascading delete of UVTT assets
  - 20 passing tests

- Updated `services/mod.rs` with exports for AssetService, UploadAssetInput, MapService, CreateMapInput, UpdateMapInput

Total: 33 new tests, all passing