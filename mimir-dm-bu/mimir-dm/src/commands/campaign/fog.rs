//! Fog of War command handlers.
//!
//! Commands for managing fog of war on maps - enabling/disabling fog,
//! revealing areas, and resetting fog state.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use mimir_dm_core::models::campaign::{FogRevealedArea, FogRevealedAreaSummary};
use mimir_dm_core::services::FogOfWarService;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{error, info};

/// Request to reveal a rectangular area
#[derive(Debug, Deserialize)]
pub struct RevealRectRequest {
    pub map_id: i32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Request to reveal a circular area
#[derive(Debug, Deserialize)]
pub struct RevealCircleRequest {
    pub map_id: i32,
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
}

/// Request to reveal the entire map
#[derive(Debug, Deserialize)]
pub struct RevealAllRequest {
    pub map_id: i32,
    pub map_width: f32,
    pub map_height: f32,
}

/// Fog state response for a map
#[derive(Debug, Serialize)]
pub struct FogState {
    pub map_id: i32,
    pub fog_enabled: bool,
    pub revealed_areas: Vec<FogRevealedAreaSummary>,
}

/// Toggle fog of war on a map.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the new fog_enabled state.
#[tauri::command]
pub async fn toggle_fog(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<bool>, ApiError> {
    info!("Toggling fog for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = FogOfWarService::new(&mut conn);

    match service.toggle_fog(map_id) {
        Ok(new_state) => {
            info!("Fog toggled to: {}", new_state);
            Ok(ApiResponse::success(new_state))
        }
        Err(e) => {
            error!("Failed to toggle fog: {}", e);
            Ok(ApiResponse::error(format!("Failed to toggle fog: {}", e)))
        }
    }
}

/// Enable fog of war on a map.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` indicating success.
#[tauri::command]
pub async fn enable_fog(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
    info!("Enabling fog for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = FogOfWarService::new(&mut conn);

    match service.enable_fog(map_id) {
        Ok(()) => {
            info!("Fog enabled");
            Ok(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to enable fog: {}", e);
            Ok(ApiResponse::error(format!("Failed to enable fog: {}", e)))
        }
    }
}

/// Disable fog of war on a map.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` indicating success.
#[tauri::command]
pub async fn disable_fog(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
    info!("Disabling fog for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = FogOfWarService::new(&mut conn);

    match service.disable_fog(map_id) {
        Ok(()) => {
            info!("Fog disabled");
            Ok(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to disable fog: {}", e);
            Ok(ApiResponse::error(format!("Failed to disable fog: {}", e)))
        }
    }
}

/// Get the current fog state for a map.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the `FogState`.
#[tauri::command]
pub async fn get_fog_state(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<FogState>, ApiError> {
    info!("Getting fog state for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = FogOfWarService::new(&mut conn);

    let fog_enabled = match service.is_fog_enabled(map_id) {
        Ok(enabled) => enabled,
        Err(e) => {
            error!("Failed to get fog enabled state: {}", e);
            return Ok(ApiResponse::error(format!(
                "Failed to get fog state: {}",
                e
            )));
        }
    };

    let revealed_areas = match service.get_revealed_area_summaries(map_id) {
        Ok(areas) => areas,
        Err(e) => {
            error!("Failed to get revealed areas: {}", e);
            return Ok(ApiResponse::error(format!(
                "Failed to get fog state: {}",
                e
            )));
        }
    };

    let fog_state = FogState {
        map_id,
        fog_enabled,
        revealed_areas,
    };

    info!(
        "Fog state: enabled={}, {} revealed areas",
        fog_state.fog_enabled,
        fog_state.revealed_areas.len()
    );
    Ok(ApiResponse::success(fog_state))
}

/// Reveal a rectangular area on the map.
///
/// # Parameters
/// - `request` - Rectangle coordinates and dimensions
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the created `FogRevealedArea`.
#[tauri::command]
pub async fn reveal_rect(
    request: RevealRectRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<FogRevealedArea>, ApiError> {
    info!(
        "Revealing rect at ({}, {}) size {}x{} on map {}",
        request.x, request.y, request.width, request.height, request.map_id
    );

    let mut conn = state.db.get_connection()?;
    let mut service = FogOfWarService::new(&mut conn);

    match service.reveal_rect(
        request.map_id,
        request.x,
        request.y,
        request.width,
        request.height,
    ) {
        Ok(area) => {
            info!("Revealed area created with ID: {}", area.id);
            Ok(ApiResponse::success(area))
        }
        Err(e) => {
            error!("Failed to reveal rect: {}", e);
            Ok(ApiResponse::error(format!("Failed to reveal area: {}", e)))
        }
    }
}

/// Reveal a circular area on the map.
///
/// # Parameters
/// - `request` - Circle center and radius
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the created `FogRevealedArea`.
#[tauri::command]
pub async fn reveal_circle(
    request: RevealCircleRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<FogRevealedArea>, ApiError> {
    info!(
        "Revealing circle at ({}, {}) radius {} on map {}",
        request.center_x, request.center_y, request.radius, request.map_id
    );

    let mut conn = state.db.get_connection()?;
    let mut service = FogOfWarService::new(&mut conn);

    match service.reveal_circle(
        request.map_id,
        request.center_x,
        request.center_y,
        request.radius,
    ) {
        Ok(area) => {
            info!("Revealed area created with ID: {}", area.id);
            Ok(ApiResponse::success(area))
        }
        Err(e) => {
            error!("Failed to reveal circle: {}", e);
            Ok(ApiResponse::error(format!("Failed to reveal area: {}", e)))
        }
    }
}

/// Reveal the entire map (removes all fog).
///
/// # Parameters
/// - `request` - Map ID and dimensions
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the created `FogRevealedArea`.
#[tauri::command]
pub async fn reveal_all(
    request: RevealAllRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<FogRevealedArea>, ApiError> {
    info!("Revealing all of map {}", request.map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = FogOfWarService::new(&mut conn);

    match service.reveal_all(request.map_id, request.map_width, request.map_height) {
        Ok(area) => {
            info!("Full map revealed with area ID: {}", area.id);
            Ok(ApiResponse::success(area))
        }
        Err(e) => {
            error!("Failed to reveal all: {}", e);
            Ok(ApiResponse::error(format!("Failed to reveal all: {}", e)))
        }
    }
}

/// Get all revealed areas for a map.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing a vector of `FogRevealedAreaSummary`.
#[tauri::command]
pub async fn get_revealed_areas(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<FogRevealedAreaSummary>>, ApiError> {
    info!("Getting revealed areas for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = FogOfWarService::new(&mut conn);

    match service.get_revealed_area_summaries(map_id) {
        Ok(areas) => {
            info!("Found {} revealed areas", areas.len());
            Ok(ApiResponse::success(areas))
        }
        Err(e) => {
            error!("Failed to get revealed areas: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to get revealed areas: {}",
                e
            )))
        }
    }
}

/// Delete a specific revealed area (re-fog that area).
///
/// # Parameters
/// - `id` - Database ID of the revealed area
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` indicating success.
#[tauri::command]
pub async fn delete_revealed_area(
    id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
    info!("Deleting revealed area {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = FogOfWarService::new(&mut conn);

    match service.delete_revealed_area(id) {
        Ok(()) => {
            info!("Revealed area deleted");
            Ok(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to delete revealed area: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to delete revealed area: {}",
                e
            )))
        }
    }
}

/// Reset fog on a map (remove all revealed areas).
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the number of revealed areas deleted.
#[tauri::command]
pub async fn reset_fog(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<usize>, ApiError> {
    info!("Resetting fog for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = FogOfWarService::new(&mut conn);

    match service.reset_fog(map_id) {
        Ok(count) => {
            info!("Reset fog, deleted {} revealed areas", count);
            Ok(ApiResponse::success(count))
        }
        Err(e) => {
            error!("Failed to reset fog: {}", e);
            Ok(ApiResponse::error(format!("Failed to reset fog: {}", e)))
        }
    }
}
