//! Database-backed feat catalog commands.
//!
//! Provides Tauri commands for searching and retrieving feat data
//! from the 5e catalog database. Used for character creation and feat selection.

use crate::state::AppState;
use mimir_dm_core::models::catalog::FeatFilters;
use mimir_dm_core::services::FeatService;
use tauri::State;

/// Search the feat catalog with optional filters.
///
/// Returns a list of feat summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `query` - Text to search in feat names (case-insensitive)
/// - `sources` - Filter by source books
/// - `has_prerequisites` - Filter for feats with/without prerequisites
///
/// # Returns
/// List of feat objects as JSON values.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_feats(
    query: Option<String>,
    sources: Option<Vec<String>>,
    has_prerequisites: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let filters = FeatFilters {
        search_pattern: query,
        sources,
        has_prerequisites,
    };

    let feats = state.with_connection("feat search", |conn| {
        FeatService::new(conn).search_feats(filters)
    })?;

    // Convert to JSON for frontend
    let json_feats: Vec<serde_json::Value> = feats
        .into_iter()
        .map(|feat| serde_json::to_value(feat).unwrap_or(serde_json::Value::Null))
        .collect();

    Ok(json_feats)
}

/// Get complete feat details by name and source.
///
/// Retrieves the full feat data including description, prerequisites, and benefits.
///
/// # Parameters
/// - `name` - Exact feat name (case-sensitive)
/// - `source` - Source book abbreviation (e.g., "PHB", "XGE")
///
/// # Returns
/// The complete feat data as a JSON value.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_feat_details(
    name: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let feat = state.with_connection("get feat details", |conn| {
        FeatService::new(conn).get_feat_by_name_and_source(&name, &source)
    })?
    .ok_or_else(|| format!("Feat '{}' not found in source '{}'", name, source))?;

    // Convert to JSON for frontend
    serde_json::to_value(feat).map_err(|e| format!("Failed to serialize feat: {}", e))
}

/// Get all unique source books containing feats.
///
/// Returns source book abbreviations that contain at least one feat.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_feat_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.with_connection("get feat sources", |conn| {
        FeatService::new(conn).get_feat_sources()
    })
}

/// Get total number of feats in the catalog.
///
/// Returns the total count of all feats across all source books.
///
/// # Returns
/// Total feat count as a 64-bit integer.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_feat_count(state: State<'_, AppState>) -> Result<i64, String> {
    state.with_connection("get feat count", |conn| {
        FeatService::new(conn).get_feat_count()
    })
}
