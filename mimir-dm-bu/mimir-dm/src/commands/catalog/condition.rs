//! Database-backed condition catalog commands.
//!
//! Provides Tauri commands for searching and retrieving condition and disease data
//! from the 5e catalog database. Includes exhaustion, blinded, poisoned, etc.

use crate::state::AppState;
use mimir_dm_core::models::catalog::{ConditionFilters, ConditionOrDisease, ConditionSummary};
use mimir_dm_core::services::ConditionService;
use tauri::State;
use tracing::debug;

/// Search the condition/disease catalog with optional filters.
///
/// Returns a list of condition summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `name` - Filter by exact condition name
/// - `search` - Text to search in condition names/descriptions (case-insensitive)
/// - `item_types` - Filter by type (e.g., `["condition", "disease"]`)
/// - `sources` - Filter by source books
///
/// # Returns
/// List of `ConditionSummary` objects containing basic condition information.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_conditions(
    state: State<'_, AppState>,
    name: Option<String>,
    search: Option<String>,
    item_types: Option<Vec<String>>,
    sources: Option<Vec<String>>,
) -> Result<Vec<ConditionSummary>, String> {
    debug!(
        "Database condition search - name: {:?}, search: {:?}, item_types: {:?}, sources: {:?}",
        name, search, item_types, sources
    );

    let filters = ConditionFilters {
        name,
        search,
        item_types,
        sources,
    };

    state.with_connection("condition search", |conn| {
        ConditionService::new(conn).search_conditions(filters)
    })
}

/// Get complete condition/disease details by database ID.
///
/// Retrieves the full condition record including effects and duration.
///
/// # Parameters
/// - `condition_id` - Database ID of the condition
///
/// # Returns
/// The complete `ConditionOrDisease` object if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_condition(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> Result<Option<ConditionOrDisease>, String> {
    debug!("Getting condition by name: {} and source: {}", name, source);

    state.with_connection("get condition", |conn| {
        ConditionService::new(conn).get_condition_by_name_and_source(&name, &source)
    })
}

/// Get complete condition/disease details by database ID.
///
/// Retrieves the full condition record including effects and duration.
///
/// # Parameters
/// - `condition_id` - Database ID of the condition
///
/// # Returns
/// The complete `ConditionOrDisease` object if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_condition_by_id(
    state: State<'_, AppState>,
    condition_id: i32,
) -> Result<Option<ConditionOrDisease>, String> {
    debug!("Getting condition by ID: {}", condition_id);

    state.with_connection("get condition by id", |conn| {
        ConditionService::new(conn).get_condition_by_id(condition_id)
    })
}

/// Get all unique condition types in the catalog.
///
/// Returns type categories (condition vs disease) present in the catalog.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of type names (e.g., `["condition", "disease"]`).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_condition_item_types(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting condition item types");

    state.with_connection("get condition item types", |conn| {
        ConditionService::new(conn).get_item_types()
    })
}

/// Get all unique source books containing conditions.
///
/// Returns source book abbreviations that contain at least one condition.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_condition_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting condition sources");

    state.with_connection("get condition sources", |conn| {
        ConditionService::new(conn).get_condition_sources()
    })
}

/// Get total number of conditions in the catalog.
///
/// Returns the total count of all conditions and diseases.
///
/// # Returns
/// Total condition count as a 64-bit integer.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_condition_count(state: State<'_, AppState>) -> Result<i64, String> {
    debug!("Getting condition count");

    state.with_connection("get condition count", |conn| {
        ConditionService::new(conn).get_condition_count()
    })
}
