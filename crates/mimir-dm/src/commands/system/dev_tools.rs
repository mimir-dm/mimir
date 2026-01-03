//! Development tools and commands.
//!
//! Provides Tauri commands for development-only functionality such as
//! test data management and debugging features.

use crate::state::AppState;
use crate::types::ApiError;
use crate::{
    embedded_test_book::{get_embedded_test_books, is_dev_build},
    types::ApiResponse,
};
use std::fs;
use tauri::State;
use tracing::info;

/// Check if the application is running in development mode.
///
/// Returns true if this is a development build.
///
/// # Returns
/// Boolean indicating development mode status.
#[tauri::command]
pub async fn is_dev_mode() -> Result<bool, String> {
    Ok(is_dev_build())
}

/// Remove all embedded development test books.
///
/// Deletes the test book directories created during development.
/// Only works when running in development mode.
///
/// # Returns
/// `ApiResponse` indicating success.
///
/// # Errors
/// Returns error response if not in development mode or deletion fails.
#[tauri::command]
pub async fn remove_dev_test_book(
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, String> {
    if !is_dev_build() {
        return Ok(ApiResponse::error("Not in development mode".to_string()));
    }

    let books_dir = state.paths.data_dir.join("books");
    let test_books = get_embedded_test_books();

    // Remove all test book directories
    for book in &test_books {
        let book_dir = books_dir.join(&book.name);
        if book_dir.exists() {
            fs::remove_dir_all(&book_dir)
                .map_err(|e| format!("Failed to remove test book {}: {}", book.name, e))?;
            info!("Removed dev test book: {}", book.name);
        }
    }

    Ok(ApiResponse::success(()))
}

/// Reseed development test data.
///
/// Deletes DB, re-runs migrations, and seeds fresh.
/// Only works in development mode.
#[tauri::command]
pub async fn reseed_dev_data(state: State<'_, AppState>) -> Result<ApiResponse<()>, ApiError> {
    use diesel::{prelude::*, SqliteConnection};
    use directories::UserDirs;
    use mimir_dm_core::{run_migrations, seed::seed_dev_data};

    if !is_dev_build() {
        return Ok(ApiResponse::error("Not in development mode".to_string()));
    }

    // Get paths
    let user_dirs = UserDirs::new().ok_or_else(|| ApiError::BadRequest("Cannot get user dirs".into()))?;
    let campaigns_dir = user_dirs
        .document_dir()
        .unwrap_or_else(|| user_dirs.home_dir())
        .join("Mimir Campaigns");

    let campaigns_path = campaigns_dir
        .to_str()
        .ok_or_else(|| ApiError::BadRequest("Invalid campaigns path".into()))?
        .to_string();
    let data_path = state
        .paths
        .data_dir
        .to_str()
        .ok_or_else(|| ApiError::BadRequest("Invalid data path".into()))?
        .to_string();
    let db_path = state.paths.data_dir.join("mimir.db");
    let db_url = format!("sqlite://{}", db_path.display());

    // 1. Delete campaigns directory
    if campaigns_dir.exists() {
        fs::remove_dir_all(&campaigns_dir)
            .map_err(|e| ApiError::Database(format!("Failed to remove campaigns dir: {}", e)))?;
        info!("Removed campaigns directory");
    }

    // 2. Delete database file
    if db_path.exists() {
        fs::remove_file(&db_path)
            .map_err(|e| ApiError::Database(format!("Failed to remove database: {}", e)))?;
        info!("Removed database file");
    }

    // 3. Create fresh connection, run migrations, seed templates, and seed dev data
    {
        let mut conn = SqliteConnection::establish(&db_url)
            .map_err(|e| ApiError::Database(format!("Failed to connect: {}", e)))?;

        run_migrations(&mut conn)
            .map_err(|e| ApiError::Database(format!("Failed to migrate: {}", e)))?;
        info!("Ran migrations");

        // Seed templates (required for campaign/module document creation)
        mimir_dm_core::seed::template_seeder::seed_templates(&mut conn)
            .map_err(|e| ApiError::Database(format!("Failed to seed templates: {}", e)))?;
        info!("Seeded templates");

        seed_dev_data(&mut conn, &campaigns_path, &data_path)
            .map_err(|e| ApiError::Database(format!("Failed to seed: {}", e)))?;
        info!("Reseeded dev data");
    }

    Ok(ApiResponse::success(()))
}
