//! Database-backed table catalog commands.
//!
//! Provides Tauri commands for searching and retrieving random table data
//! from the 5e catalog database. Used for random generation and DM tools.

use crate::state::AppState;
use mimir_dm_core::models::catalog::table::{Table, TableFilters, TableSummary};
use mimir_dm_core::services::TableService;
use tauri::State;
use tracing::{debug, info};

/// Search the table catalog with optional filters.
///
/// Returns a list of table summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `query` - Text to search in table names (case-insensitive)
/// - `categories` - Filter by table category (e.g., "Treasure", "Encounters")
/// - `sources` - Filter by source books
///
/// # Returns
/// List of `TableSummary` objects containing basic table information.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_tables(
    query: Option<String>,
    categories: Option<Vec<String>>,
    sources: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<Vec<TableSummary>, String> {
    debug!(
        "Database table search - query: {:?}, categories: {:?}, sources: {:?}",
        query, categories, sources
    );

    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut service = TableService::new(&mut conn);

    let filters = TableFilters {
        name: query,
        categories,
        sources,
    };

    let tables = service
        .search_tables(filters)
        .map_err(|e| format!("Failed to search tables: {}", e))?;

    info!("Found {} tables in database search", tables.len());
    Ok(tables)
}

/// Get table by database ID.
///
/// Retrieves a table record by its internal database identifier.
///
/// # Parameters
/// - `id` - Database ID of the table
///
/// # Returns
/// The complete `Table` object if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_table(id: i32, state: State<'_, AppState>) -> Result<Option<Table>, String> {
    debug!("Getting table by ID: {}", id);

    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut service = TableService::new(&mut conn);

    service
        .get_table_by_id(id)
        .map_err(|e| format!("Failed to get table: {}", e))
}

/// Get complete table details by name and source.
///
/// Retrieves the full table record including rows and dice expressions.
///
/// # Parameters
/// - `name` - Exact table name (case-sensitive)
/// - `source` - Source book abbreviation (e.g., "DMG", "XGE")
///
/// # Returns
/// The complete `Table` object if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_table_details(
    name: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<Option<Table>, String> {
    debug!("Getting table details: {} from {}", name, source);

    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut service = TableService::new(&mut conn);

    service
        .get_table_by_name_and_source(&name, &source)
        .map_err(|e| format!("Failed to get table details: {}", e))
}

/// Get all unique table categories in the catalog.
///
/// Returns category names for populating filter dropdowns.
/// Examples include Treasure, Encounters, Wild Magic, etc.
///
/// # Returns
/// List of category names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_table_categories(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting table categories from database");

    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut service = TableService::new(&mut conn);

    let categories = service
        .get_table_categories()
        .map_err(|e| format!("Failed to get table categories: {}", e))?;

    info!("Found {} table categories in database", categories.len());
    Ok(categories)
}

/// Get all unique source books containing tables.
///
/// Returns source book abbreviations for populating filter dropdowns.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_table_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting table sources from database");

    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut service = TableService::new(&mut conn);

    let sources = service
        .get_table_sources()
        .map_err(|e| format!("Failed to get table sources: {}", e))?;

    info!("Found {} table sources in database", sources.len());
    Ok(sources)
}
