//! Database-backed background catalog commands.
//!
//! Provides Tauri commands for searching and retrieving character background data
//! from the 5e catalog database. Used for character creation.
//!
//! This module uses the `catalog_commands!` macro to generate the standard
//! catalog commands (search, get_details, get_sources) with consistent
//! error handling and logging.

use mimir_dm_core::models::catalog::{BackgroundFilters, BackgroundSummary, CatalogBackground};
use mimir_dm_core::services::BackgroundService;

// Generate standard catalog commands using the macro
crate::catalog_commands!(
    entity: background,
    service: BackgroundService,
    filters: BackgroundFilters,
    summary: BackgroundSummary,
    full: CatalogBackground
);

// Additional background-specific commands

use crate::state::AppState;
use tauri::State;
use tracing::error;

/// Get total number of backgrounds in the catalog.
///
/// Returns the total count of all backgrounds across all source books.
///
/// # Returns
/// Total background count as a 64-bit integer.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_background_count(state: State<'_, AppState>) -> Result<i64, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = BackgroundService::new(&mut conn);
    service.get_background_count().map_err(|e| e.to_string())
}
