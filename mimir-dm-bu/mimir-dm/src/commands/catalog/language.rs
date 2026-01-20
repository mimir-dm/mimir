//! Database-backed language catalog commands.
//!
//! Provides Tauri commands for searching and retrieving language data
//! from the 5e catalog database. Used for character creation and world-building.

use crate::state::AppState;
use mimir_dm_core::models::catalog::{Language, LanguageFilters, LanguageSummary};
use mimir_dm_core::services::LanguageService;
use tauri::State;
use tracing::{debug, error, info};

/// Search the language catalog with optional filters.
///
/// Returns a list of language summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `query` - Text to search in language names (case-insensitive)
/// - `language_types` - Filter by type (e.g., `["Standard", "Exotic"]`)
/// - `scripts` - Filter by script used (e.g., `["Common", "Elvish"]`)
/// - `sources` - Filter by source books
///
/// # Returns
/// List of `LanguageSummary` objects containing basic language information.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_languages(
    state: State<'_, AppState>,
    query: Option<String>,
    language_types: Option<Vec<String>>,
    scripts: Option<Vec<String>>,
    sources: Option<Vec<String>>,
) -> Result<Vec<LanguageSummary>, String> {
    debug!(
        "Database language search - query: {:?}, types: {:?}, scripts: {:?}, sources: {:?}",
        query, language_types, scripts, sources
    );

    let filters = LanguageFilters {
        name: None,
        search: query,
        language_types,
        scripts,
        sources,
    };

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Database connection error during language search: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    match LanguageService::search_languages(&mut conn, filters) {
        Ok(languages) => {
            info!("Found {} languages in database search", languages.len());
            Ok(languages)
        }
        Err(e) => {
            error!("Database language search failed: {}", e);
            Err(format!("Failed to search languages: {}", e))
        }
    }
}

/// Get complete language details by name and source.
///
/// Retrieves the full language record including typical speakers and script.
///
/// # Parameters
/// - `name` - Exact language name (case-sensitive)
/// - `source` - Source book abbreviation
///
/// # Returns
/// The complete `Language` object.
///
/// # Errors
/// Returns an error string if the language is not found or database fails.
#[tauri::command]
pub async fn get_language_details(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> Result<Language, String> {
    debug!("Getting language details for '{}' from '{}'", name, source);

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Database connection error getting language: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    match LanguageService::get_language_by_name_and_source(&mut conn, &name, &source) {
        Ok(Some(language)) => Ok(language),
        Ok(None) => Err(format!("Language not found: {} from {}", name, source)),
        Err(e) => {
            error!("Failed to get language: {}", e);
            Err(format!("Failed to get language: {}", e))
        }
    }
}

/// Get all unique language types in the catalog.
///
/// Returns type categories present in the language catalog.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of type names (e.g., `["Standard", "Exotic"]`).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_language_types(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting language types");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Database connection error getting language types: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    match LanguageService::get_language_types(&mut conn) {
        Ok(types) => Ok(types),
        Err(e) => {
            error!("Failed to get language types: {}", e);
            Err(format!("Failed to get language types: {}", e))
        }
    }
}

/// Get all unique scripts in the language catalog.
///
/// Returns script names used by languages in the catalog.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of script names (e.g., `["Common", "Elvish", "Dwarvish"]`).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_language_scripts(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting language scripts");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Database connection error getting scripts: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    match LanguageService::get_scripts(&mut conn) {
        Ok(scripts) => Ok(scripts),
        Err(e) => {
            error!("Failed to get scripts: {}", e);
            Err(format!("Failed to get scripts: {}", e))
        }
    }
}

/// Get all unique source books containing languages.
///
/// Returns source book abbreviations that contain at least one language.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_language_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting language sources");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Database connection error getting sources: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    match LanguageService::get_sources(&mut conn) {
        Ok(sources) => Ok(sources),
        Err(e) => {
            error!("Failed to get sources: {}", e);
            Err(format!("Failed to get sources: {}", e))
        }
    }
}

/// Get total number of languages in the catalog.
///
/// Returns the total count of all languages.
///
/// # Returns
/// Total language count as a 64-bit integer.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_language_count(state: State<'_, AppState>) -> Result<i64, String> {
    debug!("Getting language count");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Database connection error getting language count: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    match LanguageService::get_language_count(&mut conn) {
        Ok(count) => Ok(count),
        Err(e) => {
            error!("Failed to get language count: {}", e);
            Err(format!("Failed to get language count: {}", e))
        }
    }
}
