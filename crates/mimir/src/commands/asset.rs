//! Asset Commands
//!
//! Tauri commands for binary asset management (images, files).

use mimir_core::models::campaign::CampaignAsset;
use mimir_core::services::{AssetService, UploadAssetInput};
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

// =============================================================================
// List Commands
// =============================================================================

/// List all assets for a campaign.
#[tauri::command]
pub fn list_campaign_assets(
    state: State<'_, AppState>,
    campaign_id: String,
) -> ApiResponse<Vec<CampaignAsset>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = AssetService::new(&mut db, &state.paths.app_dir).list_for_campaign(&campaign_id);
    to_api_response(result)
}

/// List all assets for a module.
#[tauri::command]
pub fn list_module_assets(
    state: State<'_, AppState>,
    module_id: String,
) -> ApiResponse<Vec<CampaignAsset>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = AssetService::new(&mut db, &state.paths.app_dir).list_for_module(&module_id);
    to_api_response(result)
}

// =============================================================================
// CRUD Commands
// =============================================================================

/// Get an asset by ID.
#[tauri::command]
pub fn get_asset(state: State<'_, AppState>, id: String) -> ApiResponse<CampaignAsset> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = AssetService::new(&mut db, &state.paths.app_dir).get(&id);
    match result {
        Ok(Some(asset)) => ApiResponse::ok(asset),
        Ok(None) => ApiResponse::err(format!("Asset not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for uploading an asset.
#[derive(Debug, serde::Deserialize)]
pub struct UploadAssetRequest {
    /// Campaign ID (for campaign-level assets)
    pub campaign_id: Option<String>,
    /// Module ID (for module-level assets)
    pub module_id: Option<String>,
    /// Original filename
    pub filename: String,
    /// Optional description/title for the asset
    pub description: Option<String>,
    /// MIME type (e.g., "image/png")
    pub mime_type: String,
    /// Base64-encoded file data
    pub data_base64: String,
}

/// Upload a new asset.
#[tauri::command]
pub fn upload_asset(
    state: State<'_, AppState>,
    request: UploadAssetRequest,
) -> ApiResponse<CampaignAsset> {
    // Decode base64 data
    let data = match base64_decode(&request.data_base64) {
        Ok(data) => data,
        Err(e) => return ApiResponse::err(format!("Invalid base64 data: {}", e)),
    };

    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut input = if let Some(campaign_id) = request.campaign_id {
        UploadAssetInput::for_campaign(campaign_id, &request.filename, &request.mime_type, data)
    } else if let Some(module_id) = request.module_id {
        UploadAssetInput::for_module(module_id, &request.filename, &request.mime_type, data)
    } else {
        return ApiResponse::err("Either campaign_id or module_id must be provided");
    };

    // Add description if provided
    if let Some(desc) = request.description {
        input = input.with_description(desc);
    }

    let result = AssetService::new(&mut db, &state.paths.app_dir).upload(input);
    to_api_response(result)
}

/// Delete an asset.
#[tauri::command]
pub fn delete_asset(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = AssetService::new(&mut db, &state.paths.app_dir).delete(&id);
    to_api_response(result)
}

// =============================================================================
// File Data Commands
// =============================================================================

/// Read the file data for an asset (returned as base64).
#[tauri::command]
pub fn read_asset_file(state: State<'_, AppState>, id: String) -> ApiResponse<String> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut service = AssetService::new(&mut db, &state.paths.app_dir);

    // Get the asset
    let asset = match service.get(&id) {
        Ok(Some(asset)) => asset,
        Ok(None) => return ApiResponse::err(format!("Asset not found: {}", id)),
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // Read the file
    match service.read_file(&asset) {
        Ok(data) => ApiResponse::ok(base64_encode(&data)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Simple base64 encoding.
fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// Simple base64 decoding.
fn base64_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s)
}
