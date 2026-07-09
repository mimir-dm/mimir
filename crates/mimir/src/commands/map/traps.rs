//! Map Trap Commands
//!
//! Thin Tauri wrappers over `MapStateService` — all trap logic lives in
//! mimir-core.

use mimir_core::models::campaign::MapTrap;
use mimir_core::services::{CreateTrapInput, MapStateService, UpdateTrapInput};
use serde::Deserialize;
use tauri::State;

use crate::commands::{to_api_response, ApiResponse};
use crate::state::AppState;

/// List all traps for a map.
#[tauri::command]
pub fn list_map_traps(state: State<'_, AppState>, map_id: String) -> ApiResponse<Vec<MapTrap>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).list_traps(&map_id))
}

/// Get a map trap by ID.
#[tauri::command]
pub fn get_map_trap(state: State<'_, AppState>, id: String) -> ApiResponse<MapTrap> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).get_trap(&id))
}

/// Request for creating a new map trap.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMapTrapRequest {
    pub map_id: String,
    pub name: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub description: Option<String>,
    pub trigger_description: Option<String>,
    pub effect_description: Option<String>,
    pub dc: Option<i32>,
    pub visible: Option<bool>,
}

/// Create a new map trap.
#[tauri::command]
pub fn create_map_trap(
    state: State<'_, AppState>,
    request: CreateMapTrapRequest,
) -> ApiResponse<MapTrap> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let input = CreateTrapInput {
        map_id: request.map_id,
        name: request.name,
        grid_x: request.grid_x,
        grid_y: request.grid_y,
        description: request.description,
        trigger_description: request.trigger_description,
        effect_description: request.effect_description,
        dc: request.dc,
        visible: request.visible == Some(true),
    };

    to_api_response(MapStateService::new(&mut db).create_trap(input))
}

/// Request for updating a map trap.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMapTrapRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger_description: Option<String>,
    pub effect_description: Option<String>,
    pub dc: Option<i32>,
}

/// Update a map trap.
#[tauri::command]
pub fn update_map_trap(
    state: State<'_, AppState>,
    id: String,
    request: UpdateMapTrapRequest,
) -> ApiResponse<MapTrap> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let input = UpdateTrapInput {
        name: request.name,
        description: request.description,
        trigger_description: request.trigger_description,
        effect_description: request.effect_description,
        dc: request.dc,
    };

    to_api_response(MapStateService::new(&mut db).update_trap(&id, input))
}

/// Move a map trap to a new position.
#[tauri::command]
pub fn move_map_trap(
    state: State<'_, AppState>,
    id: String,
    grid_x: i32,
    grid_y: i32,
) -> ApiResponse<MapTrap> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).move_trap(&id, grid_x, grid_y))
}

/// Toggle trap visibility for players.
#[tauri::command]
pub fn toggle_map_trap_visibility(state: State<'_, AppState>, id: String) -> ApiResponse<MapTrap> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).toggle_trap_visibility(&id))
}

/// Trigger a trap.
#[tauri::command]
pub fn trigger_map_trap(state: State<'_, AppState>, id: String) -> ApiResponse<MapTrap> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).trigger_trap(&id))
}

/// Reset (re-arm) a triggered trap.
#[tauri::command]
pub fn reset_map_trap(state: State<'_, AppState>, id: String) -> ApiResponse<MapTrap> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).reset_trap(&id))
}

/// Delete a map trap.
#[tauri::command]
pub fn delete_map_trap(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).delete_trap(&id))
}
