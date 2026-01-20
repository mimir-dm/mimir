//! Campaign archive commands for import/export functionality.
//!
//! Provides Tauri commands for exporting campaigns to portable archives
//! and importing them into new campaign instances.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use mimir_dm_core::services::{
    ArchivePreview, CampaignArchiveService, CatalogReference, ARCHIVE_EXTENSION,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use tracing::{error, info};

use super::Campaign;

/// Response for export operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    /// Path to the created archive file
    pub archive_path: String,
    /// Name of the archive file
    pub file_name: String,
}

/// Response for archive preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivePreviewResponse {
    /// Campaign name from the archive
    pub campaign_name: String,
    /// Number of content files
    pub file_count: usize,
    /// Number of asset files
    pub asset_count: usize,
    /// Catalog references in the archive
    pub catalog_references: Vec<CatalogReferenceResponse>,
    /// Mimir version that created the archive
    pub mimir_version: String,
    /// When the archive was created (ISO8601)
    pub created_at: String,
}

/// A catalog reference for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogReferenceResponse {
    /// Type of catalog item
    #[serde(rename = "type")]
    pub ref_type: String,
    /// Name of the item
    pub name: String,
    /// Source book
    pub source: String,
}

impl From<CatalogReference> for CatalogReferenceResponse {
    fn from(r: CatalogReference) -> Self {
        Self {
            ref_type: r.ref_type,
            name: r.name,
            source: r.source,
        }
    }
}

impl From<ArchivePreview> for ArchivePreviewResponse {
    fn from(preview: ArchivePreview) -> Self {
        Self {
            campaign_name: preview.campaign_name,
            file_count: preview.file_count,
            asset_count: preview.asset_count,
            catalog_references: preview
                .catalog_references
                .into_iter()
                .map(CatalogReferenceResponse::from)
                .collect(),
            mimir_version: preview.mimir_version,
            created_at: preview.created_at.to_rfc3339(),
        }
    }
}

/// Request for importing a campaign
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCampaignRequest {
    /// Path to the archive file
    pub archive_path: String,
    /// Name for the new campaign
    pub campaign_name: String,
    /// Directory where campaigns are stored
    pub campaigns_directory: String,
}

/// Export a campaign to a tar.gz archive.
///
/// Creates a portable archive containing all campaign content and assets.
/// The archive is saved to the specified output directory.
///
/// # Parameters
/// - `campaign_id` - ID of the campaign to export
/// - `output_directory` - Directory where the archive should be created
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the path to the created archive.
#[tauri::command]
pub async fn export_campaign_archive(
    campaign_id: i32,
    output_directory: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<ExportResponse>, ApiError> {
    info!(
        "Exporting campaign {} to {}",
        campaign_id, output_directory
    );

    let mut conn = state.db.get_connection()?;
    let output_path = PathBuf::from(&output_directory);
    let data_dir = &state.paths.data_dir;

    match CampaignArchiveService::export_campaign(&mut conn, campaign_id, &output_path, data_dir) {
        Ok(archive_path) => {
            let file_name = archive_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("campaign.mimir-campaign.tar.gz")
                .to_string();

            info!("Campaign exported to: {}", archive_path.display());

            Ok(ApiResponse::success(ExportResponse {
                archive_path: archive_path.to_string_lossy().to_string(),
                file_name,
            }))
        }
        Err(e) => {
            error!("Failed to export campaign: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to export campaign: {}",
                e
            )))
        }
    }
}

/// Preview an archive without importing.
///
/// Reads the archive manifest to provide information about contents
/// without actually extracting or importing the archive.
///
/// # Parameters
/// - `archive_path` - Path to the archive file
///
/// # Returns
/// `ApiResponse` containing archive preview information.
#[tauri::command]
pub async fn preview_campaign_archive(
    archive_path: String,
) -> Result<ApiResponse<ArchivePreviewResponse>, ApiError> {
    info!("Previewing archive: {}", archive_path);

    let path = PathBuf::from(&archive_path);

    match CampaignArchiveService::preview_archive(&path) {
        Ok(preview) => {
            info!(
                "Archive preview: {} ({} files, {} assets)",
                preview.campaign_name, preview.file_count, preview.asset_count
            );
            Ok(ApiResponse::success(ArchivePreviewResponse::from(preview)))
        }
        Err(e) => {
            error!("Failed to preview archive: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to preview archive: {}",
                e
            )))
        }
    }
}

/// Import a campaign from an archive.
///
/// Creates a new campaign with the specified name and populates it
/// with content from the archive, including maps, characters, module monsters,
/// and tokens with properly remapped IDs.
///
/// # Parameters
/// - `request` - Import request with archive path, campaign name, and campaigns directory
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the newly created campaign.
#[tauri::command]
pub async fn import_campaign_archive(
    request: ImportCampaignRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Campaign>, ApiError> {
    info!(
        "Importing campaign '{}' from {}",
        request.campaign_name, request.archive_path
    );

    let mut conn = state.db.get_connection()?;
    let archive_path = PathBuf::from(&request.archive_path);
    let campaigns_dir = PathBuf::from(&request.campaigns_directory);
    let data_dir = &state.paths.data_dir;

    match CampaignArchiveService::import_campaign(
        &mut conn,
        &archive_path,
        request.campaign_name.clone(),
        &campaigns_dir,
        data_dir,
    ) {
        Ok(campaign) => {
            info!(
                "Campaign imported: {} (id: {})",
                campaign.name, campaign.id
            );
            Ok(ApiResponse::success(Campaign::from(campaign)))
        }
        Err(e) => {
            error!("Failed to import campaign: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to import campaign: {}",
                e
            )))
        }
    }
}

/// Get the expected file extension for campaign archives.
///
/// Returns the standard file extension used for Mimir campaign archives.
#[tauri::command]
pub async fn get_campaign_archive_extension() -> Result<ApiResponse<String>, ApiError> {
    Ok(ApiResponse::success(ARCHIVE_EXTENSION.to_string()))
}
