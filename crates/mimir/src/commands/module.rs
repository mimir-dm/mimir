//! Module Commands
//!
//! Tauri commands for module CRUD operations.

use mimir_core::models::campaign::Module;
use mimir_core::services::{CreateModuleInput, ModuleService, ModuleType, UpdateModuleInput};
use tauri::State;

use crate::state::AppState;
use super::{to_api_response, ApiResponse};

/// List all modules for a campaign.
#[tauri::command]
pub fn list_modules(state: State<'_, AppState>, campaign_id: String) -> ApiResponse<Vec<Module>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ModuleService::new(&mut db).list_for_campaign(&campaign_id);
    to_api_response(result)
}

/// Get a module by ID.
#[tauri::command]
pub fn get_module(state: State<'_, AppState>, id: String) -> ApiResponse<Module> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ModuleService::new(&mut db).get(&id);
    match result {
        Ok(Some(module)) => ApiResponse::ok(module),
        Ok(None) => ApiResponse::err(format!("Module not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for creating a new module.
#[derive(Debug, serde::Deserialize)]
pub struct CreateModuleRequest {
    pub campaign_id: String,
    pub name: String,
    pub description: Option<String>,
    pub module_type: Option<String>,
}

/// Parse module type from string.
fn parse_module_type(s: Option<&str>) -> ModuleType {
    match s {
        Some("mystery") => ModuleType::Mystery,
        Some("dungeon") => ModuleType::Dungeon,
        Some("heist") => ModuleType::Heist,
        Some("horror") => ModuleType::Horror,
        Some("political") => ModuleType::Political,
        _ => ModuleType::General,
    }
}

/// Create a new module.
#[tauri::command]
pub fn create_module(
    state: State<'_, AppState>,
    request: CreateModuleRequest,
) -> ApiResponse<Module> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let module_type = parse_module_type(request.module_type.as_deref());

    let mut input = CreateModuleInput::new(&request.campaign_id, &request.name)
        .with_type(module_type);

    if let Some(desc) = request.description {
        input = input.with_description(desc);
    }

    let result = ModuleService::new(&mut db).create(input);
    to_api_response(result)
}

/// Request for updating a module.
#[derive(Debug, serde::Deserialize)]
pub struct UpdateModuleRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

/// Update a module.
#[tauri::command]
pub fn update_module(
    state: State<'_, AppState>,
    id: String,
    request: UpdateModuleRequest,
) -> ApiResponse<Module> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let input = UpdateModuleInput {
        name: request.name,
        description: request.description,
    };

    let result = ModuleService::new(&mut db).update(&id, input);
    to_api_response(result)
}

/// Delete a module permanently.
#[tauri::command]
pub fn delete_module(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ModuleService::new(&mut db).delete(&id);
    to_api_response(result)
}

/// Get a module by campaign ID and module number.
#[tauri::command]
pub fn get_module_by_number(
    state: State<'_, AppState>,
    campaign_id: String,
    module_number: i32,
) -> ApiResponse<Module> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ModuleService::new(&mut db).get_by_number(&campaign_id, module_number);
    match result {
        Ok(Some(module)) => ApiResponse::ok(module),
        Ok(None) => ApiResponse::err(format!(
            "Module #{} not found in campaign {}",
            module_number, campaign_id
        )),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}
