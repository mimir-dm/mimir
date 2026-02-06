//! Map CRUD Commands
//!
//! List, get, create, update, and delete map operations.

use mimir_core::models::campaign::{LightingMode, Map};
use mimir_core::services::{CreateMapInput, MapService, UpdateMapInput};
use tauri::State;

use super::{base64_decode, enrich_map_with_uvtt, enrich_maps_with_uvtt, MapResponse};
use crate::commands::{to_api_response, ApiResponse};
use crate::state::AppState;

// =============================================================================
// List Commands
// =============================================================================

/// List all maps for a campaign (including module maps).
#[tauri::command]
pub fn list_campaign_maps(state: State<'_, AppState>, campaign_id: String) -> ApiResponse<Vec<MapResponse>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let mut service = MapService::new(&mut db, &state.paths.app_dir);
    match service.list_for_campaign(&campaign_id) {
        Ok(maps) => ApiResponse::ok(enrich_maps_with_uvtt(maps, &mut service, &state.paths.app_dir)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List only campaign-level maps (not in any module).
#[tauri::command]
pub fn list_campaign_level_maps(
    state: State<'_, AppState>,
    campaign_id: String,
) -> ApiResponse<Vec<MapResponse>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let mut service = MapService::new(&mut db, &state.paths.app_dir);
    match service.list_campaign_level(&campaign_id) {
        Ok(maps) => ApiResponse::ok(enrich_maps_with_uvtt(maps, &mut service, &state.paths.app_dir)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all maps for a module.
#[tauri::command]
pub fn list_module_maps(state: State<'_, AppState>, module_id: String) -> ApiResponse<Vec<MapResponse>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let mut service = MapService::new(&mut db, &state.paths.app_dir);
    match service.list_for_module(&module_id) {
        Ok(maps) => ApiResponse::ok(enrich_maps_with_uvtt(maps, &mut service, &state.paths.app_dir)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// CRUD Commands
// =============================================================================

/// Get a map by ID.
#[tauri::command]
pub fn get_map(state: State<'_, AppState>, id: String) -> ApiResponse<MapResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let mut service = MapService::new(&mut db, &state.paths.app_dir);
    match service.get(&id) {
        Ok(Some(map)) => ApiResponse::ok(enrich_map_with_uvtt(&map, &mut service, &state.paths.app_dir)),
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

    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = MapService::new(&mut db, &state.paths.app_dir).delete(&id);
    to_api_response(result)
}
