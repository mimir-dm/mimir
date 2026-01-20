//! Database-backed vehicle catalog commands.
//!
//! Provides Tauri commands for searching and retrieving vehicle data
//! from the 5e catalog database. Includes ships, war machines, and other conveyances.

use crate::state::AppState;
use mimir_dm_core::models::catalog::vehicle::{Vehicle, VehicleFilters, VehicleSummary};
use mimir_dm_core::services::VehicleService;
use tauri::State;
use tracing::{debug, error};

/// Search the vehicle catalog with optional filters.
///
/// Returns a list of vehicle summaries matching the provided criteria.
/// All filter parameters within the `VehicleFilters` struct are optional.
///
/// # Parameters
/// - `filters` - Filter criteria including type, size, terrain, and text search
///
/// # Returns
/// List of `VehicleSummary` objects containing basic vehicle information.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_vehicles(
    filters: VehicleFilters,
    state: State<'_, AppState>,
) -> Result<Vec<VehicleSummary>, String> {
    debug!("Searching vehicles with filters: {:?}", filters);

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = VehicleService::new(&mut conn);
    service
        .search_vehicles(filters)
        .map_err(|e| format!("Database query failed: {}", e))
}

/// Get complete vehicle details by name and source.
///
/// Retrieves the full vehicle record including stats, crew requirements, and actions.
///
/// # Parameters
/// - `vehicle_name` - Exact vehicle name (case-sensitive)
/// - `vehicle_source` - Source book abbreviation (e.g., "GoS", "DMG")
///
/// # Returns
/// The complete `Vehicle` object if found, or `None` if no match.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_vehicle_details(
    vehicle_name: String,
    vehicle_source: String,
    state: State<'_, AppState>,
) -> Result<Option<Vehicle>, String> {
    debug!(
        "Getting vehicle details for name: {}, source: {}",
        vehicle_name, vehicle_source
    );

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = VehicleService::new(&mut conn);
    service
        .get_vehicle_by_name_and_source(&vehicle_name, &vehicle_source)
        .map_err(|e| format!("Database query failed: {}", e))
}

/// Get all unique vehicle types in the catalog.
///
/// Returns vehicle type categories for populating filter dropdowns.
/// Examples include Ship, Infernal War Machine, Spelljammer, etc.
///
/// # Returns
/// List of vehicle type names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_vehicle_types(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting all vehicle types");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = VehicleService::new(&mut conn);
    service
        .get_all_vehicle_types()
        .map_err(|e| format!("Database query failed: {}", e))
}

/// Get all unique vehicle sizes in the catalog.
///
/// Returns size categories for populating filter dropdowns.
/// Uses standard D&D sizes (Large, Huge, Gargantuan, etc.).
///
/// # Returns
/// List of size names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_vehicle_sizes(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting all vehicle sizes");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = VehicleService::new(&mut conn);
    service
        .get_all_sizes()
        .map_err(|e| format!("Database query failed: {}", e))
}

/// Get all unique vehicle terrains in the catalog.
///
/// Returns terrain types for populating filter dropdowns.
/// Examples include Water, Land, Air, Space, etc.
///
/// # Returns
/// List of terrain names.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_vehicle_terrains(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting all vehicle terrains");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = VehicleService::new(&mut conn);
    service
        .get_all_terrains()
        .map_err(|e| format!("Database query failed: {}", e))
}

/// Get vehicle count statistics grouped by source book.
///
/// Returns a breakdown of how many vehicles are in each source book.
/// Used for displaying catalog statistics in the UI.
///
/// # Returns
/// List of tuples containing (source abbreviation, count).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_vehicle_statistics(
    state: State<'_, AppState>,
) -> Result<Vec<(String, i64)>, String> {
    debug!("Getting vehicle statistics");

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut service = VehicleService::new(&mut conn);
    service
        .get_vehicle_count_by_source()
        .map_err(|e| format!("Database query failed: {}", e))
}
