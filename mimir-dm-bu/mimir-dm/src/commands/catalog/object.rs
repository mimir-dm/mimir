//! Database-backed object catalog commands.
//!
//! Provides Tauri commands for searching and retrieving object data
//! from the 5e catalog database. Includes mundane and hazardous objects.

use crate::state::AppState;
use mimir_dm_core::models::catalog::ObjectFilters;
use mimir_dm_core::services::ObjectService;
use tauri::State;
use tracing::error;

/// Search the object catalog with optional filters.
///
/// Returns a list of object summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `search` - Text to search in object names (case-insensitive)
/// - `sources` - Filter by source books
/// - `object_types` - Filter by object type
/// - `sizes` - Filter by size category
///
/// # Returns
/// List of object summaries as JSON values.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_objects(
    search: Option<String>,
    sources: Option<Vec<String>>,
    object_types: Option<Vec<String>>,
    sizes: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let filters = ObjectFilters {
        search_pattern: search,
        sources,
        object_types,
        sizes,
    };

    let results = ObjectService::search_objects(&mut conn, filters)
        .map_err(|e| format!("Failed to search objects: {}", e))?;

    // Convert ObjectSummary to JSON values for frontend compatibility
    let json_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|obj| serde_json::to_value(&obj).unwrap_or_default())
        .collect();

    Ok(json_results)
}

/// Get complete object details by name and source.
///
/// Retrieves the full object data as stored JSON.
///
/// # Parameters
/// - `name` - Exact object name (case-sensitive)
/// - `source` - Source book abbreviation
///
/// # Returns
/// The complete object JSON string if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_object_details(
    name: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    ObjectService::get_object_details(&mut conn, &name, &source)
        .map_err(|e| format!("Failed to get object details: {}", e))
}

/// Get all unique source books containing objects.
///
/// Returns source book abbreviations for populating filter dropdowns.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_object_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    ObjectService::get_object_sources(&mut conn)
        .map_err(|e| format!("Failed to get object sources: {}", e))
}

/// Get total number of objects in the catalog.
///
/// Returns the total count of all objects across all source books.
///
/// # Returns
/// Total object count as a 64-bit integer.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_object_count(state: State<'_, AppState>) -> Result<i64, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    ObjectService::get_object_count(&mut conn)
        .map_err(|e| format!("Failed to get object count: {}", e))
}

/// Get all unique object types in the catalog.
///
/// Returns type names for populating filter dropdowns.
///
/// # Returns
/// List of type names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_object_types(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    ObjectService::get_object_types(&mut conn)
        .map_err(|e| format!("Failed to get object types: {}", e))
}

/// Get all unique object sizes in the catalog.
///
/// Returns size categories for populating filter dropdowns.
/// Uses standard D&D sizes (Tiny, Small, Medium, Large, etc.).
///
/// # Returns
/// List of size names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_object_sizes(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    ObjectService::get_object_sizes(&mut conn)
        .map_err(|e| format!("Failed to get object sizes: {}", e))
}
