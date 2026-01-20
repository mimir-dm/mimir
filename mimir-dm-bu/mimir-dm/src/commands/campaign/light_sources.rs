//! Light source command handlers.
//!
//! Commands for managing light sources on maps - creating, updating, deleting,
//! and toggling lights for the vision and lighting system.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use mimir_dm_core::models::campaign::{LightSource, LightSourceSummary, LightType, NewLightSource};
use mimir_dm_core::services::LightSourceService;
use serde::Deserialize;
use tauri::State;
use tracing::{error, info};

/// Request to create a new light source
#[derive(Debug, Deserialize)]
pub struct CreateLightSourceRequest {
    pub map_id: i32,
    pub token_id: Option<i32>,
    pub name: String,
    pub light_type: String,
    pub x: f32,
    pub y: f32,
    pub bright_radius_ft: f32,
    pub dim_radius_ft: f32,
    pub color: Option<String>,
}

/// Request to update a light source
#[derive(Debug, Deserialize)]
pub struct UpdateLightSourceRequest {
    pub name: Option<String>,
    pub light_type: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub bright_radius_ft: Option<f32>,
    pub dim_radius_ft: Option<f32>,
    pub color: Option<Option<String>>,
    pub is_active: Option<bool>,
}

/// Create a new light source on a map.
///
/// # Parameters
/// - `request` - Light source details
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the created `LightSource`.
#[tauri::command]
pub async fn create_light_source(
    request: CreateLightSourceRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<LightSource>, ApiError> {
    info!(
        "Creating light source '{}' at ({}, {}) on map {}",
        request.name, request.x, request.y, request.map_id
    );

    let mut conn = state.db.get_connection()?;
    let mut service = LightSourceService::new(&mut conn);

    let light_type = LightType::from_str(&request.light_type);
    let new_light = NewLightSource {
        map_id: request.map_id,
        token_id: request.token_id,
        name: request.name,
        light_type: light_type.as_str().to_string(),
        x: request.x,
        y: request.y,
        bright_radius_ft: request.bright_radius_ft,
        dim_radius_ft: request.dim_radius_ft,
        color: request.color,
        is_active: true,
    };

    match service.create_light_source(new_light) {
        Ok(light) => {
            info!("Light source created with ID: {}", light.id);
            Ok(ApiResponse::success(light))
        }
        Err(e) => {
            error!("Failed to create light source: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to create light source: {}",
                e
            )))
        }
    }
}

/// Create a quick torch at a position.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `x` - X coordinate in pixels
/// - `y` - Y coordinate in pixels
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the created `LightSource`.
#[tauri::command]
pub async fn create_torch(
    map_id: i32,
    x: f32,
    y: f32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<LightSource>, ApiError> {
    info!("Creating torch at ({}, {}) on map {}", x, y, map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = LightSourceService::new(&mut conn);

    match service.create_torch(map_id, x, y) {
        Ok(light) => {
            info!("Torch created with ID: {}", light.id);
            Ok(ApiResponse::success(light))
        }
        Err(e) => {
            error!("Failed to create torch: {}", e);
            Ok(ApiResponse::error(format!("Failed to create torch: {}", e)))
        }
    }
}

/// Create a quick lantern at a position.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `x` - X coordinate in pixels
/// - `y` - Y coordinate in pixels
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the created `LightSource`.
#[tauri::command]
pub async fn create_lantern(
    map_id: i32,
    x: f32,
    y: f32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<LightSource>, ApiError> {
    info!("Creating lantern at ({}, {}) on map {}", x, y, map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = LightSourceService::new(&mut conn);

    match service.create_lantern(map_id, x, y) {
        Ok(light) => {
            info!("Lantern created with ID: {}", light.id);
            Ok(ApiResponse::success(light))
        }
        Err(e) => {
            error!("Failed to create lantern: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to create lantern: {}",
                e
            )))
        }
    }
}

/// Get a light source by ID.
///
/// # Parameters
/// - `id` - Database ID of the light source
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the `LightSource`.
#[tauri::command]
pub async fn get_light_source(
    id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<LightSource>, ApiError> {
    info!("Getting light source {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = LightSourceService::new(&mut conn);

    match service.get_light_source(id) {
        Ok(light) => Ok(ApiResponse::success(light)),
        Err(e) => {
            error!("Failed to get light source: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to get light source: {}",
                e
            )))
        }
    }
}

/// Get all light sources for a map.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing a vector of `LightSourceSummary`.
#[tauri::command]
pub async fn list_light_sources(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<LightSourceSummary>>, ApiError> {
    info!("Listing light sources for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = LightSourceService::new(&mut conn);

    match service.get_light_source_summaries(map_id) {
        Ok(lights) => {
            info!("Found {} light sources", lights.len());
            Ok(ApiResponse::success(lights))
        }
        Err(e) => {
            error!("Failed to list light sources: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list light sources: {}",
                e
            )))
        }
    }
}

/// Get all active light sources for a map.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing a vector of `LightSource`.
#[tauri::command]
pub async fn list_active_light_sources(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<LightSource>>, ApiError> {
    info!("Listing active light sources for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = LightSourceService::new(&mut conn);

    match service.get_active_light_sources(map_id) {
        Ok(lights) => {
            info!("Found {} active light sources", lights.len());
            Ok(ApiResponse::success(lights))
        }
        Err(e) => {
            error!("Failed to list active light sources: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list active light sources: {}",
                e
            )))
        }
    }
}

/// Update a light source.
///
/// # Parameters
/// - `id` - Database ID of the light source
/// - `request` - Fields to update
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the updated `LightSource`.
#[tauri::command]
pub async fn update_light_source(
    id: i32,
    request: UpdateLightSourceRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<LightSource>, ApiError> {
    info!("Updating light source {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = LightSourceService::new(&mut conn);

    let update = mimir_dm_core::models::campaign::UpdateLightSource {
        name: request.name,
        light_type: request.light_type,
        x: request.x,
        y: request.y,
        bright_radius_ft: request.bright_radius_ft,
        dim_radius_ft: request.dim_radius_ft,
        color: request.color,
        is_active: request.is_active,
        updated_at: None,
    };

    match service.update_light_source(id, update) {
        Ok(light) => {
            info!("Light source updated");
            Ok(ApiResponse::success(light))
        }
        Err(e) => {
            error!("Failed to update light source: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to update light source: {}",
                e
            )))
        }
    }
}

/// Move a light source to a new position.
///
/// # Parameters
/// - `id` - Database ID of the light source
/// - `x` - New X coordinate in pixels
/// - `y` - New Y coordinate in pixels
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the updated `LightSource`.
#[tauri::command]
pub async fn move_light_source(
    id: i32,
    x: f32,
    y: f32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<LightSource>, ApiError> {
    info!("Moving light source {} to ({}, {})", id, x, y);

    let mut conn = state.db.get_connection()?;
    let mut service = LightSourceService::new(&mut conn);

    match service.move_light_source(id, x, y) {
        Ok(light) => {
            info!("Light source moved");
            Ok(ApiResponse::success(light))
        }
        Err(e) => {
            error!("Failed to move light source: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to move light source: {}",
                e
            )))
        }
    }
}

/// Toggle a light source on/off.
///
/// # Parameters
/// - `id` - Database ID of the light source
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the updated `LightSource`.
#[tauri::command]
pub async fn toggle_light_source(
    id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<LightSource>, ApiError> {
    info!("Toggling light source {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = LightSourceService::new(&mut conn);

    match service.toggle_light_source(id) {
        Ok(light) => {
            info!("Light source toggled to: {}", light.is_active);
            Ok(ApiResponse::success(light))
        }
        Err(e) => {
            error!("Failed to toggle light source: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to toggle light source: {}",
                e
            )))
        }
    }
}

/// Delete a light source.
///
/// # Parameters
/// - `id` - Database ID of the light source
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` indicating success.
#[tauri::command]
pub async fn delete_light_source(
    id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
    info!("Deleting light source {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = LightSourceService::new(&mut conn);

    match service.delete_light_source(id) {
        Ok(()) => {
            info!("Light source deleted");
            Ok(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to delete light source: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to delete light source: {}",
                e
            )))
        }
    }
}

/// Delete all light sources on a map.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the number of light sources deleted.
#[tauri::command]
pub async fn delete_all_light_sources(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<usize>, ApiError> {
    info!("Deleting all light sources for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = LightSourceService::new(&mut conn);

    match service.delete_all_for_map(map_id) {
        Ok(count) => {
            info!("Deleted {} light sources", count);
            Ok(ApiResponse::success(count))
        }
        Err(e) => {
            error!("Failed to delete light sources: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to delete light sources: {}",
                e
            )))
        }
    }
}
