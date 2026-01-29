//! Development Tools Commands
//!
//! Tauri commands for development-only functionality such as test data seeding.
//! These commands check for dev mode before executing.

use mimir_core::seed;
use serde::Serialize;
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

/// Application info returned to the frontend.
#[derive(Serialize)]
pub struct AppInfo {
    pub database_path: String,
}

/// Get application info (database path, etc.) for the settings UI.
#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> ApiResponse<AppInfo> {
    ApiResponse::ok(AppInfo {
        database_path: state.paths.database_path.to_string_lossy().to_string(),
    })
}

/// Check if the application is running in development mode.
#[tauri::command]
pub fn is_dev_mode() -> ApiResponse<bool> {
    ApiResponse::ok(cfg!(debug_assertions))
}

/// Check if dev seed data already exists.
#[tauri::command]
pub fn is_dev_seeded(state: State<'_, AppState>) -> ApiResponse<bool> {
    if !cfg!(debug_assertions) {
        return ApiResponse::err("Not in development mode");
    }

    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match seed::is_already_seeded(&mut db) {
        Ok(seeded) => ApiResponse::ok(seeded),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Seed development test data.
///
/// Creates a test campaign with modules, characters, monsters, and NPCs.
/// Only works in development mode and only seeds if data doesn't already exist.
#[tauri::command]
pub fn seed_dev_data(state: State<'_, AppState>) -> ApiResponse<bool> {
    if !cfg!(debug_assertions) {
        return ApiResponse::err("Not in development mode");
    }

    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = seed::seed_dev_data(&mut db, &state.paths.app_dir);
    to_api_response(result)
}

/// Clear dev seed data and reseed fresh.
///
/// Deletes existing dev campaign and creates fresh test data.
/// Only works in development mode.
#[tauri::command]
pub fn reseed_dev_data(state: State<'_, AppState>) -> ApiResponse<bool> {
    if !cfg!(debug_assertions) {
        return ApiResponse::err("Not in development mode");
    }

    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Clear existing data
    if let Err(e) = seed::clear_dev_seed_data(&mut db) {
        return ApiResponse::err(format!("Failed to clear: {}", e));
    }

    // Seed fresh data
    let result = seed::seed_dev_data(&mut db, &state.paths.app_dir);
    to_api_response(result)
}

/// Clear dev seed data without reseeding.
#[tauri::command]
pub fn clear_dev_data(state: State<'_, AppState>) -> ApiResponse<()> {
    if !cfg!(debug_assertions) {
        return ApiResponse::err("Not in development mode");
    }

    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = seed::clear_dev_seed_data(&mut db);
    to_api_response(result)
}
