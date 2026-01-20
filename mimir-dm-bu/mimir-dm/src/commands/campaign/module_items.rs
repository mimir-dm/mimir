//! Module item management commands.
//!
//! Provides Tauri commands for managing item/treasure associations within campaign modules.
//! Items are catalog references looked up by name and source.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use mimir_dm_core::{
    models::campaign::module_items::{LocationGroup, ModuleItem, ModuleItemWithData},
    services::ModuleItemService,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddItemRequest {
    pub module_id: i32,
    pub name: String,
    pub source: String,
    pub quantity: i32,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateItemRequest {
    pub quantity: Option<i32>,
    pub location: Option<Option<String>>,
    pub notes: Option<Option<String>>,
}

/// Add an item to a module.
///
/// Associates an item from the catalog with a module, optionally tagged to a location.
/// If the same item already exists at the same location, quantities are combined.
#[tauri::command]
pub async fn add_module_item(
    request: AddItemRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<ModuleItem>, ApiError> {
    info!(
        "Adding item {} ({}) to module {} with quantity {}",
        request.name, request.source, request.module_id, request.quantity
    );

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleItemService::new(&mut conn);

    match service.add_item(
        request.module_id,
        request.name,
        request.source,
        request.quantity,
        request.location,
        request.notes,
    ) {
        Ok(item) => {
            info!("Item added successfully with ID: {}", item.id);
            Ok(ApiResponse::success(item))
        }
        Err(e) => {
            error!("Failed to add item: {}", e);
            Ok(ApiResponse::error(format!("Failed to add item: {}", e)))
        }
    }
}

/// Remove an item from a module.
#[tauri::command]
pub async fn remove_module_item(
    item_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
    info!("Removing module item with ID: {}", item_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleItemService::new(&mut conn);

    match service.remove_item(item_id) {
        Ok(()) => {
            info!("Item removed successfully");
            Ok(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to remove item: {}", e);
            Ok(ApiResponse::error(format!("Failed to remove item: {}", e)))
        }
    }
}

/// Update a module item entry.
#[tauri::command]
pub async fn update_module_item(
    item_id: i32,
    request: UpdateItemRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<ModuleItem>, ApiError> {
    info!("Updating module item with ID: {}", item_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleItemService::new(&mut conn);

    match service.update_item(item_id, request.quantity, request.location, request.notes) {
        Ok(item) => {
            info!("Item updated successfully");
            Ok(ApiResponse::success(item))
        }
        Err(e) => {
            error!("Failed to update item: {}", e);
            Ok(ApiResponse::error(format!("Failed to update item: {}", e)))
        }
    }
}

/// List all items for a module.
#[tauri::command]
pub async fn list_module_items(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<ModuleItem>>, ApiError> {
    info!("Listing items for module: {}", module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleItemService::new(&mut conn);

    match service.get_items_for_module(module_id) {
        Ok(items) => {
            info!("Found {} items", items.len());
            Ok(ApiResponse::success(items))
        }
        Err(e) => {
            error!("Failed to list items: {}", e);
            Ok(ApiResponse::error(format!("Failed to list items: {}", e)))
        }
    }
}

/// List items for a module with full catalog data.
#[tauri::command]
pub async fn list_module_items_with_data(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<ModuleItemWithData>>, ApiError> {
    info!("Listing items with data for module: {}", module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleItemService::new(&mut conn);

    match service.get_items_with_data(module_id) {
        Ok(items) => {
            info!("Found {} items with data", items.len());
            Ok(ApiResponse::success(items))
        }
        Err(e) => {
            error!("Failed to list items with data: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list items with data: {}",
                e
            )))
        }
    }
}

/// List items grouped by location.
#[tauri::command]
pub async fn list_module_items_by_location(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<LocationGroup>>, ApiError> {
    info!(
        "Listing items grouped by location for module: {}",
        module_id
    );

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleItemService::new(&mut conn);

    match service.get_items_grouped_by_location(module_id) {
        Ok(groups) => {
            info!("Found {} location groups", groups.len());
            Ok(ApiResponse::success(groups))
        }
        Err(e) => {
            error!("Failed to list items by location: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list items by location: {}",
                e
            )))
        }
    }
}

/// Get item locations for a module.
#[tauri::command]
pub async fn get_module_item_locations(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Option<String>>>, ApiError> {
    info!("Getting item locations for module: {}", module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleItemService::new(&mut conn);

    match service.get_locations(module_id) {
        Ok(locations) => {
            info!("Found {} distinct locations", locations.len());
            Ok(ApiResponse::success(locations))
        }
        Err(e) => {
            error!("Failed to get item locations: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to get item locations: {}",
                e
            )))
        }
    }
}

/// Clear all items from a module.
#[tauri::command]
pub async fn clear_module_items(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<usize>, ApiError> {
    info!("Clearing all items from module: {}", module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleItemService::new(&mut conn);

    match service.clear_module_items(module_id) {
        Ok(count) => {
            info!("Cleared {} items", count);
            Ok(ApiResponse::success(count))
        }
        Err(e) => {
            error!("Failed to clear items: {}", e);
            Ok(ApiResponse::error(format!("Failed to clear items: {}", e)))
        }
    }
}
