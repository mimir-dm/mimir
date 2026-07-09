//! Map POI (Point of Interest) Commands
//!
//! Thin Tauri wrappers over `MapStateService` — all POI logic lives in
//! mimir-core.

use mimir_core::models::campaign::MapPoi;
use mimir_core::services::{CreatePoiInput, MapStateService, UpdatePoiInput};
use serde::Deserialize;
use tauri::State;

use crate::commands::{to_api_response, ApiResponse};
use crate::state::AppState;

/// List all POIs for a map.
#[tauri::command]
pub fn list_map_pois(state: State<'_, AppState>, map_id: String) -> ApiResponse<Vec<MapPoi>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).list_pois(&map_id))
}

/// Get a map POI by ID.
#[tauri::command]
pub fn get_map_poi(state: State<'_, AppState>, id: String) -> ApiResponse<MapPoi> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).get_poi(&id))
}

/// Request for creating a new map POI.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMapPoiRequest {
    pub map_id: String,
    pub name: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub visible: Option<bool>,
}

/// Create a new map POI.
#[tauri::command]
pub fn create_map_poi(
    state: State<'_, AppState>,
    request: CreateMapPoiRequest,
) -> ApiResponse<MapPoi> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let input = CreatePoiInput {
        map_id: request.map_id,
        name: request.name,
        grid_x: request.grid_x,
        grid_y: request.grid_y,
        description: request.description,
        icon: request.icon,
        color: request.color,
        visible: request.visible == Some(true),
    };

    to_api_response(MapStateService::new(&mut db).create_poi(input))
}

/// Request for updating a map POI.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMapPoiRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// Update a map POI.
#[tauri::command]
pub fn update_map_poi(
    state: State<'_, AppState>,
    id: String,
    request: UpdateMapPoiRequest,
) -> ApiResponse<MapPoi> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let input = UpdatePoiInput {
        name: request.name,
        description: request.description,
        icon: request.icon,
        color: request.color,
    };

    to_api_response(MapStateService::new(&mut db).update_poi(&id, input))
}

/// Move a map POI to a new position.
#[tauri::command]
pub fn move_map_poi(
    state: State<'_, AppState>,
    id: String,
    grid_x: i32,
    grid_y: i32,
) -> ApiResponse<MapPoi> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).move_poi(&id, grid_x, grid_y))
}

/// Toggle POI visibility for players.
#[tauri::command]
pub fn toggle_map_poi_visibility(state: State<'_, AppState>, id: String) -> ApiResponse<MapPoi> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).toggle_poi_visibility(&id))
}

/// Delete a map POI.
#[tauri::command]
pub fn delete_map_poi(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(MapStateService::new(&mut db).delete_poi(&id))
}
