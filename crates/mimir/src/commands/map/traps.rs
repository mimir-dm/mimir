//! Map Trap Commands
//!
//! Commands for managing traps placed on maps.

use mimir_core::dal::campaign as dal;
use mimir_core::models::campaign::{MapTrap, NewMapTrap, UpdateMapTrap};
use mimir_core::utils::now_rfc3339;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::commands::ApiResponse;
use crate::state::AppState;

// =============================================================================
// Map Trap Commands
// =============================================================================

/// List all traps for a map.
#[tauri::command]
pub fn list_map_traps(state: State<'_, AppState>, map_id: String) -> ApiResponse<Vec<MapTrap>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match dal::list_map_traps(&mut db, &map_id) {
        Ok(traps) => ApiResponse::ok(traps),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a map trap by ID.
#[tauri::command]
pub fn get_map_trap(state: State<'_, AppState>, id: String) -> ApiResponse<MapTrap> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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

    let id = Uuid::new_v4().to_string();
    let mut trap = NewMapTrap::new(&id, &request.map_id, &request.name, request.grid_x, request.grid_y);

    if let Some(desc) = &request.description {
        trap = trap.with_description(desc);
    }
    if let Some(trigger) = &request.trigger_description {
        trap = trap.with_trigger(trigger);
    }
    if let Some(effect) = &request.effect_description {
        trap = trap.with_effect(effect);
    }
    if let Some(dc) = request.dc {
        trap = trap.with_dc(dc);
    }
    if request.visible == Some(true) {
        trap = trap.visible();
    }

    if let Err(e) = dal::insert_map_trap(&mut db, &trap) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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

    let now = now_rfc3339();
    let name_str = request.name;
    let desc_str = request.description;
    let trigger_str = request.trigger_description;
    let effect_str = request.effect_description;

    let update = UpdateMapTrap {
        name: name_str.as_deref(),
        description: desc_str.as_ref().map(|s| Some(s.as_str())),
        trigger_description: trigger_str.as_ref().map(|s| Some(s.as_str())),
        effect_description: effect_str.as_ref().map(|s| Some(s.as_str())),
        dc: request.dc.map(Some),
        updated_at: Some(&now),
        ..Default::default()
    };

    if let Err(e) = dal::update_map_trap(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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

    let now = now_rfc3339();
    let update = UpdateMapTrap::set_position(grid_x, grid_y, &now);

    if let Err(e) = dal::update_map_trap(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Toggle trap visibility for players.
#[tauri::command]
pub fn toggle_map_trap_visibility(state: State<'_, AppState>, id: String) -> ApiResponse<MapTrap> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get current state
    let trap = match dal::get_map_trap(&mut db, &id) {
        Ok(t) => t,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let now = now_rfc3339();
    let update = UpdateMapTrap::set_visible(!trap.is_visible(), &now);

    if let Err(e) = dal::update_map_trap(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Trigger a trap.
#[tauri::command]
pub fn trigger_map_trap(state: State<'_, AppState>, id: String) -> ApiResponse<MapTrap> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let now = now_rfc3339();
    let update = UpdateMapTrap::trigger(&now);

    if let Err(e) = dal::update_map_trap(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Reset (re-arm) a triggered trap.
#[tauri::command]
pub fn reset_map_trap(state: State<'_, AppState>, id: String) -> ApiResponse<MapTrap> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let now = now_rfc3339();
    let update = UpdateMapTrap::reset(&now);

    if let Err(e) = dal::update_map_trap(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete a map trap.
#[tauri::command]
pub fn delete_map_trap(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match dal::delete_map_trap(&mut db, &id) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}
