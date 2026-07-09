//! Light Source Commands
//!
//! Thin Tauri wrappers over `MapStateService`. The pixel↔grid coordinate
//! conversion and `LightSourceResponse` transform are presentation concerns
//! (they need the map's UVTT grid size) and stay here.

use mimir_core::services::{CreateLightInput, MapService, MapStateService, UpdateLightInput};
use serde::Deserialize;
use tauri::State;

use super::{get_map_grid_size_for_lights, transform_light_source, LightSourceResponse};
use crate::commands::ApiResponse;
use crate::state::AppState;

/// Look up the grid size for a map (presentation: pixels per grid unit).
fn grid_size_px(
    db: &mut diesel::SqliteConnection,
    app_dir: &std::path::Path,
    map_id: &str,
) -> i32 {
    let mut service = MapService::new(db, app_dir);
    get_map_grid_size_for_lights(&mut service, map_id)
}

/// List all light sources for a map.
#[tauri::command]
pub fn list_light_sources(state: State<'_, AppState>, map_id: String) -> ApiResponse<Vec<LightSourceResponse>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let px = grid_size_px(&mut db, &state.paths.app_dir, &map_id);

    match MapStateService::new(&mut db).list_lights(&map_id) {
        Ok(lights) => ApiResponse::ok(
            lights
                .into_iter()
                .map(|l| transform_light_source(l, px))
                .collect(),
        ),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for creating a light source.
#[derive(Debug, Deserialize)]
pub struct CreateLightSourceRequest {
    pub map_id: String,
    pub name: String,
    pub light_type: String,
    pub x: f64,
    pub y: f64,
    pub bright_radius_ft: i32,
    pub dim_radius_ft: i32,
    pub color: Option<String>,
    pub is_active: bool,
}

/// Create a new light source.
#[tauri::command]
pub fn create_light_source(
    state: State<'_, AppState>,
    request: CreateLightSourceRequest,
) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Convert pixel coordinates to grid coordinates (presentation concern)
    let px = grid_size_px(&mut db, &state.paths.app_dir, &request.map_id);
    let grid_x = (request.x / px as f64) as i32;
    let grid_y = (request.y / px as f64) as i32;

    let input = CreateLightInput {
        map_id: request.map_id,
        grid_x,
        grid_y,
        bright_radius_ft: request.bright_radius_ft,
        dim_radius_ft: request.dim_radius_ft,
        name: request.name,
        color: request.color,
        is_active: request.is_active,
    };

    match MapStateService::new(&mut db).create_light(input) {
        Ok(light) => ApiResponse::ok(transform_light_source(light, px)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Toggle a light source on/off.
#[tauri::command]
pub fn toggle_light_source(state: State<'_, AppState>, id: String) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match MapStateService::new(&mut db).toggle_light(&id) {
        Ok(light) => {
            let px = grid_size_px(&mut db, &state.paths.app_dir, &light.map_id);
            ApiResponse::ok(transform_light_source(light, px))
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete a light source.
#[tauri::command]
pub fn delete_light_source(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match MapStateService::new(&mut db).delete_light(&id) {
        Ok(()) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete all light sources on a map.
#[tauri::command]
pub fn delete_all_light_sources(state: State<'_, AppState>, map_id: String) -> ApiResponse<i32> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match MapStateService::new(&mut db).delete_all_lights(&map_id) {
        Ok(count) => ApiResponse::ok(count),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Create a torch light source (20ft bright, 40ft dim).
#[tauri::command]
pub fn create_torch(
    state: State<'_, AppState>,
    map_id: String,
    x: i32,
    y: i32,
) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match MapStateService::new(&mut db).create_torch(&map_id, x, y) {
        Ok(light) => {
            let px = grid_size_px(&mut db, &state.paths.app_dir, &light.map_id);
            ApiResponse::ok(transform_light_source(light, px))
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Create a lantern light source (30ft bright, 60ft dim).
#[tauri::command]
pub fn create_lantern(
    state: State<'_, AppState>,
    map_id: String,
    x: i32,
    y: i32,
) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match MapStateService::new(&mut db).create_lantern(&map_id, x, y) {
        Ok(light) => {
            let px = grid_size_px(&mut db, &state.paths.app_dir, &light.map_id);
            ApiResponse::ok(transform_light_source(light, px))
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for updating a light source.
#[derive(Debug, Deserialize)]
pub struct UpdateLightSourceRequest {
    pub name: Option<Option<String>>,
    pub bright_radius_ft: Option<i32>,
    pub dim_radius_ft: Option<i32>,
    pub color: Option<Option<String>>,
    pub is_active: Option<bool>,
}

/// Update a light source.
#[tauri::command]
pub fn update_light_source(
    state: State<'_, AppState>,
    id: String,
    request: UpdateLightSourceRequest,
) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let input = UpdateLightInput {
        name: request.name,
        bright_radius_ft: request.bright_radius_ft,
        dim_radius_ft: request.dim_radius_ft,
        color: request.color,
        is_active: request.is_active,
    };

    match MapStateService::new(&mut db).update_light(&id, input) {
        Ok(light) => {
            let px = grid_size_px(&mut db, &state.paths.app_dir, &light.map_id);
            ApiResponse::ok(transform_light_source(light, px))
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Move a light source to a new position.
#[tauri::command]
pub fn move_light_source(
    state: State<'_, AppState>,
    id: String,
    x: i32,
    y: i32,
) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match MapStateService::new(&mut db).move_light(&id, x, y) {
        Ok(light) => {
            let px = grid_size_px(&mut db, &state.paths.app_dir, &light.map_id);
            ApiResponse::ok(transform_light_source(light, px))
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}
