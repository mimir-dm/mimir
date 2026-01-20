//! Database-backed variant rule catalog commands.
//!
//! Provides Tauri commands for searching and retrieving variant and optional rules
//! from the 5e catalog database. Includes house rules, optional mechanics, etc.

use crate::state::AppState;
use mimir_dm_core::models::catalog::variant_rule::{VariantRule, VariantRuleFilters};
use mimir_dm_core::services::VariantRuleService;
use tauri::State;

/// Search the variant rule catalog with optional filters.
///
/// Returns a list of variant rules matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `query` - Text to search in rule names (case-insensitive)
/// - `rule_types` - Filter by rule type
/// - `sources` - Filter by source books
///
/// # Returns
/// List of variant rules as JSON values.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_variant_rules(
    query: Option<String>,
    rule_types: Option<Vec<String>>,
    sources: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut service = VariantRuleService::new(&mut conn);

    let filters = VariantRuleFilters {
        name: query,
        rule_types,
        sources,
    };

    let variant_rules = service
        .search_variant_rules(filters)
        .map_err(|e| format!("Failed to search variant rules: {}", e))?;

    // Convert to JSON values to match frontend expectations
    let json_results: Vec<serde_json::Value> = variant_rules
        .into_iter()
        .map(|rule| serde_json::to_value(rule).unwrap_or_default())
        .collect();

    Ok(json_results)
}

/// Get variant rule by database ID.
///
/// Retrieves a rule record by its internal database identifier.
///
/// # Parameters
/// - `id` - Database ID of the variant rule
///
/// # Returns
/// The complete `VariantRule` object if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_variant_rule(
    id: i32,
    state: State<'_, AppState>,
) -> Result<Option<VariantRule>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut service = VariantRuleService::new(&mut conn);

    service
        .get_variant_rule_by_id(id)
        .map_err(|e| format!("Failed to get variant rule: {}", e))
}

/// Get complete variant rule details by name and source.
///
/// Retrieves the full rule record including description and mechanics.
///
/// # Parameters
/// - `name` - Exact rule name (case-sensitive)
/// - `source` - Source book abbreviation (e.g., "DMG", "PHB")
///
/// # Returns
/// The complete `VariantRule` object if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_variant_rule_details(
    name: String,
    source: String,
    state: State<'_, AppState>,
) -> Result<Option<VariantRule>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut service = VariantRuleService::new(&mut conn);

    service
        .get_variant_rule_by_name_and_source(&name, &source)
        .map_err(|e| format!("Failed to get variant rule details: {}", e))
}

/// Get all unique rule types in the variant rule catalog.
///
/// Returns rule type categories for populating filter dropdowns.
///
/// # Returns
/// List of rule type names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_variant_rule_types(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut service = VariantRuleService::new(&mut conn);

    service
        .get_variant_rule_types()
        .map_err(|e| format!("Failed to get variant rule types: {}", e))
}

/// Get all unique source books containing variant rules.
///
/// Returns source book abbreviations for populating filter dropdowns.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_variant_rule_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let mut service = VariantRuleService::new(&mut conn);

    service
        .get_variant_rule_sources()
        .map_err(|e| format!("Failed to get variant rule sources: {}", e))
}
