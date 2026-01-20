//! Database-backed psionic catalog commands.
//!
//! Provides Tauri commands for searching and retrieving psionic power data
//! from the 5e catalog database. Includes disciplines and talents from UA content.

use crate::state::AppState;
use mimir_dm_core::models::catalog::PsionicFilters;
use mimir_dm_core::services::PsionicService;
use tauri::State;

/// Search the psionic catalog with optional filters.
///
/// Returns a list of psionic summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `query` - Text to search in psionic names (case-insensitive)
/// - `psionic_types` - Filter by type ("D" for Discipline, "T" for Talent)
/// - `orders` - Filter by psionic order (e.g., "Avatar", "Awakened", "Immortal")
/// - `sources` - Filter by source books
///
/// # Returns
/// List of psionic objects as JSON values.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_psionics(
    query: Option<String>,
    psionic_types: Option<Vec<String>>, // "D", "T"
    orders: Option<Vec<String>>,        // Avatar, Awakened, etc.
    sources: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let filters = PsionicFilters {
        name: query,
        psionic_types,
        orders,
        sources,
    };

    let psionics = PsionicService::search_psionics(&mut conn, filters)
        .map_err(|e| format!("Failed to search psionics: {}", e))?;

    // Convert to JSON for frontend
    let json_psionics: Vec<serde_json::Value> = psionics
        .into_iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect();

    Ok(json_psionics)
}

/// Get complete psionic details by name and source.
///
/// Retrieves the full psionic record including effects and focus benefits.
///
/// # Parameters
/// - `name` - Exact psionic name (case-sensitive)
/// - `source` - Source book abbreviation
///
/// # Returns
/// The complete psionic data as JSON if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_psionic_details(
    name: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<Option<serde_json::Value>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let psionic = PsionicService::get_psionic_by_name_and_source(&mut conn, &name, &source)
        .map_err(|e| format!("Failed to get psionic details: {}", e))?;

    Ok(psionic.map(|p| serde_json::to_value(p).unwrap_or_default()))
}

/// Get all unique psionic types in the catalog.
///
/// Returns type codes for populating filter dropdowns.
/// Types are "D" (Discipline) and "T" (Talent).
///
/// # Returns
/// List of type codes.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_psionic_types(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    PsionicService::get_all_psionic_types(&mut conn)
        .map_err(|e| format!("Failed to get psionic types: {}", e))
}

/// Get all unique psionic orders in the catalog.
///
/// Returns order names for populating filter dropdowns.
/// Examples include Avatar, Awakened, Immortal, Nomad, Wu Jen.
///
/// # Returns
/// List of order names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_psionic_orders(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    PsionicService::get_all_psionic_orders(&mut conn)
        .map_err(|e| format!("Failed to get psionic orders: {}", e))
}

/// Get all unique source books containing psionics.
///
/// Returns source book abbreviations for populating filter dropdowns.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_psionic_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    PsionicService::get_all_psionic_sources(&mut conn)
        .map_err(|e| format!("Failed to get psionic sources: {}", e))
}
