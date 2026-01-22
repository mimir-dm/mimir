//! Asset Service
//!
//! Business logic for managing binary assets (images, files).
//! Assets are stored on disk with metadata in the database.

use diesel::SqliteConnection;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{
    extension_for_mime_type, is_allowed_mime_type, CampaignAsset, NewCampaignAsset,
};
use crate::services::{ServiceError, ServiceResult};

/// Input for uploading an image asset.
#[derive(Debug, Clone)]
pub struct UploadAssetInput {
    /// Campaign this asset belongs to (for campaign-level assets)
    pub campaign_id: Option<String>,
    /// Module this asset belongs to (for module-level assets)
    pub module_id: Option<String>,
    /// Original filename
    pub filename: String,
    /// MIME type (e.g., "image/png")
    pub mime_type: String,
    /// File data
    pub data: Vec<u8>,
}

impl UploadAssetInput {
    /// Create input for a campaign-level asset.
    pub fn for_campaign(
        campaign_id: impl Into<String>,
        filename: impl Into<String>,
        mime_type: impl Into<String>,
        data: Vec<u8>,
    ) -> Self {
        Self {
            campaign_id: Some(campaign_id.into()),
            module_id: None,
            filename: filename.into(),
            mime_type: mime_type.into(),
            data,
        }
    }

    /// Create input for a module-level asset.
    pub fn for_module(
        module_id: impl Into<String>,
        filename: impl Into<String>,
        mime_type: impl Into<String>,
        data: Vec<u8>,
    ) -> Self {
        Self {
            campaign_id: None,
            module_id: Some(module_id.into()),
            filename: filename.into(),
            mime_type: mime_type.into(),
            data,
        }
    }
}

/// Service for managing binary assets.
///
/// Handles uploading, retrieving, and deleting assets stored on disk.
pub struct AssetService<'a> {
    conn: &'a mut SqliteConnection,
    app_data_dir: PathBuf,
}

impl<'a> AssetService<'a> {
    /// Create a new asset service.
    pub fn new(conn: &'a mut SqliteConnection, app_data_dir: impl Into<PathBuf>) -> Self {
        Self {
            conn,
            app_data_dir: app_data_dir.into(),
        }
    }

    /// Upload an asset (image or other file).
    ///
    /// Saves the file to disk and creates a database record.
    pub fn upload(&mut self, input: UploadAssetInput) -> ServiceResult<CampaignAsset> {
        // Validate MIME type
        if !is_allowed_mime_type(&input.mime_type) {
            return Err(ServiceError::validation(format!(
                "Unsupported MIME type: {}",
                input.mime_type
            )));
        }

        // Validate that either campaign_id or module_id is set
        if input.campaign_id.is_none() && input.module_id.is_none() {
            return Err(ServiceError::validation(
                "Asset must belong to either a campaign or a module",
            ));
        }

        // Generate UUID and determine file extension
        let asset_id = Uuid::new_v4().to_string();
        let ext = extension_for_mime_type(&input.mime_type).unwrap_or("bin");

        // Determine storage path
        let relative_path = format!("assets/{}.{}", asset_id, ext);
        let full_path = self.app_data_dir.join(&relative_path);

        // Ensure directory exists
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write file to disk
        fs::write(&full_path, &input.data)?;

        // Create database record
        let file_size = input.data.len() as i32;
        let campaign_id_ref = input.campaign_id.as_deref();
        let module_id_ref = input.module_id.as_deref();

        let new_asset = if let Some(campaign_id) = campaign_id_ref {
            NewCampaignAsset::for_campaign(
                &asset_id,
                campaign_id,
                &input.filename,
                &input.mime_type,
                &relative_path,
            )
            .with_file_size(file_size)
        } else if let Some(module_id) = module_id_ref {
            NewCampaignAsset::for_module(
                &asset_id,
                module_id,
                &input.filename,
                &input.mime_type,
                &relative_path,
            )
            .with_file_size(file_size)
        } else {
            unreachable!("Validated above")
        };

        dal::insert_campaign_asset(self.conn, &new_asset)?;
        dal::get_campaign_asset(self.conn, &asset_id).map_err(ServiceError::from)
    }

    /// Get an asset by ID.
    pub fn get(&mut self, id: &str) -> ServiceResult<Option<CampaignAsset>> {
        dal::get_campaign_asset_optional(self.conn, id).map_err(ServiceError::from)
    }

    /// Get the full file path for an asset.
    pub fn get_file_path(&self, asset: &CampaignAsset) -> PathBuf {
        self.app_data_dir.join(&asset.blob_path)
    }

    /// Read the file data for an asset.
    pub fn read_file(&self, asset: &CampaignAsset) -> ServiceResult<Vec<u8>> {
        let path = self.get_file_path(asset);
        fs::read(&path).map_err(ServiceError::from)
    }

    /// Delete an asset (removes from disk and database).
    pub fn delete(&mut self, id: &str) -> ServiceResult<()> {
        // Get the asset to find the file path
        let asset = dal::get_campaign_asset_optional(self.conn, id)?;

        if let Some(asset) = asset {
            // Delete from disk (ignore errors if file doesn't exist)
            let file_path = self.get_file_path(&asset);
            let _ = fs::remove_file(&file_path);

            // Delete from database
            dal::delete_campaign_asset(self.conn, id)?;
            Ok(())
        } else {
            Err(ServiceError::not_found("Asset", id))
        }
    }

    /// List assets for a campaign.
    pub fn list_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<Vec<CampaignAsset>> {
        dal::list_campaign_assets(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// List assets for a module.
    pub fn list_for_module(&mut self, module_id: &str) -> ServiceResult<Vec<CampaignAsset>> {
        dal::list_module_assets(self.conn, module_id).map_err(ServiceError::from)
    }

    /// List image assets for a campaign.
    pub fn list_images_for_campaign(
        &mut self,
        campaign_id: &str,
    ) -> ServiceResult<Vec<CampaignAsset>> {
        // Get all assets and filter to images
        let assets = dal::list_campaign_assets(self.conn, campaign_id)?;
        Ok(assets.into_iter().filter(|a| a.is_image()).collect())
    }

    /// Check if an asset exists.
    pub fn exists(&mut self, id: &str) -> ServiceResult<bool> {
        dal::campaign_asset_exists(self.conn, id).map_err(ServiceError::from)
    }

    /// Count assets for a campaign.
    pub fn count_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<i64> {
        dal::count_campaign_assets(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Count assets for a module.
    pub fn count_for_module(&mut self, module_id: &str) -> ServiceResult<i64> {
        dal::count_module_assets(self.conn, module_id).map_err(ServiceError::from)
    }

    /// Get the app data directory.
    pub fn app_data_dir(&self) -> &Path {
        &self.app_data_dir
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

    #[test]
    fn test_upload_campaign_asset() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        let data = b"fake image data".to_vec();
        let input = UploadAssetInput::for_campaign(&campaign_id, "test.png", "image/png", data);

        let asset = service.upload(input).expect("Failed to upload");

        assert_eq!(asset.filename, "test.png");
        assert_eq!(asset.mime_type, "image/png");
        assert_eq!(asset.campaign_id, Some(campaign_id));
        assert!(asset.module_id.is_none());
        assert_eq!(asset.file_size, Some(15)); // "fake image data".len()
    }

    #[test]
    fn test_upload_module_asset() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        let data = b"module asset data".to_vec();
        let input = UploadAssetInput::for_module(&module_id, "map.png", "image/png", data);

        let asset = service.upload(input).expect("Failed to upload");

        assert!(asset.campaign_id.is_none());
        assert_eq!(asset.module_id, Some(module_id));
    }

    #[test]
    fn test_upload_invalid_mime_type() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        let input =
            UploadAssetInput::for_campaign(&campaign_id, "test.exe", "application/exe", vec![]);

        let result = service.upload(input);
        assert!(matches!(result, Err(ServiceError::Validation(_))));
    }

    #[test]
    fn test_upload_no_owner() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        let input = UploadAssetInput {
            campaign_id: None,
            module_id: None,
            filename: "orphan.png".to_string(),
            mime_type: "image/png".to_string(),
            data: vec![],
        };

        let result = service.upload(input);
        assert!(matches!(result, Err(ServiceError::Validation(_))));
    }

    #[test]
    fn test_file_written_to_disk() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        let data = b"test file content".to_vec();
        let input = UploadAssetInput::for_campaign(&campaign_id, "test.png", "image/png", data.clone());

        let asset = service.upload(input).expect("Failed to upload");

        // Verify file exists and has correct content
        let file_path = service.get_file_path(&asset);
        assert!(file_path.exists());

        let read_data = fs::read(&file_path).expect("Failed to read file");
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_read_file() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        let data = b"content to read".to_vec();
        let input = UploadAssetInput::for_campaign(&campaign_id, "test.png", "image/png", data.clone());

        let asset = service.upload(input).expect("Failed to upload");
        let read_data = service.read_file(&asset).expect("Failed to read file");

        assert_eq!(read_data, data);
    }

    #[test]
    fn test_get_asset() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        let input = UploadAssetInput::for_campaign(&campaign_id, "test.png", "image/png", vec![1, 2, 3]);
        let uploaded = service.upload(input).expect("Failed to upload");

        let retrieved = service
            .get(&uploaded.id)
            .expect("Failed to get")
            .expect("Asset not found");

        assert_eq!(retrieved.id, uploaded.id);
        assert_eq!(retrieved.filename, "test.png");
    }

    #[test]
    fn test_get_asset_not_found() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        let result = service.get("nonexistent").expect("Failed to query");
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_asset() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        let input = UploadAssetInput::for_campaign(&campaign_id, "delete_me.png", "image/png", vec![1, 2, 3]);
        let asset = service.upload(input).expect("Failed to upload");
        let file_path = service.get_file_path(&asset);

        assert!(file_path.exists());
        assert!(service.exists(&asset.id).expect("Failed to check"));

        service.delete(&asset.id).expect("Failed to delete");

        assert!(!file_path.exists());
        assert!(!service.exists(&asset.id).expect("Failed to check"));
    }

    #[test]
    fn test_delete_asset_not_found() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        let result = service.delete("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_list_for_campaign() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        let input1 = UploadAssetInput::for_campaign(&campaign_id, "a.png", "image/png", vec![]);
        let input2 = UploadAssetInput::for_campaign(&campaign_id, "b.jpg", "image/jpeg", vec![]);
        service.upload(input1).expect("Failed to upload");
        service.upload(input2).expect("Failed to upload");

        let assets = service
            .list_for_campaign(&campaign_id)
            .expect("Failed to list");
        assert_eq!(assets.len(), 2);
    }

    #[test]
    fn test_list_for_module() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        let input1 = UploadAssetInput::for_module(&module_id, "map1.png", "image/png", vec![]);
        let input2 = UploadAssetInput::for_module(&module_id, "map2.png", "image/png", vec![]);
        let input3 = UploadAssetInput::for_campaign(&campaign_id, "other.png", "image/png", vec![]);
        service.upload(input1).expect("Failed to upload");
        service.upload(input2).expect("Failed to upload");
        service.upload(input3).expect("Failed to upload");

        let assets = service.list_for_module(&module_id).expect("Failed to list");
        assert_eq!(assets.len(), 2);
    }

    #[test]
    fn test_count_assets() {
        let (mut conn, temp_dir) = setup_test_env();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = AssetService::new(&mut conn, temp_dir.path());

        assert_eq!(
            service.count_for_campaign(&campaign_id).expect("Failed to count"),
            0
        );

        let input1 = UploadAssetInput::for_campaign(&campaign_id, "a.png", "image/png", vec![]);
        let input2 = UploadAssetInput::for_module(&module_id, "b.png", "image/png", vec![]);
        service.upload(input1).expect("Failed to upload");
        service.upload(input2).expect("Failed to upload");

        assert_eq!(
            service.count_for_campaign(&campaign_id).expect("Failed to count"),
            1
        );
        assert_eq!(
            service.count_for_module(&module_id).expect("Failed to count"),
            1
        );
    }
}
