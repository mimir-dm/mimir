//! Archive Commands
//!
//! Tauri commands for campaign export/import operations.

use mimir_core::services::{ArchivePreview, ArchiveService, ImportResult};
use serde::Serialize;
use std::path::Path;
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

/// Result of exporting a campaign.
#[derive(Debug, Serialize)]
pub struct ExportResult {
    /// Path to the created archive file.
    pub archive_path: String,
    /// Size of the archive in bytes.
    pub size_bytes: u64,
}

/// Export a campaign to an archive file.
///
/// Creates a `.mimir-campaign.tar.gz` archive containing all campaign data
/// and assets, suitable for sharing with other DMs.
#[tauri::command]
pub fn export_campaign(
    state: State<'_, AppState>,
    campaign_id: String,
    output_dir: String,
) -> ApiResponse<ExportResult> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let output_path = Path::new(&output_dir);
    let assets_path = &state.paths.assets_dir;

    match ArchiveService::new(&mut db).export_campaign(&campaign_id, output_path, assets_path) {
        Ok(archive_path) => {
            // Get file size
            let size_bytes = std::fs::metadata(&archive_path)
                .map(|m| m.len())
                .unwrap_or(0);

            ApiResponse::ok(ExportResult {
                archive_path: archive_path.display().to_string(),
                size_bytes,
            })
        }
        Err(e) => ApiResponse::err(format!("Export failed: {}", e)),
    }
}

/// Preview an archive without importing it.
///
/// Returns information about the archive contents including campaign name,
/// entity counts, and catalog references.
#[tauri::command]
pub fn preview_archive(archive_path: String) -> ApiResponse<ArchivePreview> {
    let path = Path::new(&archive_path);
    let result = ArchiveService::preview_archive(path);
    to_api_response(result)
}

/// Import a campaign from an archive file.
///
/// Creates a new campaign with all data from the archive. Assets are copied
/// to the local assets directory.
#[tauri::command]
pub fn import_campaign(
    state: State<'_, AppState>,
    archive_path: String,
    new_name: Option<String>,
) -> ApiResponse<ImportResult> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let archive = Path::new(&archive_path);
    let assets_path = &state.paths.assets_dir;

    let result = ArchiveService::new(&mut db).import_campaign(
        archive,
        assets_path,
        new_name.as_deref(),
    );
    to_api_response(result)
}
