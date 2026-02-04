//! Module Commands
//!
//! Tauri commands for module CRUD operations and related entities.

use mimir_core::dal::campaign as dal;
use mimir_core::dal::catalog::get_monster_by_name;
use mimir_core::models::campaign::{Module, ModuleMonster, ModuleNpc, NewModuleMonster, UpdateModuleMonster};
use mimir_core::models::catalog::Monster;
use mimir_core::services::{
    CreateModuleInput, CreateTokenInput, ModuleService, ModuleType, TokenResponse, TokenService,
    UpdateModuleInput, UpdateTokenInput,
};
use mimir_core::utils::now_rfc3339;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::state::AppState;
use super::{to_api_response, ApiResponse};

/// List all modules for a campaign.
#[tauri::command]
pub fn list_modules(state: State<'_, AppState>, campaign_id: String) -> ApiResponse<Vec<Module>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ModuleService::new(&mut db).list_for_campaign(&campaign_id);
    to_api_response(result)
}

/// Get a module by ID.
#[tauri::command]
pub fn get_module(state: State<'_, AppState>, id: String) -> ApiResponse<Module> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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

/// Create a new module.
#[tauri::command]
pub fn create_module(
    state: State<'_, AppState>,
    request: CreateModuleRequest,
) -> ApiResponse<Module> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let module_type = ModuleType::from(request.module_type.as_deref());

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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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

/// Reorder a module by moving it to a new position (1-indexed).
#[tauri::command]
pub fn reorder_module(
    state: State<'_, AppState>,
    module_id: String,
    new_position: i32,
) -> ApiResponse<Vec<Module>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ModuleService::new(&mut db).reorder(&module_id, new_position);
    to_api_response(result)
}

// =============================================================================
// Module Monster Commands
// =============================================================================

/// Monster with optional catalog data for display.
#[derive(Debug, Serialize)]
pub struct MonsterWithData {
    #[serde(flatten)]
    pub monster: ModuleMonster,
    pub monster_data: Option<serde_json::Value>,
}

/// List all monsters for a module with catalog data.
#[tauri::command]
pub fn list_module_monsters_with_data(
    state: State<'_, AppState>,
    module_id: String,
) -> ApiResponse<Vec<MonsterWithData>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get module monsters
    let monsters = match dal::list_module_monsters(&mut db, &module_id) {
        Ok(m) => m,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // Convert to MonsterWithData with catalog lookup
    let result: Vec<MonsterWithData> = monsters
        .into_iter()
        .map(|m| {
            // Look up the monster in the catalog
            let monster_data = get_monster_by_name(&mut db, &m.monster_name, &m.monster_source)
                .ok()
                .flatten()
                .and_then(|catalog_monster: Monster| catalog_monster.parse_data().ok());

            MonsterWithData {
                monster: m,
                monster_data,
            }
        })
        .collect();

    ApiResponse::ok(result)
}

/// Request for adding a module monster.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddModuleMonsterRequest {
    pub module_id: String,
    pub monster_name: String,
    pub monster_source: String,
    pub quantity: Option<i32>,
    pub display_name: Option<String>,
    pub notes: Option<String>,
}

/// Add a monster to a module (or increment quantity if it already exists).
#[tauri::command]
pub fn add_module_monster(
    state: State<'_, AppState>,
    request: AddModuleMonsterRequest,
) -> ApiResponse<ModuleMonster> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Check if this monster already exists in the module
    let existing = dal::list_module_monsters(&mut db, &request.module_id)
        .ok()
        .and_then(|monsters| {
            monsters.into_iter().find(|m| {
                m.monster_name == request.monster_name && m.monster_source == request.monster_source
            })
        });

    if let Some(existing_monster) = existing {
        // Increment quantity
        let new_qty = existing_monster.quantity + request.quantity.unwrap_or(1);
        let now = now_rfc3339();
        let update = UpdateModuleMonster {
            display_name: None,
            notes: None,
            quantity: Some(new_qty),
            updated_at: Some(&now),
        };
        if let Err(e) = dal::update_module_monster(&mut db, &existing_monster.id, &update) {
            return ApiResponse::err(e.to_string());
        }
        match dal::get_module_monster(&mut db, &existing_monster.id) {
            Ok(m) => ApiResponse::ok(m),
            Err(e) => ApiResponse::err(e.to_string()),
        }
    } else {
        // Create new module monster
        let id = Uuid::new_v4().to_string();
        let display_name_ref = request.display_name.as_deref();
        let notes_ref = request.notes.as_deref();

        let new_monster = NewModuleMonster {
            id: &id,
            module_id: &request.module_id,
            monster_name: &request.monster_name,
            monster_source: &request.monster_source,
            display_name: display_name_ref,
            notes: notes_ref,
            quantity: request.quantity.unwrap_or(1),
        };

        if let Err(e) = dal::insert_module_monster(&mut db, &new_monster) {
            return ApiResponse::err(e.to_string());
        }

        match dal::get_module_monster(&mut db, &id) {
            Ok(m) => ApiResponse::ok(m),
            Err(e) => ApiResponse::err(e.to_string()),
        }
    }
}

/// Request for updating a module monster.
#[derive(Debug, serde::Deserialize)]
pub struct UpdateModuleMonsterRequest {
    pub display_name: Option<String>,
    pub notes: Option<String>,
    pub quantity: Option<i32>,
}

/// Update a module monster.
#[tauri::command]
pub fn update_module_monster(
    state: State<'_, AppState>,
    monster_id: String,
    request: UpdateModuleMonsterRequest,
) -> ApiResponse<ModuleMonster> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let now = now_rfc3339();

    // Convert Option<String> to Option<Option<&str>> for the update struct
    let display_name_ref = request.display_name.as_ref().map(|s| Some(s.as_str()));
    let notes_ref = request.notes.as_ref().map(|s| Some(s.as_str()));

    let update = UpdateModuleMonster {
        display_name: display_name_ref,
        notes: notes_ref,
        quantity: request.quantity,
        updated_at: Some(&now),
    };

    if let Err(e) = dal::update_module_monster(&mut db, &monster_id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_module_monster(&mut db, &monster_id) {
        Ok(monster) => ApiResponse::ok(monster),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Module NPC Commands
// =============================================================================

/// List all NPCs for a module.
#[tauri::command]
pub fn list_module_npcs(state: State<'_, AppState>, module_id: String) -> ApiResponse<Vec<ModuleNpc>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match dal::list_module_npcs(&mut db, &module_id) {
        Ok(npcs) => ApiResponse::ok(npcs),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Token Commands
// =============================================================================

/// List all tokens for a map with resolved names.
#[tauri::command]
pub fn list_tokens(state: State<'_, AppState>, map_id: String) -> ApiResponse<Vec<TokenResponse>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match TokenService::new(&mut db, &state.paths.app_dir).list(&map_id) {
        Ok(tokens) => ApiResponse::ok(tokens),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List token summaries (alias for list_tokens for frontend compatibility).
#[tauri::command]
pub fn list_token_summaries(state: State<'_, AppState>, map_id: String) -> ApiResponse<Vec<TokenResponse>> {
    list_tokens(state, map_id)
}

/// Request for creating a token placement.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenRequest {
    pub map_id: String,
    pub module_monster_id: Option<String>,
    pub module_npc_id: Option<String>,
    pub grid_x: i32,
    pub grid_y: i32,
    pub label: Option<String>,
    pub faction_color: Option<String>,
    pub hidden: bool,
}

/// Create a new token placement.
#[tauri::command]
pub fn create_token(
    state: State<'_, AppState>,
    request: CreateTokenRequest,
) -> ApiResponse<TokenResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let input = CreateTokenInput {
        map_id: request.map_id,
        module_monster_id: request.module_monster_id,
        module_npc_id: request.module_npc_id,
        grid_x: request.grid_x,
        grid_y: request.grid_y,
        label: request.label,
        faction_color: request.faction_color,
        hidden: request.hidden,
    };

    match TokenService::new(&mut db, &state.paths.app_dir).create(input) {
        Ok(token) => ApiResponse::ok(token),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for updating a token placement.
#[derive(Debug, Deserialize)]
pub struct UpdateTokenRequest {
    pub grid_x: Option<i32>,
    pub grid_y: Option<i32>,
    pub label: Option<Option<String>>,
    pub faction_color: Option<Option<String>>,
    pub hidden: Option<bool>,
}

/// Update a token placement.
#[tauri::command]
pub fn update_token(
    state: State<'_, AppState>,
    id: String,
    request: UpdateTokenRequest,
) -> ApiResponse<TokenResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let input = UpdateTokenInput {
        grid_x: request.grid_x,
        grid_y: request.grid_y,
        label: request.label,
        faction_color: request.faction_color,
        hidden: request.hidden,
        ..Default::default()
    };

    match TokenService::new(&mut db, &state.paths.app_dir).update(&id, input) {
        Ok(token) => ApiResponse::ok(token),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Update only the position of a token (optimized for drag operations).
#[tauri::command]
pub fn update_token_position(
    state: State<'_, AppState>,
    id: String,
    grid_x: i32,
    grid_y: i32,
) -> ApiResponse<TokenResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match TokenService::new(&mut db, &state.paths.app_dir).update_position(&id, grid_x, grid_y) {
        Ok(token) => ApiResponse::ok(token),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Update a token's vision settings (D&D 5e vision rules).
#[tauri::command]
pub fn update_token_vision(
    state: State<'_, AppState>,
    id: String,
    vision_bright_ft: Option<i32>,
    vision_dim_ft: Option<i32>,
    vision_dark_ft: i32,
    light_radius_ft: i32,
) -> ApiResponse<TokenResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match TokenService::new(&mut db, &state.paths.app_dir).update_vision(
        &id,
        vision_bright_ft,
        vision_dim_ft,
        vision_dark_ft,
        light_radius_ft,
    ) {
        Ok(token) => ApiResponse::ok(token),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Toggle a token's visibility (hidden from players).
#[tauri::command]
pub fn toggle_token_visibility(
    state: State<'_, AppState>,
    id: String,
) -> ApiResponse<TokenResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match TokenService::new(&mut db, &state.paths.app_dir).toggle_visibility(&id) {
        Ok(token) => ApiResponse::ok(token),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete a token placement.
#[tauri::command]
pub fn delete_token(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match TokenService::new(&mut db, &state.paths.app_dir).delete(&id) {
        Ok(()) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Token Image Commands
// =============================================================================


/// Serve a token's image as a base64 data URL.
///
/// Uses convention-based paths: `bestiary/tokens/{source}/{name}.{ext}`
/// The token_image_path field in the catalog is reserved for custom overrides.
#[tauri::command]
pub fn serve_token_image(
    state: State<'_, AppState>,
    token_id: String,
) -> ApiResponse<Option<String>> {
    tracing::debug!("serve_token_image called for token_id: {}", token_id);

    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get the token placement
    let token = match dal::get_token_placement(&mut db, &token_id) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("Token not found: {} - {}", token_id, e);
            return ApiResponse::err(format!("Token not found: {}", e));
        }
    };

    // Only monster tokens have images from the catalog
    let Some(ref monster_id) = token.module_monster_id else {
        tracing::debug!("Token {} has no module_monster_id, skipping image", token_id);
        return ApiResponse::ok(None);
    };

    // Get the module monster to find the catalog reference
    let module_monster = match dal::get_module_monster_optional(&mut db, monster_id) {
        Ok(Some(m)) => m,
        Ok(None) => {
            tracing::warn!("Module monster not found for id: {}", monster_id);
            return ApiResponse::ok(None);
        }
        Err(e) => return ApiResponse::err(format!("Failed to get module monster: {}", e)),
    };

    tracing::debug!(
        "Looking for token image: monster_name={}, monster_source={}",
        module_monster.monster_name,
        module_monster.monster_source
    );

    // Images are stored in assets/catalog/bestiary/tokens/{source}/{name}.{ext}
    let img_base = state.paths.assets_dir.join("catalog").join("bestiary").join("tokens");
    let source_dir = img_base.join(&module_monster.monster_source);

    tracing::debug!("Token image source dir: {:?}, exists: {}", source_dir, source_dir.exists());

    // Try different extensions in order of preference
    let extensions = ["webp", "png", "jpg", "jpeg"];
    let mut found_path: Option<std::path::PathBuf> = None;
    let mut found_ext: Option<&str> = None;

    for ext in &extensions {
        let path = source_dir.join(format!("{}.{}", &module_monster.monster_name, ext));
        tracing::debug!("Trying path: {:?}, exists: {}", path, path.exists());
        if path.exists() {
            found_path = Some(path);
            found_ext = Some(ext);
            break;
        }
    }

    let (full_path, ext) = match (found_path, found_ext) {
        (Some(p), Some(e)) => {
            tracing::debug!("Found token image at: {:?}", p);
            (p, e)
        }
        _ => {
            tracing::debug!(
                "No token image found for {}/{}",
                module_monster.monster_source,
                module_monster.monster_name
            );
            return ApiResponse::ok(None);
        }
    };

    // Read the image file
    let image_bytes = match std::fs::read(&full_path) {
        Ok(bytes) => {
            tracing::debug!("Read {} bytes from token image", bytes.len());
            bytes
        }
        Err(e) => {
            tracing::warn!("Failed to read token image at {:?}: {}", full_path, e);
            return ApiResponse::ok(None);
        }
    };

    // Determine MIME type from extension
    let mime_type = match ext {
        "webp" => "image/webp",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    };

    // Encode as base64 data URL
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
    let data_url = format!("data:{};base64,{}", mime_type, b64);

    tracing::debug!("Returning token image data URL ({} chars)", data_url.len());
    ApiResponse::ok(Some(data_url))
}
