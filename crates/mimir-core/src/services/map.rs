//! Map Service
//!
//! Business logic for managing maps. Maps are UVTT files stored as assets
//! with metadata in the database.

use diesel::SqliteConnection;
use std::path::PathBuf;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{
    is_allowed_mime_type, CampaignAsset, LightingMode, Map, NewCampaignAsset, NewMap, UpdateMap,
};
use crate::services::{ServiceError, ServiceResult};

/// Input for creating a map from a UVTT file.
#[derive(Debug, Clone)]
pub struct CreateMapInput {
    /// Campaign this map belongs to
    pub campaign_id: String,
    /// Module this map belongs to (optional - for module-level maps)
    pub module_id: Option<String>,
    /// Display name for the map
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Initial lighting mode (defaults to Bright)
    pub lighting_mode: Option<LightingMode>,
    /// UVTT file data
    pub uvtt_data: Vec<u8>,
    /// Original filename (for reference)
    pub filename: String,
}

impl CreateMapInput {
    /// Create input for a campaign-level map.
    pub fn for_campaign(
        campaign_id: impl Into<String>,
        name: impl Into<String>,
        filename: impl Into<String>,
        uvtt_data: Vec<u8>,
    ) -> Self {
        Self {
            campaign_id: campaign_id.into(),
            module_id: None,
            name: name.into(),
            description: None,
            lighting_mode: None,
            uvtt_data,
            filename: filename.into(),
        }
    }

    /// Create input for a module-level map.
    pub fn for_module(
        campaign_id: impl Into<String>,
        module_id: impl Into<String>,
        name: impl Into<String>,
        filename: impl Into<String>,
        uvtt_data: Vec<u8>,
    ) -> Self {
        Self {
            campaign_id: campaign_id.into(),
            module_id: Some(module_id.into()),
            name: name.into(),
            description: None,
            lighting_mode: None,
            uvtt_data,
            filename: filename.into(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the lighting mode.
    pub fn with_lighting_mode(mut self, mode: LightingMode) -> Self {
        self.lighting_mode = Some(mode);
        self
    }
}

/// Input for updating a map.
#[derive(Debug, Clone, Default)]
pub struct UpdateMapInput {
    /// Update the name
    pub name: Option<String>,
    /// Update the description (Some(None) to clear)
    pub description: Option<Option<String>>,
    /// Update the sort order
    pub sort_order: Option<i32>,
    /// Update the lighting mode
    pub lighting_mode: Option<LightingMode>,
    /// Move to a module (Some(Some(id))) or to campaign level (Some(None))
    pub module_id: Option<Option<String>>,
}

impl UpdateMapInput {
    /// Update only the name.
    pub fn set_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    /// Update only the description.
    pub fn set_description(description: Option<String>) -> Self {
        Self {
            description: Some(description),
            ..Default::default()
        }
    }

    /// Update only the sort order.
    pub fn set_sort_order(sort_order: i32) -> Self {
        Self {
            sort_order: Some(sort_order),
            ..Default::default()
        }
    }

    /// Update only the lighting mode.
    pub fn set_lighting_mode(mode: LightingMode) -> Self {
        Self {
            lighting_mode: Some(mode),
            ..Default::default()
        }
    }

    /// Move map to a module.
    pub fn move_to_module(module_id: impl Into<String>) -> Self {
        Self {
            module_id: Some(Some(module_id.into())),
            ..Default::default()
        }
    }

    /// Move map to campaign level (out of any module).
    pub fn move_to_campaign() -> Self {
        Self {
            module_id: Some(None),
            ..Default::default()
        }
    }
}

/// Service for managing maps.
///
/// Handles uploading UVTT files, creating map records, and managing map metadata.
pub struct MapService<'a> {
    conn: &'a mut SqliteConnection,
    app_data_dir: PathBuf,
}

impl<'a> MapService<'a> {
    /// Create a new map service.
    pub fn new(conn: &'a mut SqliteConnection, app_data_dir: impl Into<PathBuf>) -> Self {
        Self {
            conn,
            app_data_dir: app_data_dir.into(),
        }
    }

    /// Create a new map from a UVTT file.
    ///
    /// This uploads the UVTT file as an asset and creates a map record.
    pub fn create(&mut self, input: CreateMapInput) -> ServiceResult<Map> {
        // Validate campaign exists
        if !dal::campaign_exists(self.conn, &input.campaign_id)? {
            return Err(ServiceError::not_found("Campaign", &input.campaign_id));
        }

        // Validate module exists if specified
        if let Some(ref module_id) = input.module_id {
            if !dal::module_exists(self.conn, module_id)? {
                return Err(ServiceError::not_found("Module", module_id));
            }
        }

        // Upload the UVTT file as an asset
        let asset = self.upload_uvtt_asset(
            &input.campaign_id,
            input.module_id.as_deref(),
            &input.filename,
            &input.uvtt_data,
        )?;

        // Get next sort order
        let sort_order = if let Some(ref module_id) = input.module_id {
            dal::get_next_module_sort_order(self.conn, module_id)?
        } else {
            dal::get_next_campaign_sort_order(self.conn, &input.campaign_id)?
        };

        // Create the map record
        let map_id = Uuid::new_v4().to_string();
        let lighting_mode = input.lighting_mode.unwrap_or_default();
        let description = input.description.as_deref();

        let mut new_map = if let Some(ref module_id) = input.module_id {
            NewMap::for_module(&map_id, &input.campaign_id, module_id, &input.name, &asset.id)
        } else {
            NewMap::for_campaign(&map_id, &input.campaign_id, &input.name, &asset.id)
        };

        new_map = new_map
            .with_sort_order(sort_order)
            .with_lighting_mode(lighting_mode);

        if let Some(desc) = description {
            new_map = new_map.with_description(desc);
        }

        dal::insert_map(self.conn, &new_map)?;
        dal::get_map(self.conn, &map_id).map_err(ServiceError::from)
    }

    /// Helper to upload a UVTT file as an asset.
    fn upload_uvtt_asset(
        &mut self,
        campaign_id: &str,
        module_id: Option<&str>,
        filename: &str,
        data: &[u8],
    ) -> ServiceResult<CampaignAsset> {
        // UVTT files are application/octet-stream
        let mime_type = "application/octet-stream";

        if !is_allowed_mime_type(mime_type) {
            return Err(ServiceError::validation(format!(
                "Unsupported MIME type: {}",
                mime_type
            )));
        }

        let asset_id = Uuid::new_v4().to_string();
        let relative_path = format!("assets/{}.uvtt", asset_id);
        let full_path = self.app_data_dir.join(&relative_path);

        // Ensure directory exists
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write file to disk
        std::fs::write(&full_path, data)?;

        // Create database record
        let file_size = data.len() as i32;

        let new_asset = if let Some(mod_id) = module_id {
            NewCampaignAsset::for_module(&asset_id, mod_id, filename, mime_type, &relative_path)
                .with_file_size(file_size)
        } else {
            NewCampaignAsset::for_campaign(
                &asset_id,
                campaign_id,
                filename,
                mime_type,
                &relative_path,
            )
            .with_file_size(file_size)
        };

        dal::insert_campaign_asset(self.conn, &new_asset)?;
        dal::get_campaign_asset(self.conn, &asset_id).map_err(ServiceError::from)
    }

    /// Get a map by ID.
    pub fn get(&mut self, id: &str) -> ServiceResult<Option<Map>> {
        dal::get_map_optional(self.conn, id).map_err(ServiceError::from)
    }

    /// Get a map by ID, returning an error if not found.
    pub fn get_required(&mut self, id: &str) -> ServiceResult<Map> {
        dal::get_map_optional(self.conn, id)?.ok_or_else(|| ServiceError::not_found("Map", id))
    }

    /// List all maps for a campaign (including module maps).
    pub fn list_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<Vec<Map>> {
        dal::list_campaign_maps(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// List only campaign-level maps (not in any module).
    pub fn list_campaign_level(&mut self, campaign_id: &str) -> ServiceResult<Vec<Map>> {
        dal::list_campaign_level_maps(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// List all maps for a module.
    pub fn list_for_module(&mut self, module_id: &str) -> ServiceResult<Vec<Map>> {
        dal::list_module_maps(self.conn, module_id).map_err(ServiceError::from)
    }

    /// Update a map.
    pub fn update(&mut self, id: &str, input: UpdateMapInput) -> ServiceResult<Map> {
        // Verify map exists
        if !dal::map_exists(self.conn, id)? {
            return Err(ServiceError::not_found("Map", id));
        }

        // Validate module if moving to one
        if let Some(Some(ref module_id)) = input.module_id {
            if !dal::module_exists(self.conn, module_id)? {
                return Err(ServiceError::not_found("Module", module_id));
            }
        }

        let now = chrono::Utc::now().to_rfc3339();

        // Build update struct
        let name_ref = input.name.as_deref();
        let description_ref = input.description.as_ref().map(|d| d.as_deref());
        let lighting_mode_ref = input.lighting_mode.as_ref().map(|m| m.as_str());
        let module_id_ref = input.module_id.as_ref().map(|m| m.as_deref());

        let update = UpdateMap {
            name: name_ref,
            description: description_ref,
            sort_order: input.sort_order,
            lighting_mode: lighting_mode_ref,
            fog_enabled: None,
            module_id: module_id_ref,
            updated_at: Some(&now),
        };

        dal::update_map(self.conn, id, &update)?;
        self.get_required(id)
    }

    /// Delete a map and its associated UVTT asset.
    pub fn delete(&mut self, id: &str) -> ServiceResult<()> {
        // Get the map to find the asset
        let map = self.get_required(id)?;
        let asset_id = map.uvtt_asset_id.clone();

        // Delete the map record first (removes FK constraint to asset)
        dal::delete_map(self.conn, id)?;

        // Now delete the UVTT asset file from disk and database
        if let Ok(Some(asset)) = dal::get_campaign_asset_optional(self.conn, &asset_id) {
            let file_path = self.app_data_dir.join(&asset.blob_path);
            let _ = std::fs::remove_file(&file_path);
            // Delete asset from database
            let _ = dal::delete_campaign_asset(self.conn, &asset.id);
        }

        Ok(())
    }

    /// Check if a map exists.
    pub fn exists(&mut self, id: &str) -> ServiceResult<bool> {
        dal::map_exists(self.conn, id).map_err(ServiceError::from)
    }

    /// Count maps for a campaign.
    pub fn count_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<i64> {
        dal::count_campaign_maps(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Count maps for a module.
    pub fn count_for_module(&mut self, module_id: &str) -> ServiceResult<i64> {
        dal::count_module_maps(self.conn, module_id).map_err(ServiceError::from)
    }

    /// Get the UVTT asset for a map.
    pub fn get_uvtt_asset(&mut self, map: &Map) -> ServiceResult<Option<CampaignAsset>> {
        dal::get_campaign_asset_optional(self.conn, &map.uvtt_asset_id).map_err(ServiceError::from)
    }

    /// Read the UVTT file data for a map.
    pub fn read_uvtt_file(&mut self, map: &Map) -> ServiceResult<Vec<u8>> {
        let asset = dal::get_campaign_asset_optional(self.conn, &map.uvtt_asset_id)?
            .ok_or_else(|| ServiceError::not_found("Asset", &map.uvtt_asset_id))?;

        let file_path = self.app_data_dir.join(&asset.blob_path);
        std::fs::read(&file_path).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::campaign::{insert_campaign, insert_module};
    use crate::models::campaign::{NewCampaign, NewModule};
    use crate::test_utils::setup_test_db;
    use tempfile::TempDir;

    fn setup_test_env() -> (SqliteConnection, TempDir) {
        let conn = setup_test_db();
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        (conn, temp_dir)
    }

    fn create_test_campaign(conn: &mut SqliteConnection) -> String {
        let campaign_id = Uuid::new_v4().to_string();
        let campaign = NewCampaign::new(&campaign_id, "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");
        campaign_id
    }

    fn create_test_module(conn: &mut SqliteConnection, campaign_id: &str) -> String {
        let module_id = Uuid::new_v4().to_string();
        let module = NewModule::new(&module_id, campaign_id, "Test Module", 1);
        insert_module(conn, &module).expect("Failed to create module");
        module_id
    }

    fn fake_uvtt_data() -> Vec<u8> {
        // Minimal fake UVTT data (in reality this would be a zip file)
        b"fake uvtt file content".to_vec()
    }

    #[test]
    fn test_create_campaign_map() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input = CreateMapInput::for_campaign(&campaign_id, "World Map", "world.uvtt", fake_uvtt_data());

        let map = service.create(input).expect("Failed to create map");

        assert_eq!(map.name, "World Map");
        assert_eq!(map.campaign_id, campaign_id);
        assert!(map.module_id.is_none());
        assert_eq!(map.lighting_mode, "bright");
        assert_eq!(map.sort_order, 1);
    }

    #[test]
    fn test_create_module_map() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input = CreateMapInput::for_module(&campaign_id, &module_id, "Dungeon Floor 1", "dungeon.uvtt", fake_uvtt_data())
            .with_lighting_mode(LightingMode::Dark);

        let map = service.create(input).expect("Failed to create map");

        assert_eq!(map.name, "Dungeon Floor 1");
        assert_eq!(map.module_id, Some(module_id));
        assert_eq!(map.lighting_mode, "dark");
    }

    #[test]
    fn test_create_map_with_description() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input = CreateMapInput::for_campaign(&campaign_id, "Region Map", "region.uvtt", fake_uvtt_data())
            .with_description("The northern region of the kingdom");

        let map = service.create(input).expect("Failed to create map");

        assert_eq!(map.description, Some("The northern region of the kingdom".to_string()));
    }

    #[test]
    fn test_create_map_invalid_campaign() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input = CreateMapInput::for_campaign("nonexistent", "Map", "map.uvtt", fake_uvtt_data());

        let result = service.create(input);
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_create_map_invalid_module() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input = CreateMapInput::for_module(&campaign_id, "nonexistent", "Map", "map.uvtt", fake_uvtt_data());

        let result = service.create(input);
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_uvtt_file_stored_on_disk() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let uvtt_data = fake_uvtt_data();
        let input = CreateMapInput::for_campaign(&campaign_id, "Map", "test.uvtt", uvtt_data.clone());

        let map = service.create(input).expect("Failed to create map");

        // Read the file back
        let read_data = service.read_uvtt_file(&map).expect("Failed to read file");
        assert_eq!(read_data, uvtt_data);
    }

    #[test]
    fn test_get_map() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input = CreateMapInput::for_campaign(&campaign_id, "Test Map", "test.uvtt", fake_uvtt_data());
        let created = service.create(input).expect("Failed to create map");

        let retrieved = service.get(&created.id).expect("Failed to get").expect("Map not found");
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, "Test Map");
    }

    #[test]
    fn test_get_map_not_found() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let result = service.get("nonexistent").expect("Failed to query");
        assert!(result.is_none());
    }

    #[test]
    fn test_list_for_campaign() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input1 = CreateMapInput::for_campaign(&campaign_id, "World Map", "world.uvtt", fake_uvtt_data());
        let input2 = CreateMapInput::for_module(&campaign_id, &module_id, "Dungeon", "dungeon.uvtt", fake_uvtt_data());
        service.create(input1).expect("Failed to create");
        service.create(input2).expect("Failed to create");

        // All campaign maps (includes module maps)
        let all = service.list_for_campaign(&campaign_id).expect("Failed to list");
        assert_eq!(all.len(), 2);

        // Campaign-level only
        let campaign_level = service.list_campaign_level(&campaign_id).expect("Failed to list");
        assert_eq!(campaign_level.len(), 1);
        assert_eq!(campaign_level[0].name, "World Map");
    }

    #[test]
    fn test_list_for_module() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input1 = CreateMapInput::for_module(&campaign_id, &module_id, "Floor 1", "f1.uvtt", fake_uvtt_data());
        let input2 = CreateMapInput::for_module(&campaign_id, &module_id, "Floor 2", "f2.uvtt", fake_uvtt_data());
        let input3 = CreateMapInput::for_campaign(&campaign_id, "World", "world.uvtt", fake_uvtt_data());
        service.create(input1).expect("Failed to create");
        service.create(input2).expect("Failed to create");
        service.create(input3).expect("Failed to create");

        let module_maps = service.list_for_module(&module_id).expect("Failed to list");
        assert_eq!(module_maps.len(), 2);
    }

    #[test]
    fn test_update_map_name() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input = CreateMapInput::for_campaign(&campaign_id, "Old Name", "map.uvtt", fake_uvtt_data());
        let map = service.create(input).expect("Failed to create");

        let update = UpdateMapInput::set_name("New Name");
        let updated = service.update(&map.id, update).expect("Failed to update");

        assert_eq!(updated.name, "New Name");
    }

    #[test]
    fn test_update_lighting_mode() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input = CreateMapInput::for_campaign(&campaign_id, "Map", "map.uvtt", fake_uvtt_data());
        let map = service.create(input).expect("Failed to create");
        assert_eq!(map.lighting_mode, "bright");

        let update = UpdateMapInput::set_lighting_mode(LightingMode::Dark);
        let updated = service.update(&map.id, update).expect("Failed to update");

        assert_eq!(updated.lighting_mode, "dark");
    }

    #[test]
    fn test_update_move_to_module() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input = CreateMapInput::for_campaign(&campaign_id, "Region Map", "region.uvtt", fake_uvtt_data());
        let map = service.create(input).expect("Failed to create");
        assert!(map.module_id.is_none());

        let update = UpdateMapInput::move_to_module(&module_id);
        let updated = service.update(&map.id, update).expect("Failed to update");

        assert_eq!(updated.module_id, Some(module_id));
    }

    #[test]
    fn test_update_move_to_campaign() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input = CreateMapInput::for_module(&campaign_id, &module_id, "Dungeon", "dungeon.uvtt", fake_uvtt_data());
        let map = service.create(input).expect("Failed to create");
        assert!(map.module_id.is_some());

        let update = UpdateMapInput::move_to_campaign();
        let updated = service.update(&map.id, update).expect("Failed to update");

        assert!(updated.module_id.is_none());
    }

    #[test]
    fn test_update_map_not_found() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let update = UpdateMapInput::set_name("New Name");
        let result = service.update("nonexistent", update);
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_delete_map() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input = CreateMapInput::for_campaign(&campaign_id, "Delete Me", "delete.uvtt", fake_uvtt_data());
        let map = service.create(input).expect("Failed to create");
        let asset_id = map.uvtt_asset_id.clone();

        assert!(service.exists(&map.id).expect("Failed to check"));

        service.delete(&map.id).expect("Failed to delete");

        assert!(!service.exists(&map.id).expect("Failed to check"));
        // Asset should also be deleted
        let asset = dal::get_campaign_asset_optional(&mut service.conn, &asset_id).expect("Failed to query");
        assert!(asset.is_none());
    }

    #[test]
    fn test_delete_map_not_found() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let result = service.delete("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_count_maps() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        assert_eq!(service.count_for_campaign(&campaign_id).expect("Failed"), 0);

        let input1 = CreateMapInput::for_campaign(&campaign_id, "World", "world.uvtt", fake_uvtt_data());
        let input2 = CreateMapInput::for_module(&campaign_id, &module_id, "Dungeon", "dungeon.uvtt", fake_uvtt_data());
        service.create(input1).expect("Failed to create");
        service.create(input2).expect("Failed to create");

        assert_eq!(service.count_for_campaign(&campaign_id).expect("Failed"), 2);
        assert_eq!(service.count_for_module(&module_id).expect("Failed"), 1);
    }

    #[test]
    fn test_auto_increment_sort_order() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input1 = CreateMapInput::for_campaign(&campaign_id, "Map 1", "m1.uvtt", fake_uvtt_data());
        let input2 = CreateMapInput::for_campaign(&campaign_id, "Map 2", "m2.uvtt", fake_uvtt_data());
        let input3 = CreateMapInput::for_campaign(&campaign_id, "Map 3", "m3.uvtt", fake_uvtt_data());

        let map1 = service.create(input1).expect("Failed to create");
        let map2 = service.create(input2).expect("Failed to create");
        let map3 = service.create(input3).expect("Failed to create");

        assert_eq!(map1.sort_order, 1);
        assert_eq!(map2.sort_order, 2);
        assert_eq!(map3.sort_order, 3);
    }

    #[test]
    fn test_get_uvtt_asset() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = MapService::new(&mut conn, temp_dir.path());

        let input = CreateMapInput::for_campaign(&campaign_id, "Map", "test.uvtt", fake_uvtt_data());
        let map = service.create(input).expect("Failed to create");

        let asset = service.get_uvtt_asset(&map).expect("Failed to get asset").expect("Asset not found");
        assert_eq!(asset.id, map.uvtt_asset_id);
        assert_eq!(asset.filename, "test.uvtt");
    }
}
