//! Database-backed action catalog commands.
//!
//! Provides Tauri commands for searching and retrieving combat action data
//! from the 5e catalog database. Actions include standard combat options
//! like Attack, Dash, Dodge, etc.

use crate::state::AppState;
use mimir_dm_core::models::catalog::{Action, ActionFilters, ActionSummary};
use mimir_dm_core::services::ActionService;
use tauri::State;
use tracing::debug;

/// Search the action catalog with optional filters.
///
/// Returns a list of action summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `name` - Text to search in action names (case-insensitive)
/// - `search` - Text to search in action descriptions
/// - `time_types` - Filter by action time (e.g., `["Action", "Bonus Action"]`)
/// - `sources` - Filter by source books
///
/// # Returns
/// List of `ActionSummary` objects containing basic action information.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_actions(
    name: Option<String>,
    search: Option<String>,
    time_types: Option<Vec<String>>,
    sources: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<Vec<ActionSummary>, String> {
    debug!(
        "Database action search - name: {:?}, search: {:?}, time_types: {:?}, sources: {:?}",
        name, search, time_types, sources
    );

    let filters = ActionFilters {
        name,
        search,
        time_types,
        sources,
    };

    state.with_connection("action search", |conn| {
        ActionService::new(conn).search_actions(filters)
    })
}

/// Get complete action details by name and source.
///
/// Retrieves the full action record including description and rules text.
///
/// # Parameters
/// - `name` - Name of the action
/// - `source` - Source book abbreviation
///
/// # Returns
/// The complete `Action` object if found, or `None` if no match exists.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_action(
    name: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<Option<Action>, String> {
    debug!("Getting action by name: {} and source: {}", name, source);

    state.with_connection("get action", |conn| {
        ActionService::new(conn).get_action_by_name_and_source(&name, &source)
    })
}

/// Get complete action details by database ID.
///
/// Retrieves the full action record including description and rules text.
///
/// # Parameters
/// - `action_id` - Database ID of the action
///
/// # Returns
/// The complete `Action` object if found, or `None` if no match exists.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_action_by_id(
    action_id: i32,
    state: State<'_, AppState>,
) -> Result<Option<Action>, String> {
    debug!("Getting action by ID: {}", action_id);

    state.with_connection("get action by id", |conn| {
        ActionService::new(conn).get_action_by_id(action_id)
    })
}

/// Get all unique action time types in the catalog.
///
/// Returns time type values present in the action catalog.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of time types (e.g., `["Action", "Bonus Action", "Reaction"]`).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_action_time_types(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting action time types");

    state.with_connection("get action time types", |conn| {
        ActionService::new(conn).get_time_types()
    })
}

/// Get all unique source books containing actions.
///
/// Returns source book abbreviations that contain at least one action.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_action_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting action sources");

    state.with_connection("get action sources", |conn| {
        ActionService::new(conn).get_action_sources()
    })
}

/// Get total number of actions in the catalog.
///
/// Returns the total count of all actions across all source books.
///
/// # Returns
/// Total action count as a 64-bit integer.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_action_count(state: State<'_, AppState>) -> Result<i64, String> {
    debug!("Getting action count");

    state.with_connection("get action count", |conn| {
        ActionService::new(conn).get_action_count()
    })
}
