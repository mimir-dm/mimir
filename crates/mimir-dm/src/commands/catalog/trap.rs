//! Database-backed trap catalog commands.
//!
//! Provides Tauri commands for searching and retrieving trap and hazard data
//! from the 5e catalog database. Used for dungeon design and encounter building.

use crate::state::AppState;
use mimir_dm_core::models::catalog::{CatalogTrap, TrapFilters, TrapOrHazard, TrapSummary};
use mimir_dm_core::services::TrapService;
use tauri::State;
use tracing::{debug, error, warn};

/// Search the trap catalog with optional filters.
///
/// Returns a list of trap summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `search` - Text to search in trap names (case-insensitive)
/// - `sources` - Filter by source books
/// - `categories` - Filter by trap category
/// - `trap_types` - Filter by trap type (mechanical, magical, etc.)
///
/// # Returns
/// List of `TrapSummary` objects containing basic trap information.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_traps(
    search: Option<String>,
    sources: Option<Vec<String>>,
    categories: Option<Vec<String>>,
    trap_types: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<Vec<TrapSummary>, String> {
    debug!("search_traps called with search: {:?}", search);

    let filters = TrapFilters {
        search,
        sources,
        categories,
        trap_types,
    };

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection error: {}", e)
    })?;

    let service = TrapService;
    service.search_traps(&mut conn, filters).map_err(|e| {
        error!("Failed to search traps: {}", e);
        format!("Search error: {}", e)
    })
}

/// Get complete trap details by name and source.
///
/// Retrieves the full trap record including trigger, effect, and countermeasures.
///
/// # Parameters
/// - `name` - Exact trap name (case-sensitive)
/// - `source` - Source book abbreviation (e.g., "DMG", "XGE")
///
/// # Returns
/// The complete `TrapOrHazard` object if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_trap_details(
    name: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<Option<TrapOrHazard>, String> {
    debug!("get_trap_details called for: {} from {}", name, source);

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection error: {}", e)
    })?;

    let service = TrapService;
    let catalog_trap = service
        .get_trap_details(&mut conn, name, source)
        .map_err(|e| {
            error!("Failed to get trap details: {}", e);
            format!("Database error: {}", e)
        })?;

    // Parse the full_trap_json to get the actual trap data with entries
    match catalog_trap {
        Some(trap) => {
            let parsed: TrapOrHazard = serde_json::from_str(&trap.full_trap_json).map_err(|e| {
                warn!("Failed to parse trap JSON for {}: {}", trap.name, e);
                format!("Failed to parse trap data: {}", e)
            })?;
            Ok(Some(parsed))
        }
        None => Ok(None),
    }
}

/// Get all unique source books containing traps.
///
/// Returns source book abbreviations for populating filter dropdowns.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_trap_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("get_trap_sources called");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection error: {}", e)
    })?;

    let service = TrapService;
    service.get_trap_sources(&mut conn).map_err(|e| {
        error!("Failed to get trap sources: {}", e);
        format!("Database error: {}", e)
    })
}

/// Get total number of traps in the catalog.
///
/// Returns the total count of all traps and hazards.
///
/// # Returns
/// Total trap count as a 64-bit integer.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_trap_count(state: State<'_, AppState>) -> Result<i64, String> {
    debug!("get_trap_count called");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection error: {}", e)
    })?;

    let service = TrapService;
    service.get_trap_count(&mut conn).map_err(|e| {
        error!("Failed to get trap count: {}", e);
        format!("Database error: {}", e)
    })
}

/// Get all unique trap types in the catalog.
///
/// Returns trap type categories for populating filter dropdowns.
/// Examples include mechanical, magical, etc.
///
/// # Returns
/// List of type names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_trap_types(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("get_trap_types called");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection error: {}", e)
    })?;

    let service = TrapService;
    service.get_trap_types(&mut conn).map_err(|e| {
        error!("Failed to get trap types: {}", e);
        format!("Database error: {}", e)
    })
}

/// Get all unique trap categories in the catalog.
///
/// Returns category names for populating filter dropdowns.
///
/// # Returns
/// List of category names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_trap_categories(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("get_trap_categories called");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection error: {}", e)
    })?;

    let service = TrapService;
    service.get_trap_categories(&mut conn).map_err(|e| {
        error!("Failed to get trap categories: {}", e);
        format!("Database error: {}", e)
    })
}
