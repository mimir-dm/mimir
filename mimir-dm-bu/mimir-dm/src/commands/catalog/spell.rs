//! Database-backed spell catalog commands.
//!
//! Provides Tauri commands for searching, filtering, and retrieving spell data
//! from the 5e catalog database. Used by the frontend for spell browsing and
//! character spell selection.

use crate::state::AppState;
use mimir_dm_core::models::catalog::{Spell, SpellFilters, SpellSummary};
use mimir_dm_core::services::SpellService;
use tauri::State;
use tracing::debug;

/// Search the spell catalog with optional filters.
///
/// Returns a paginated list of spell summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `query` - Text to search in spell names (case-insensitive)
/// - `sources` - Filter by source books (e.g., `["PHB", "XGE"]`)
/// - `levels` - Filter by spell levels (0 for cantrips, 1-9 for leveled spells)
/// - `schools` - Filter by magic schools (e.g., `["Evocation", "Abjuration"]`)
/// - `tags` - Filter by spell tags (e.g., `["ritual", "concentration"]`)
/// - `limit` - Maximum number of results to return
/// - `offset` - Number of results to skip (for pagination)
///
/// # Returns
/// List of `SpellSummary` objects containing basic spell information.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn search_spells(
    query: Option<String>,
    sources: Option<Vec<String>>,
    levels: Option<Vec<i32>>,
    schools: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    classes: Option<Vec<String>>,
    limit: Option<i32>,
    offset: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<SpellSummary>, String> {
    debug!(
        "Database spell search - query: {:?}, sources: {:?}, levels: {:?}",
        query, sources, levels
    );

    let filters = SpellFilters {
        query,
        levels: levels.unwrap_or_default(),
        schools: schools.unwrap_or_default(),
        sources: sources.unwrap_or_default(),
        tags: tags.unwrap_or_default(),
        classes: classes.unwrap_or_default(),
        limit,
        offset,
    };

    state.with_connection("spell search", |conn| {
        SpellService::search_spells(conn, filters)
    })
}

/// Get complete spell details by name and source.
///
/// Retrieves the full spell record including description, components,
/// casting time, range, duration, and all other spell properties.
///
/// # Parameters
/// - `name` - Exact spell name (case-sensitive)
/// - `source` - Source book abbreviation (e.g., "PHB", "XGE")
///
/// # Returns
/// The complete `Spell` object if found, or `None` if no matching spell exists.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_spell_details(
    name: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<Option<Spell>, String> {
    debug!(
        "Getting spell details from database: {} from {}",
        name, source
    );

    state.with_connection("get spell details", |conn| {
        SpellService::get_spell_details(conn, &name, &source)
    })
}

/// Get all unique spell source books in the catalog.
///
/// Returns a list of source book abbreviations that contain at least one spell.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of source abbreviations (e.g., `["PHB", "XGE", "TCE"]`).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_spell_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting spell sources from database");

    state.with_connection("get spell sources", |conn| {
        SpellService::get_spell_sources(conn)
    })
}

/// Get all unique spell schools in the catalog.
///
/// Returns a list of magic schools present in the spell catalog.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of school names (e.g., `["Abjuration", "Conjuration", "Evocation"]`).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_spell_schools(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting spell schools from database");

    state.with_connection("get spell schools", |conn| {
        SpellService::get_spell_schools(conn)
    })
}

/// Get spell count statistics grouped by source book.
///
/// Returns the number of spells from each source book in the catalog.
/// Useful for displaying catalog coverage and statistics in the UI.
///
/// # Returns
/// List of tuples containing (source_abbreviation, spell_count).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_spell_statistics(
    state: State<'_, AppState>,
) -> Result<Vec<(String, i64)>, String> {
    debug!("Getting spell statistics from database");

    state.with_connection("get spell statistics", |conn| {
        SpellService::get_spell_count_by_source(conn)
    })
}

/// Get total number of spells in the catalog.
///
/// Returns the total count of all spells across all source books.
///
/// # Returns
/// Total spell count as a 64-bit integer.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_spell_count(state: State<'_, AppState>) -> Result<i64, String> {
    debug!("Getting total spell count from database");

    state.with_connection("get spell count", |conn| {
        SpellService::get_total_spell_count(conn)
    })
}
