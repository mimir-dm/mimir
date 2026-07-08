//! Fog of War Commands
//!
//! Thin Tauri wrappers over `MapStateService` — all fog logic lives in
//! mimir-core.

use mimir_core::models::campaign::{FogRevealedArea, FogState};
use mimir_core::services::MapStateService;
use serde::Deserialize;
use tauri::State;

use crate::commands::{to_api_response, ApiResponse};
use crate::state::AppState;

/// Get the fog state for a map (enabled + revealed areas).
#[tauri::command]
pub fn get_fog_state(state: State<'_, AppState>, map_id: String) -> ApiResponse<FogState> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).fog_state(&map_id))
}

/// Toggle fog of war on/off for a map.
#[tauri::command]
pub fn toggle_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<bool> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).toggle_fog(&map_id))
}

/// Enable fog of war for a map.
#[tauri::command]
pub fn enable_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).enable_fog(&map_id))
}

/// Disable fog of war for a map.
#[tauri::command]
pub fn disable_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).disable_fog(&map_id))
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

    to_api_response(MapStateService::new(&mut db).reveal_rect(
        &request.map_id,
        request.x,
        request.y,
        request.width,
        request.height,
    ))
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

    to_api_response(MapStateService::new(&mut db).reveal_circle(
        &request.map_id,
        request.center_x,
        request.center_y,
        request.radius,
    ))
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

    to_api_response(MapStateService::new(&mut db).reveal_all(
        &request.map_id,
        request.width,
        request.height,
    ))
}

/// Delete a revealed area.
#[tauri::command]
pub fn delete_revealed_area(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).delete_revealed_area(&id))
}

/// Reset fog by clearing all revealed areas for a map.
#[tauri::command]
pub fn reset_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<i32> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).reset_fog(&map_id))
}
