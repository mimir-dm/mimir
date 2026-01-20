//! Database-backed class catalog commands.
//!
//! Provides Tauri commands for searching and retrieving character class data
//! from the 5e catalog database. Used for character creation and class feature lookup.

use crate::state::AppState;
use mimir_dm_core::models::catalog::class::{Class, ClassFilters, ClassSummary, Subclass};
use mimir_dm_core::services::ClassService;
use tauri::State;

/// Search the class catalog with optional filters.
///
/// Returns a list of class summaries matching the provided criteria.
/// All filter parameters within the `ClassFilters` struct are optional.
///
/// # Parameters
/// - `filters` - Filter criteria including source, primary ability, and text search
///
/// # Returns
/// List of `ClassSummary` objects containing basic class information.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_classes(
    state: State<'_, AppState>,
    filters: ClassFilters,
) -> Result<Vec<ClassSummary>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut class_service = ClassService::new(&mut conn);
    class_service
        .search_classes(filters)
        .map_err(|e| e.to_string())
}

/// Get complete class details by name and source.
///
/// Retrieves the full class record including hit dice, proficiencies, and features.
///
/// # Parameters
/// - `className` - Exact class name (case-sensitive, e.g., "Fighter", "Wizard")
/// - `classSource` - Source book abbreviation (e.g., "PHB", "XGE")
///
/// # Returns
/// The complete `Class` object if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_class_details(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] className: String,
    #[allow(non_snake_case)] classSource: String,
) -> Result<Option<Class>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut class_service = ClassService::new(&mut conn);
    class_service
        .get_class_by_name_and_source(&className, &classSource)
        .map_err(|e| e.to_string())
}

/// Get complete subclass details by name, parent class, and source.
///
/// Retrieves the full subclass record including features and requirements.
///
/// # Parameters
/// - `subclassName` - Exact subclass name (e.g., "Champion", "Evocation")
/// - `className` - Parent class name (e.g., "Fighter", "Wizard")
/// - `classSource` - Source book abbreviation for the parent class
///
/// # Returns
/// The complete `Subclass` object if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_subclass_details(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] subclassName: String,
    #[allow(non_snake_case)] className: String,
    #[allow(non_snake_case)] classSource: String,
) -> Result<Option<Subclass>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut class_service = ClassService::new(&mut conn);
    class_service
        .get_subclass_by_name(&subclassName, &className, &classSource)
        .map_err(|e| e.to_string())
}

/// Get all subclasses for a specific class.
///
/// Retrieves all subclass options available for a given base class.
///
/// # Parameters
/// - `className` - Parent class name (e.g., "Fighter", "Wizard")
/// - `classSource` - Source book abbreviation for the parent class
///
/// # Returns
/// List of `Subclass` objects for the specified class.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_class_subclasses(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] className: String,
    #[allow(non_snake_case)] classSource: String,
) -> Result<Vec<Subclass>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut class_service = ClassService::new(&mut conn);
    class_service
        .get_subclasses_for_class(&className, &classSource)
        .map_err(|e| e.to_string())
}

/// Get all unique source books containing classes.
///
/// Returns source book abbreviations for populating filter dropdowns.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_class_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut class_service = ClassService::new(&mut conn);
    class_service.get_class_sources().map_err(|e| e.to_string())
}

/// Get all unique primary abilities used by classes.
///
/// Returns ability score names for populating filter dropdowns.
/// Examples include Strength, Dexterity, Intelligence, etc.
///
/// # Returns
/// List of ability names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_class_primary_abilities(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut class_service = ClassService::new(&mut conn);
    class_service
        .get_primary_abilities()
        .map_err(|e| e.to_string())
}

/// Get class count statistics grouped by source book.
///
/// Returns a breakdown of how many classes are in each source book.
/// Used for displaying catalog statistics in the UI.
///
/// # Returns
/// List of tuples containing (source abbreviation, count).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_class_statistics(
    state: State<'_, AppState>,
) -> Result<Vec<(String, i64)>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut class_service = ClassService::new(&mut conn);
    class_service
        .get_class_count_by_source()
        .map_err(|e| e.to_string())
}
