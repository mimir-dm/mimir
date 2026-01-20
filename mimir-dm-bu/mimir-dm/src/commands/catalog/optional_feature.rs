//! Database-backed optional feature catalog commands.
//!
//! Provides Tauri commands for searching and retrieving optional class features
//! from the 5e catalog database. Includes fighting styles, invocations, maneuvers, etc.

use crate::state::AppState;
use mimir_dm_core::models::catalog::optionalfeature::{
    OptionalFeature, OptionalFeatureFilters, OptionalFeatureSummary,
};
use mimir_dm_core::services::OptionalFeatureService;
use tauri::State;
use tracing::{debug, error, info};

/// Search the optional feature catalog with optional filters.
///
/// Returns a list of optional feature summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `name` - Text to search in feature names (case-insensitive)
/// - `feature_types` - Filter by type (e.g., "Fighting Style", "Eldritch Invocation")
/// - `sources` - Filter by source books
/// - `grants_spells` - Filter for features that grant spells
///
/// # Returns
/// List of `OptionalFeatureSummary` objects containing basic feature information.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_optional_features(
    name: Option<String>,
    feature_types: Option<Vec<String>>,
    sources: Option<Vec<String>>,
    grants_spells: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<OptionalFeatureSummary>, String> {
    debug!("Searching optional features with name: {:?}, feature_types: {:?}, sources: {:?}, grants_spells: {:?}",
           name, feature_types, sources, grants_spells);

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let filters = OptionalFeatureFilters {
        name,
        feature_types,
        sources,
        grants_spells,
    };

    let mut service = OptionalFeatureService::new(&mut conn);
    let results = service
        .search_optional_features(filters)
        .map_err(|e| format!("Database query failed: {}", e))?;

    info!("Found {} optional features", results.len());
    Ok(results)
}

/// Get optional feature by database ID.
///
/// Retrieves a feature record by its internal database identifier.
///
/// # Parameters
/// - `id` - Database ID of the optional feature
///
/// # Returns
/// The complete `OptionalFeature` object.
///
/// # Errors
/// Returns an error string if the feature is not found or database fails.
#[tauri::command]
pub async fn get_optional_feature(
    id: i32,
    state: State<'_, AppState>,
) -> Result<OptionalFeature, String> {
    debug!("Getting optional feature details for ID: {}", id);

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = OptionalFeatureService::new(&mut conn);
    let result = service
        .get_optional_feature_by_id(id)
        .map_err(|e| format!("Database query failed: {}", e))?;

    result.ok_or_else(|| format!("Optional feature with ID {} not found", id))
}

/// Get complete optional feature details by name and source.
///
/// Retrieves the full feature record including prerequisites and effects.
///
/// # Parameters
/// - `name` - Exact feature name (case-sensitive)
/// - `source` - Source book abbreviation (e.g., "PHB", "TCE")
///
/// # Returns
/// The complete `OptionalFeature` object.
///
/// # Errors
/// Returns an error string if the feature is not found or database fails.
#[tauri::command]
pub async fn get_optional_feature_details(
    name: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<OptionalFeature, String> {
    debug!(
        "Getting optional feature details for name: {}, source: {}",
        name, source
    );

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = OptionalFeatureService::new(&mut conn);
    let result = service
        .get_optional_feature_by_name_and_source(&name, &source)
        .map_err(|e| format!("Database query failed: {}", e))?;

    result.ok_or_else(|| {
        format!(
            "Optional feature '{}' from source '{}' not found",
            name, source
        )
    })
}

/// Get all unique feature types in the optional feature catalog.
///
/// Returns feature type categories for populating filter dropdowns.
/// Examples include Fighting Style, Eldritch Invocation, Metamagic, etc.
///
/// # Returns
/// List of feature type names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_optional_feature_types(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting all optional feature types");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = OptionalFeatureService::new(&mut conn);
    let types = service
        .get_optional_feature_types()
        .map_err(|e| format!("Database query failed: {}", e))?;

    info!("Found {} feature types", types.len());
    Ok(types)
}

/// Get all unique source books containing optional features.
///
/// Returns source book abbreviations for populating filter dropdowns.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_optional_feature_sources(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    debug!("Getting all optional feature sources");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = OptionalFeatureService::new(&mut conn);
    let sources = service
        .get_optional_feature_sources()
        .map_err(|e| format!("Database query failed: {}", e))?;

    info!("Found {} sources", sources.len());
    Ok(sources)
}
