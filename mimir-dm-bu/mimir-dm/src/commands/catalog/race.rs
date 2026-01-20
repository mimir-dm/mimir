//! Database-backed race catalog commands.
//!
//! Provides Tauri commands for searching and retrieving playable race data
//! from the 5e catalog database. Used for character creation and race browsing.

use crate::state::AppState;
use mimir_dm_core::models::catalog::RaceFilters;
use mimir_dm_core::services::RaceService;
use tauri::State;

/// Search the race catalog with optional filters.
///
/// Returns a list of race summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `search` - Text to search in race names (case-insensitive)
/// - `sources` - Filter by source books
/// - `sizes` - Filter by creature size (e.g., `["Small", "Medium"]`)
/// - `has_darkvision` - Filter for races with darkvision
/// - `has_flight` - Filter for races with innate flight
///
/// # Returns
/// List of race summary objects as JSON values.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_races(
    search: Option<String>,
    sources: Option<Vec<String>>,
    sizes: Option<Vec<String>>,
    has_darkvision: Option<bool>,
    has_flight: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let filters = RaceFilters {
        search_pattern: search,
        sources,
        sizes,
        has_darkvision,
        has_flight,
    };

    let results = state.with_connection("race search", |conn| {
        RaceService::search_races(conn, filters)
    })?;

    // Convert RaceSummary to JSON values for frontend compatibility
    let json_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|race| serde_json::to_value(&race).unwrap_or_default())
        .collect();

    Ok(json_results)
}

/// Get complete race details by name and source.
///
/// Retrieves the full race data including traits, ability bonuses, and features.
///
/// # Parameters
/// - `name` - Exact race name (case-sensitive)
/// - `source` - Source book abbreviation (e.g., "PHB", "VGM")
///
/// # Returns
/// The race data as a JSON string if found, or `None` if no match exists.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_race_details(
    name: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    state.with_connection("get race details", |conn| {
        RaceService::get_race_details(conn, &name, &source)
    })
}

/// Get all unique source books containing races.
///
/// Returns source book abbreviations that contain at least one race.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_race_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.with_connection("get race sources", |conn| {
        RaceService::get_race_sources(conn)
    })
}

/// Get total number of races in the catalog.
///
/// Returns the total count of all races across all source books.
///
/// # Returns
/// Total race count as a 64-bit integer.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_race_count(state: State<'_, AppState>) -> Result<i64, String> {
    state.with_connection("get race count", |conn| {
        RaceService::get_race_count(conn)
    })
}

/// Get all unique race sizes in the catalog.
///
/// Returns size categories present in the race catalog.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of size names (e.g., `["Small", "Medium"]`).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_race_sizes(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.with_connection("get race sizes", |conn| {
        RaceService::get_race_sizes(conn)
    })
}
