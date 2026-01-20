//! Database-backed deity catalog commands.
//!
//! Provides Tauri commands for searching and retrieving deity data
//! from the 5e catalog database. Used for world-building and cleric domain selection.

use crate::state::AppState;
use mimir_dm_core::models::catalog::deity::{Deity, DeityFilters, DeitySummary};
use mimir_dm_core::services::DeityService;
use tauri::State;
use tracing::{debug, error};

/// Search the deity catalog with optional filters.
///
/// Returns a list of deity summaries matching the provided criteria.
/// All filter parameters within the `DeityFilters` struct are optional.
///
/// # Parameters
/// - `filters` - Filter criteria including pantheon, domain, alignment, and text search
///
/// # Returns
/// List of `DeitySummary` objects containing basic deity information.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_deities(
    filters: DeityFilters,
    state: State<'_, AppState>,
) -> Result<Vec<DeitySummary>, String> {
    debug!("Searching deities with filters: {:?}", filters);

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = DeityService::new(&mut conn);
    service
        .search_deities(filters)
        .map_err(|e| format!("Database query failed: {}", e))
}

/// Get complete deity details by name and source.
///
/// Retrieves the full deity record including domains, symbol, and worshippers.
///
/// # Parameters
/// - `deity_name` - Exact deity name (case-sensitive)
/// - `deity_source` - Source book abbreviation (e.g., "PHB", "SCAG")
///
/// # Returns
/// The complete `Deity` object if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_deity_details(
    deity_name: String,
    deity_source: String,
    state: State<'_, AppState>,
) -> Result<Option<Deity>, String> {
    debug!(
        "Getting deity details for name: {}, source: {}",
        deity_name, deity_source
    );

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = DeityService::new(&mut conn);
    service
        .get_deity_by_name_and_source(&deity_name, &deity_source)
        .map_err(|e| format!("Database query failed: {}", e))
}

/// Get all unique pantheons in the deity catalog.
///
/// Returns pantheon names for populating filter dropdowns in the UI.
/// Examples include Greek, Norse, Egyptian, Forgotten Realms, etc.
///
/// # Returns
/// List of pantheon names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_deity_pantheons(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting all deity pantheons");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = DeityService::new(&mut conn);
    service
        .get_all_pantheons()
        .map_err(|e| format!("Database query failed: {}", e))
}

/// Get all unique domains in the deity catalog.
///
/// Returns cleric domain names for populating filter dropdowns.
/// Examples include Life, Death, War, Knowledge, etc.
///
/// # Returns
/// List of domain names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_deity_domains(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting all deity domains");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = DeityService::new(&mut conn);
    service
        .get_all_domains()
        .map_err(|e| format!("Database query failed: {}", e))
}

/// Get all unique alignments in the deity catalog.
///
/// Returns alignment strings for populating filter dropdowns.
/// Uses standard D&D alignments (LG, NG, CG, LN, N, CN, LE, NE, CE).
///
/// # Returns
/// List of alignment strings.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_deity_alignments(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting all deity alignments");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = DeityService::new(&mut conn);
    service
        .get_all_alignments()
        .map_err(|e| format!("Database query failed: {}", e))
}

/// Get deity count statistics grouped by source book.
///
/// Returns a breakdown of how many deities are in each source book.
/// Used for displaying catalog statistics in the UI.
///
/// # Returns
/// List of tuples containing (source abbreviation, count).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_deity_statistics(
    state: State<'_, AppState>,
) -> Result<Vec<(String, i64)>, String> {
    debug!("Getting deity statistics");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = DeityService::new(&mut conn);
    service
        .get_deity_count_by_source()
        .map_err(|e| format!("Database query failed: {}", e))
}
