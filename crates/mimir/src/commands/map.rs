//! Map Commands
//!
//! Tauri commands for map management (UVTT files).

use mimir_core::models::campaign::{LightingMode, Map};
use mimir_core::services::{CreateMapInput, MapService, UpdateMapInput};
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

// =============================================================================
// List Commands
// =============================================================================

/// List all maps for a campaign (including module maps).
#[tauri::command]
pub fn list_campaign_maps(state: State<'_, AppState>, campaign_id: String) -> ApiResponse<Vec<Map>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = MapService::new(&mut db, &state.paths.app_dir).list_for_campaign(&campaign_id);
    to_api_response(result)
}

/// List only campaign-level maps (not in any module).
#[tauri::command]
pub fn list_campaign_level_maps(
    state: State<'_, AppState>,
    campaign_id: String,
) -> ApiResponse<Vec<Map>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = MapService::new(&mut db, &state.paths.app_dir).list_campaign_level(&campaign_id);
    to_api_response(result)
}

/// List all maps for a module.
#[tauri::command]
pub fn list_module_maps(state: State<'_, AppState>, module_id: String) -> ApiResponse<Vec<Map>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = MapService::new(&mut db, &state.paths.app_dir).list_for_module(&module_id);
    to_api_response(result)
}

// =============================================================================
// CRUD Commands
// =============================================================================

/// Get a map by ID.
#[tauri::command]
pub fn get_map(state: State<'_, AppState>, id: String) -> ApiResponse<Map> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = MapService::new(&mut db, &state.paths.app_dir).get(&id);
    match result {
        Ok(Some(map)) => ApiResponse::ok(map),
        Ok(None) => ApiResponse::err(format!("Map not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Parse lighting mode from string.
fn parse_lighting_mode(s: Option<&str>) -> LightingMode {
    match s {
        Some("dim") => LightingMode::Dim,
        Some("dark") => LightingMode::Dark,
        _ => LightingMode::Bright,
    }
}

/// Request for creating a map.
#[derive(Debug, serde::Deserialize)]
pub struct CreateMapRequest {
    pub campaign_id: String,
    pub module_id: Option<String>,
    pub name: String,
    pub filename: String,
    pub description: Option<String>,
    pub lighting_mode: Option<String>,
    /// Base64-encoded UVTT file data
    pub uvtt_data_base64: String,
}

/// Create a new map from a UVTT file.
#[tauri::command]
pub fn create_map(state: State<'_, AppState>, request: CreateMapRequest) -> ApiResponse<Map> {
    // Decode base64 data
    let uvtt_data = match base64_decode(&request.uvtt_data_base64) {
        Ok(data) => data,
        Err(e) => return ApiResponse::err(format!("Invalid base64 data: {}", e)),
    };

    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut input = if let Some(module_id) = request.module_id {
        CreateMapInput::for_module(
            &request.campaign_id,
            module_id,
            &request.name,
            &request.filename,
            uvtt_data,
        )
    } else {
        CreateMapInput::for_campaign(&request.campaign_id, &request.name, &request.filename, uvtt_data)
    };

    if let Some(desc) = request.description {
        input = input.with_description(desc);
    }

    let lighting_mode = parse_lighting_mode(request.lighting_mode.as_deref());
    input = input.with_lighting_mode(lighting_mode);

    let result = MapService::new(&mut db, &state.paths.app_dir).create(input);
    to_api_response(result)
}

/// Request for updating a map.
#[derive(Debug, serde::Deserialize)]
pub struct UpdateMapRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub sort_order: Option<i32>,
    pub lighting_mode: Option<String>,
}

/// Update a map.
#[tauri::command]
pub fn update_map(
    state: State<'_, AppState>,
    id: String,
    request: UpdateMapRequest,
) -> ApiResponse<Map> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let input = UpdateMapInput {
        name: request.name,
        description: request.description,
        sort_order: request.sort_order,
        lighting_mode: request.lighting_mode.as_deref().map(|s| parse_lighting_mode(Some(s))),
        module_id: None, // Don't allow moving maps via update
    };

    let result = MapService::new(&mut db, &state.paths.app_dir).update(&id, input);
    to_api_response(result)
}

/// Delete a map and its associated UVTT asset.
#[tauri::command]
pub fn delete_map(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = MapService::new(&mut db, &state.paths.app_dir).delete(&id);
    to_api_response(result)
}

// =============================================================================
// UVTT Data Commands
// =============================================================================

/// Read the UVTT file data for a map (returned as base64).
#[tauri::command]
pub fn read_map_uvtt(state: State<'_, AppState>, map_id: String) -> ApiResponse<String> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut service = MapService::new(&mut db, &state.paths.app_dir);

    // Get the map
    let map = match service.get(&map_id) {
        Ok(Some(map)) => map,
        Ok(None) => return ApiResponse::err(format!("Map not found: {}", map_id)),
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // Read the UVTT file
    match service.read_uvtt_file(&map) {
        Ok(data) => ApiResponse::ok(base64_encode(&data)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Simple base64 encoding using the standard library.
fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// Simple base64 decoding.
fn base64_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s)
}
