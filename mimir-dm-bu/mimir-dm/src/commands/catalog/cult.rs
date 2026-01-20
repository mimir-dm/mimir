//! Database-backed cult and boon catalog commands.
//!
//! Provides Tauri commands for searching and retrieving cult and demonic boon data
//! from the 5e catalog database. Used for villain creation and campaign planning.

use crate::state::AppState;
use mimir_dm_core::models::catalog::cult::{CatalogCult, CultBoonSummary, CultFilters};
use mimir_dm_core::services::CultService;
use tauri::State;
use tracing::{debug, error};

/// Search the cult catalog with optional filters.
///
/// Returns a list of cult and boon summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `name` - Filter by exact cult/boon name
/// - `sources` - Filter by source books
/// - `categories` - Filter by category
/// - `cult_types` - Filter by cult type
///
/// # Returns
/// List of `CultBoonSummary` objects containing basic cult information.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_cults(
    name: Option<String>,
    sources: Option<Vec<String>>,
    categories: Option<Vec<String>>,
    cult_types: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<Vec<CultBoonSummary>, String> {
    debug!("search_cults called with name: {:?}", name);

    let filters = CultFilters {
        name,
        source: sources,
        category: categories,
        cult_type: cult_types,
    };

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection error: {}", e)
    })?;

    let service = CultService;
    service.search_cults(&mut conn, filters).map_err(|e| {
        error!("Failed to search cults: {}", e);
        format!("Search error: {}", e)
    })
}

/// Get complete cult details by name and source.
///
/// Retrieves the full cult record including goals, traits, and boons.
///
/// # Parameters
/// - `name` - Exact cult name (case-sensitive)
/// - `source` - Source book abbreviation
///
/// # Returns
/// The complete `CatalogCult` object if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_cult_details(
    name: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<Option<CatalogCult>, String> {
    debug!("get_cult_details called for: {} from {}", name, source);

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection error: {}", e)
    })?;

    let service = CultService;
    service
        .get_cult_details(&mut conn, name, source)
        .map_err(|e| {
            error!("Failed to get cult details: {}", e);
            format!("Database error: {}", e)
        })
}

/// Get all unique source books containing cults.
///
/// Returns source book abbreviations for populating filter dropdowns.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_cult_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("get_cult_sources called");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection error: {}", e)
    })?;

    let service = CultService;
    service.get_cult_sources(&mut conn).map_err(|e| {
        error!("Failed to get cult sources: {}", e);
        format!("Database error: {}", e)
    })
}

/// Get total number of cults and boons in the catalog.
///
/// Returns the total count of all cult-related entries.
///
/// # Returns
/// Total cult count as a 64-bit integer.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_cult_count(state: State<'_, AppState>) -> Result<i64, String> {
    debug!("get_cult_count called");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection error: {}", e)
    })?;

    let service = CultService;
    service.get_cult_count(&mut conn).map_err(|e| {
        error!("Failed to get cult count: {}", e);
        format!("Database error: {}", e)
    })
}

/// Get all unique cult types in the catalog.
///
/// Returns cult type categories for populating filter dropdowns.
///
/// # Returns
/// List of type names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_cult_types(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("get_cult_types called");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection error: {}", e)
    })?;

    let service = CultService;
    service.get_cult_types(&mut conn).map_err(|e| {
        error!("Failed to get cult types: {}", e);
        format!("Database error: {}", e)
    })
}

/// Get all unique cult categories in the catalog.
///
/// Returns category names for populating filter dropdowns.
///
/// # Returns
/// List of category names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_cult_categories(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("get_cult_categories called");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection error: {}", e)
    })?;

    let service = CultService;
    service.get_cult_categories(&mut conn).map_err(|e| {
        error!("Failed to get cult categories: {}", e);
        format!("Database error: {}", e)
    })
}
