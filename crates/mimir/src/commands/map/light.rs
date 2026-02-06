//! Light Source Commands
//!
//! Commands for managing light sources on maps.

use mimir_core::dal::campaign as dal;
use mimir_core::models::campaign::{NewLightSource, UpdateLightSource};
use mimir_core::services::MapService;
use mimir_core::utils::now_rfc3339;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use super::{get_map_grid_size_for_lights, transform_light_source, LightSourceResponse};
use crate::commands::ApiResponse;
use crate::state::AppState;

// =============================================================================
// Light Source Commands
// =============================================================================

/// List all light sources for a map.
#[tauri::command]
pub fn list_light_sources(state: State<'_, AppState>, map_id: String) -> ApiResponse<Vec<LightSourceResponse>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get grid size for coordinate conversion
    let mut service = MapService::new(&mut db, &state.paths.app_dir);
    let grid_size_px = get_map_grid_size_for_lights(&mut service, &map_id);

    match dal::list_light_sources(&mut db, &map_id) {
        Ok(lights) => {
            let responses: Vec<LightSourceResponse> = lights
                .into_iter()
                .map(|l| transform_light_source(l, grid_size_px))
                .collect();
            ApiResponse::ok(responses)
        }
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

/// Helper to get a light source and return it as a response with proper coordinates.
fn get_light_response(
    db: &mut diesel::SqliteConnection,
    app_dir: &std::path::Path,
    light_id: &str,
) -> Result<LightSourceResponse, String> {
    let ls = dal::get_light_source(db, light_id).map_err(|e| e.to_string())?;
    let mut service = MapService::new(db, app_dir);
    let grid_size_px = get_map_grid_size_for_lights(&mut service, &ls.map_id);
    Ok(transform_light_source(ls, grid_size_px))
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

    // Get grid size to convert pixel coordinates to grid coordinates
    let mut service = MapService::new(&mut db, &state.paths.app_dir);
    let grid_size_px = get_map_grid_size_for_lights(&mut service, &request.map_id);

    // Convert pixel coordinates to grid coordinates
    let grid_x = (request.x / grid_size_px as f64) as i32;
    let grid_y = (request.y / grid_size_px as f64) as i32;

    let id = Uuid::new_v4().to_string();
    let name_owned = request.name.clone();
    let color_owned = request.color.clone();

    let mut light = NewLightSource::new(
        &id,
        &request.map_id,
        grid_x,
        grid_y,
        request.bright_radius_ft,
        request.dim_radius_ft,
    )
    .with_name(&name_owned);

    if let Some(ref color) = color_owned {
        light = light.with_color(color);
    }

    if !request.is_active {
        light = light.inactive();
    }

    if let Err(e) = dal::insert_light_source(&mut db, &light) {
        return ApiResponse::err(e.to_string());
    }

    match get_light_response(&mut db, &state.paths.app_dir, &id) {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::err(e),
    }
}

/// Toggle a light source on/off.
#[tauri::command]
pub fn toggle_light_source(state: State<'_, AppState>, id: String) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get current state
    let light = match dal::get_light_source(&mut db, &id) {
        Ok(l) => l,
        Err(e) => return ApiResponse::err(format!("Light source not found: {}", e)),
    };

    let now = now_rfc3339();
    let update = if light.is_active() {
        UpdateLightSource::turn_off(&now)
    } else {
        UpdateLightSource::turn_on(&now)
    };

    if let Err(e) = dal::update_light_source(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match get_light_response(&mut db, &state.paths.app_dir, &id) {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::err(e),
    }
}

/// Delete a light source.
#[tauri::command]
pub fn delete_light_source(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match dal::delete_light_source(&mut db, &id) {
        Ok(_) => ApiResponse::ok(()),
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

    match dal::delete_all_light_sources(&mut db, &map_id) {
        Ok(count) => ApiResponse::ok(count as i32),
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

    let id = Uuid::new_v4().to_string();
    let light = NewLightSource::torch(&id, &map_id, x, y);

    if let Err(e) = dal::insert_light_source(&mut db, &light) {
        return ApiResponse::err(e.to_string());
    }

    match get_light_response(&mut db, &state.paths.app_dir, &id) {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::err(e),
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

    let id = Uuid::new_v4().to_string();
    let light = NewLightSource::lantern(&id, &map_id, x, y);

    if let Err(e) = dal::insert_light_source(&mut db, &light) {
        return ApiResponse::err(e.to_string());
    }

    match get_light_response(&mut db, &state.paths.app_dir, &id) {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::err(e),
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

    let now = now_rfc3339();

    // Build update struct based on what fields are provided
    let name: Option<Option<&str>> = match &request.name {
        Some(inner) => Some(inner.as_deref()),
        None => None,
    };
    let color: Option<Option<&str>> = match &request.color {
        Some(inner) => Some(inner.as_deref()),
        None => None,
    };

    let update = UpdateLightSource {
        grid_x: None,
        grid_y: None,
        name,
        bright_radius: request.bright_radius_ft,
        dim_radius: request.dim_radius_ft,
        color,
        active: request.is_active.map(|a| if a { 1 } else { 0 }),
        updated_at: Some(&now),
    };

    if let Err(e) = dal::update_light_source(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match get_light_response(&mut db, &state.paths.app_dir, &id) {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::err(e),
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

    let now = now_rfc3339();
    let update = UpdateLightSource::set_position(x, y, &now);

    if let Err(e) = dal::update_light_source(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match get_light_response(&mut db, &state.paths.app_dir, &id) {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::err(e),
    }
}
