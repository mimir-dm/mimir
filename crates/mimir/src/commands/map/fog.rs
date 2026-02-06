//! Fog of War Commands
//!
//! Commands for managing fog of war state and revealed areas.

use mimir_core::dal::campaign as dal;
use mimir_core::models::campaign::{FogRevealedArea, FogState, NewFogRevealedArea, UpdateMap};
use mimir_core::utils::now_rfc3339;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::commands::ApiResponse;
use crate::state::AppState;

// =============================================================================
// Fog of War Commands
// =============================================================================

/// Get the fog state for a map (enabled + revealed areas).
#[tauri::command]
pub fn get_fog_state(state: State<'_, AppState>, map_id: String) -> ApiResponse<FogState> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get map to check fog_enabled
    let map = match dal::get_map(&mut db, &map_id) {
        Ok(m) => m,
        Err(e) => return ApiResponse::err(format!("Map not found: {}", e)),
    };

    // Get revealed areas
    let revealed_areas = match dal::list_fog_revealed_areas(&mut db, &map_id) {
        Ok(areas) => areas,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    ApiResponse::ok(FogState::new(
        map_id,
        map.is_fog_enabled(),
        revealed_areas,
    ))
}

/// Toggle fog of war on/off for a map.
#[tauri::command]
pub fn toggle_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<bool> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get current fog state
    let map = match dal::get_map(&mut db, &map_id) {
        Ok(m) => m,
        Err(e) => return ApiResponse::err(format!("Map not found: {}", e)),
    };

    let now = now_rfc3339();
    let new_enabled = !map.is_fog_enabled();
    let update = UpdateMap::set_fog_enabled(new_enabled, &now);

    if let Err(e) = dal::update_map(&mut db, &map_id, &update) {
        return ApiResponse::err(e.to_string());
    }

    ApiResponse::ok(new_enabled)
}

/// Enable fog of war for a map.
#[tauri::command]
pub fn enable_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let now = now_rfc3339();
    let update = UpdateMap::enable_fog(&now);

    match dal::update_map(&mut db, &map_id, &update) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Disable fog of war for a map.
#[tauri::command]
pub fn disable_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let now = now_rfc3339();
    let update = UpdateMap::disable_fog(&now);

    match dal::update_map(&mut db, &map_id, &update) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for revealing a rectangular area.
#[derive(Debug, Deserialize)]
pub struct RevealRectRequest {
    pub map_id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Reveal a rectangular area on the map.
#[tauri::command]
pub fn reveal_rect(
    state: State<'_, AppState>,
    request: RevealRectRequest,
) -> ApiResponse<FogRevealedArea> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let id = Uuid::new_v4().to_string();
    let area = NewFogRevealedArea::rect(
        &id,
        &request.map_id,
        request.x,
        request.y,
        request.width,
        request.height,
    );

    if let Err(e) = dal::insert_fog_revealed_area(&mut db, &area) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_fog_revealed_area(&mut db, &id) {
        Ok(area) => ApiResponse::ok(area),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for revealing a circular area.
#[derive(Debug, Deserialize)]
pub struct RevealCircleRequest {
    pub map_id: String,
    pub center_x: f64,
    pub center_y: f64,
    pub radius: f64,
}

/// Reveal a circular area on the map (stored as bounding box).
#[tauri::command]
pub fn reveal_circle(
    state: State<'_, AppState>,
    request: RevealCircleRequest,
) -> ApiResponse<FogRevealedArea> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let id = Uuid::new_v4().to_string();
    let area = NewFogRevealedArea::circle(
        &id,
        &request.map_id,
        request.center_x,
        request.center_y,
        request.radius,
    );

    if let Err(e) = dal::insert_fog_revealed_area(&mut db, &area) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_fog_revealed_area(&mut db, &id) {
        Ok(area) => ApiResponse::ok(area),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for revealing the entire map.
#[derive(Debug, Deserialize)]
pub struct RevealAllRequest {
    pub map_id: String,
    pub width: f64,
    pub height: f64,
}

/// Reveal the entire map.
#[tauri::command]
pub fn reveal_all(
    state: State<'_, AppState>,
    request: RevealAllRequest,
) -> ApiResponse<FogRevealedArea> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let id = Uuid::new_v4().to_string();
    let area = NewFogRevealedArea::rect(
        &id,
        &request.map_id,
        0.0,
        0.0,
        request.width,
        request.height,
    );

    if let Err(e) = dal::insert_fog_revealed_area(&mut db, &area) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_fog_revealed_area(&mut db, &id) {
        Ok(area) => ApiResponse::ok(area),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete a revealed area.
#[tauri::command]
pub fn delete_revealed_area(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match dal::delete_fog_revealed_area(&mut db, &id) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Reset fog by clearing all revealed areas for a map.
#[tauri::command]
pub fn reset_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<i32> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match dal::delete_all_fog_revealed_areas(&mut db, &map_id) {
        Ok(count) => ApiResponse::ok(count as i32),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}
