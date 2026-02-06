//! Map POI (Point of Interest) Commands
//!
//! Commands for managing points of interest placed on maps.

use mimir_core::dal::campaign as dal;
use mimir_core::models::campaign::{MapPoi, NewMapPoi, UpdateMapPoi};
use mimir_core::utils::now_rfc3339;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::commands::ApiResponse;
use crate::state::AppState;

// =============================================================================
// Map POI Commands
// =============================================================================

/// List all POIs for a map.
#[tauri::command]
pub fn list_map_pois(state: State<'_, AppState>, map_id: String) -> ApiResponse<Vec<MapPoi>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match dal::list_map_pois(&mut db, &map_id) {
        Ok(pois) => ApiResponse::ok(pois),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a map POI by ID.
#[tauri::command]
pub fn get_map_poi(state: State<'_, AppState>, id: String) -> ApiResponse<MapPoi> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match dal::get_map_poi(&mut db, &id) {
        Ok(poi) => ApiResponse::ok(poi),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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

    let id = Uuid::new_v4().to_string();
    let mut poi = NewMapPoi::new(&id, &request.map_id, &request.name, request.grid_x, request.grid_y);

    if let Some(desc) = &request.description {
        poi = poi.with_description(desc);
    }
    if let Some(icon) = &request.icon {
        poi = poi.with_icon(icon);
    }
    if let Some(color) = &request.color {
        poi = poi.with_color(color);
    }
    if request.visible == Some(true) {
        poi = poi.visible();
    }

    if let Err(e) = dal::insert_map_poi(&mut db, &poi) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_poi(&mut db, &id) {
        Ok(poi) => ApiResponse::ok(poi),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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

    let now = now_rfc3339();
    let name_str = request.name;
    let desc_str = request.description;
    let icon_str = request.icon;
    let color_str = request.color;

    let update = UpdateMapPoi {
        name: name_str.as_deref(),
        description: desc_str.as_ref().map(|s| Some(s.as_str())),
        icon: icon_str.as_deref(),
        color: color_str.as_ref().map(|s| Some(s.as_str())),
        updated_at: Some(&now),
        ..Default::default()
    };

    if let Err(e) = dal::update_map_poi(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_poi(&mut db, &id) {
        Ok(poi) => ApiResponse::ok(poi),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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

    let now = now_rfc3339();
    let update = UpdateMapPoi::set_position(grid_x, grid_y, &now);

    if let Err(e) = dal::update_map_poi(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_poi(&mut db, &id) {
        Ok(poi) => ApiResponse::ok(poi),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Toggle POI visibility for players.
#[tauri::command]
pub fn toggle_map_poi_visibility(state: State<'_, AppState>, id: String) -> ApiResponse<MapPoi> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get current state
    let poi = match dal::get_map_poi(&mut db, &id) {
        Ok(p) => p,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let now = now_rfc3339();
    let update = UpdateMapPoi::set_visible(!poi.is_visible(), &now);

    if let Err(e) = dal::update_map_poi(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_poi(&mut db, &id) {
        Ok(poi) => ApiResponse::ok(poi),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete a map POI.
#[tauri::command]
pub fn delete_map_poi(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match dal::delete_map_poi(&mut db, &id) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}
