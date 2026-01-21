---
id: implement-asset-services
level: task
title: "Implement asset services (MapService, image upload)"
short_code: "MIMIR-T-0398"
created_at: 2026-01-21T03:02:31.021931+00:00
updated_at: 2026-01-21T03:02:31.021931+00:00
parent: MIMIR-I-0044
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

### AssetService (General)
- [ ] `AssetService` struct with connection + app_data_dir
- [ ] `upload_image()` - save image to disk, create DB record
- [ ] `get_asset()` - get asset metadata by ID
- [ ] `delete_asset()` - remove from disk and DB
- [ ] `list_for_campaign()` - list assets for a campaign
- [ ] `list_for_module()` - list assets for a module

### MapService
- [ ] `MapService` struct extending asset functionality
- [ ] `upload_map()` - upload UVTT file, parse metadata, create records
- [ ] `get_map()` - get map with UVTT data
- [ ] `list_for_module()` - list maps for a module
- [ ] `delete_map()` - delete map and associated asset

### Storage
- [ ] Images stored at `{app_data_dir}/images/{uuid}.{ext}`
- [ ] Maps stored at `{app_data_dir}/maps/{uuid}.dd2vtt`
- [ ] Unit tests using tempdir for storage

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

*To be added during implementation*