//! Database-backed item catalog commands.
//!
//! Provides Tauri commands for searching, filtering, and retrieving item data
//! from the 5e catalog database. Items include equipment, magic items, and
//! other gear.

use crate::state::AppState;
use mimir_dm_core::models::catalog::item::{Item, ItemFilters, ItemSummary};
use mimir_dm_core::services::item_service::ItemService;
use tauri::State;
use tracing::debug;

/// Search the item catalog with optional filters.
///
/// Returns a list of item summaries matching the provided criteria.
/// All filter parameters are optional and can be combined.
///
/// # Parameters
/// - `name` - Text to search in item names (case-insensitive)
/// - `item_types` - Filter by item type (e.g., `["Weapon", "Armor", "Wondrous Item"]`)
/// - `rarities` - Filter by rarity (e.g., `["Common", "Uncommon", "Rare"]`)
/// - `sources` - Filter by source books
/// - `min_value` - Minimum item value in gold pieces
/// - `max_value` - Maximum item value in gold pieces
///
/// # Returns
/// List of `ItemSummary` objects containing basic item information.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn search_items(
    name: Option<String>,
    item_types: Option<Vec<String>>,
    rarities: Option<Vec<String>>,
    sources: Option<Vec<String>>,
    min_value: Option<f64>,
    max_value: Option<f64>,
    state: State<'_, AppState>,
) -> Result<Vec<ItemSummary>, String> {
    debug!(
        "Searching items with name: {:?}, item_types: {:?}, rarities: {:?}, sources: {:?}",
        name, item_types, rarities, sources
    );

    let filters = ItemFilters {
        name,
        item_types,
        rarities,
        sources,
        min_value,
        max_value,
    };

    state.with_connection("item search", |conn| {
        ItemService::new(conn).search_items(filters)
    })
}

/// Get complete item details by database ID.
///
/// Retrieves the full item record including properties, description, and value.
///
/// # Parameters
/// - `item_id` - Database ID of the item
///
/// # Returns
/// The complete `Item` object if found, or `None` if no match exists.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_item(item_id: i32, state: State<'_, AppState>) -> Result<Option<Item>, String> {
    debug!("Getting item details for ID: {}", item_id);

    state.with_connection("get item", |conn| {
        ItemService::new(conn).get_item_by_id(item_id)
    })
}

/// Get complete item details by name and source.
///
/// Retrieves the full item record including properties, description, and value.
///
/// # Parameters
/// - `item_name` - Exact item name (case-sensitive)
/// - `item_source` - Source book abbreviation (e.g., "PHB", "DMG")
///
/// # Returns
/// The complete `Item` object if found, or `None` if no match exists.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_item_details(
    item_name: String,
    item_source: String,
    state: State<'_, AppState>,
) -> Result<Option<Item>, String> {
    debug!(
        "Getting item details for name: {}, source: {}",
        item_name, item_source
    );

    state.with_connection("get item details", |conn| {
        ItemService::new(conn).get_item_by_name_and_source(&item_name, &item_source)
    })
}

/// Get all unique item types in the catalog.
///
/// Returns item type categories present in the item catalog.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of type names (e.g., `["Weapon", "Armor", "Potion", "Wondrous Item"]`).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_item_types(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting all item types");

    state.with_connection("get item types", |conn| {
        ItemService::new(conn).get_item_types()
    })
}

/// Get all unique item rarities in the catalog.
///
/// Returns rarity values present in the item catalog.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of rarity names (e.g., `["Common", "Uncommon", "Rare", "Very Rare", "Legendary"]`).
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_item_rarities(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting all item rarities");

    state.with_connection("get item rarities", |conn| {
        ItemService::new(conn).get_item_rarities()
    })
}

/// Get all unique source books containing items.
///
/// Returns source book abbreviations that contain at least one item.
/// Used to populate filter dropdowns in the UI.
///
/// # Returns
/// List of source abbreviations.
///
/// # Errors
/// Returns an error string if the database connection or query fails.
#[tauri::command]
pub async fn get_item_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    debug!("Getting all item sources");

    state.with_connection("get item sources", |conn| {
        ItemService::new(conn).get_item_sources()
    })
}
