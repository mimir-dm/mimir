//! Development tools and commands.
//!
//! Provides Tauri commands for development-only functionality such as
//! test data management and debugging features.

use crate::app_init::AppPaths;
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
/// Truncates all tables and reseeds fresh data using the existing connection pool.
/// Only works in development mode.
#[tauri::command]
pub async fn reseed_dev_data(state: State<'_, AppState>) -> Result<ApiResponse<()>, ApiError> {
    use diesel::prelude::*;
    use directories::UserDirs;
    use mimir_dm_core::seed::seed_dev_data;

    if !is_dev_build() {
        return Ok(ApiResponse::error("Not in development mode".to_string()));
    }

    // Get paths - use correct folder name based on dev/release mode
    let user_dirs = UserDirs::new().ok_or_else(|| ApiError::BadRequest("Cannot get user dirs".into()))?;
    let campaigns_dir = user_dirs
        .document_dir()
        .unwrap_or_else(|| user_dirs.home_dir())
        .join(AppPaths::campaigns_folder_name());

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

    // 1. Delete campaigns directory on disk
    if campaigns_dir.exists() {
        fs::remove_dir_all(&campaigns_dir)
            .map_err(|e| ApiError::Database(format!("Failed to remove campaigns dir: {}", e)))?;
        info!("Removed campaigns directory");
    }

    // 2. Use the existing connection pool to truncate and reseed
    let mut conn = state.db.get_connection()
        .map_err(|e| ApiError::Database(format!("Failed to get connection: {}", e)))?;

    // Truncate all data tables (in reverse dependency order to avoid FK violations)
    // Note: We don't touch __diesel_schema_migrations or template_documents
    let tables_to_truncate = [
        "session_notes",
        "session_encounters",
        "sessions",
        "documents",
        "workflow_cards",
        "modules",
        "campaign_characters",
        "campaigns",
        "characters",
        "players",
    ];

    for table in tables_to_truncate {
        // Ignore errors for tables that don't exist
        if let Err(e) = diesel::sql_query(format!("DELETE FROM {}", table)).execute(&mut *conn) {
            info!("Skipping table {} (may not exist): {}", table, e);
        }
        // Reset auto-increment counter
        diesel::sql_query(format!("DELETE FROM sqlite_sequence WHERE name='{}'", table))
            .execute(&mut *conn)
            .ok(); // Ignore errors - table might not have auto-increment
    }
    info!("Truncated data tables");

    // 3. Reseed dev data
    seed_dev_data(&mut conn, &campaigns_path, &data_path)
        .map_err(|e| ApiError::Database(format!("Failed to seed: {}", e)))?;
    info!("Reseeded dev data");

    Ok(ApiResponse::success(()))
}
