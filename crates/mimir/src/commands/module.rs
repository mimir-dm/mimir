//! Module Commands
//!
//! Tauri commands for module CRUD operations and related entities.

use mimir_core::dal::campaign as dal;
use mimir_core::dal::get_monster_by_name;
use mimir_core::models::campaign::{
    Module, ModuleMonster, ModuleNpc, NewModuleMonster, NewTokenPlacement, TokenPlacement,
    UpdateModuleMonster, UpdateTokenPlacement,
};
use mimir_core::models::catalog::Monster;
use mimir_core::services::{CreateModuleInput, ModuleService, ModuleType, UpdateModuleInput};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
        let now = chrono::Utc::now().to_rfc3339();
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();

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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match dal::list_module_npcs(&mut db, &module_id) {
        Ok(npcs) => ApiResponse::ok(npcs),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Token Commands
// =============================================================================

use mimir_core::services::MapService;

/// Token response matching the frontend Token interface.
/// Transforms backend TokenPlacement to frontend-expected format.
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub id: String,
    pub map_id: String,
    pub name: String,
    pub token_type: String,
    pub size: String,
    pub x: f64,
    pub y: f64,
    pub visible_to_players: bool,
    pub color: Option<String>,
    pub image_path: Option<String>,
    pub monster_id: Option<String>,
    pub character_id: Option<String>,
    pub notes: Option<String>,
    pub vision_type: String,
    pub vision_range_ft: Option<i32>,
    // New vision fields (D&D 5e rules)
    pub vision_bright_ft: Option<i32>,
    pub vision_dim_ft: Option<i32>,
    pub vision_dark_ft: i32,
    pub light_radius_ft: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Get the grid size (pixels per grid) from a map's UVTT file.
fn get_map_grid_size(db: &mut diesel::SqliteConnection, app_dir: &std::path::Path, map_id: &str) -> i32 {
    let mut service = MapService::new(db, app_dir);

    if let Ok(Some(map)) = service.get(map_id) {
        if let Ok(uvtt_bytes) = service.read_uvtt_file(&map) {
            if let Ok(uvtt_json) = serde_json::from_slice::<serde_json::Value>(&uvtt_bytes) {
                return uvtt_json
                    .get("resolution")
                    .and_then(|r| r.get("pixels_per_grid"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(70) as i32;
            }
        }
    }
    70 // Default grid size
}

/// Transform a TokenPlacement into a TokenResponse.
fn transform_token(
    placement: TokenPlacement,
    token_type: String,
    name: Option<String>,
    size: String,
    grid_size_px: i32,
) -> TokenResponse {
    // Convert grid coordinates to pixel coordinates (center of grid cell)
    let x = (placement.grid_x as f64 + 0.5) * grid_size_px as f64;
    let y = (placement.grid_y as f64 + 0.5) * grid_size_px as f64;

    TokenResponse {
        id: placement.id,
        map_id: placement.map_id,
        name: placement.label.or(name).unwrap_or_else(|| "Unknown".to_string()),
        token_type,
        size,
        x,
        y,
        visible_to_players: placement.hidden == 0,
        color: placement.faction_color,
        image_path: None, // Could be populated from monster data
        monster_id: placement.module_monster_id,
        character_id: placement.module_npc_id,
        notes: None,
        vision_type: "normal".to_string(),
        vision_range_ft: None,
        // New vision fields
        vision_bright_ft: placement.vision_bright_ft,
        vision_dim_ft: placement.vision_dim_ft,
        vision_dark_ft: placement.vision_dark_ft,
        light_radius_ft: placement.light_radius_ft,
        created_at: placement.created_at.clone(),
        updated_at: placement.created_at, // Use created_at as updated_at since we don't track it
    }
}

/// List all tokens for a map with resolved names.
#[tauri::command]
pub fn list_tokens(state: State<'_, AppState>, map_id: String) -> ApiResponse<Vec<TokenResponse>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Get the map's grid size from UVTT
    let grid_size_px = get_map_grid_size(&mut db, &state.paths.app_dir, &map_id);

    // Get raw token placements
    let placements = match dal::list_token_placements(&mut db, &map_id) {
        Ok(p) => p,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // Resolve names and transform each token
    let mut tokens: Vec<TokenResponse> = Vec::new();
    for placement in placements {
        let (token_type, name, size) = resolve_token_names(&mut db, &placement);
        tokens.push(transform_token(placement, token_type, name, size, grid_size_px));
    }

    ApiResponse::ok(tokens)
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
    // Validate: can't have both monster_id and npc_id
    if request.module_monster_id.is_some() && request.module_npc_id.is_some() {
        return ApiResponse::err("Only one of module_monster_id or module_npc_id can be provided".to_string());
    }
    // If neither monster nor npc is provided, a label is required (for PC tokens)
    if request.module_monster_id.is_none() && request.module_npc_id.is_none() && request.label.is_none() {
        return ApiResponse::err("A label is required for PC tokens (when no monster_id or npc_id is provided)".to_string());
    }

    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Get the map's grid size from UVTT
    let grid_size_px = get_map_grid_size(&mut db, &state.paths.app_dir, &request.map_id);

    let id = Uuid::new_v4().to_string();

    // Build the placement
    let label_ref = request.label.as_deref();
    let color_ref = request.faction_color.as_deref();
    let monster_ref = request.module_monster_id.as_deref();
    let npc_ref = request.module_npc_id.as_deref();

    let placement = NewTokenPlacement {
        id: &id,
        map_id: &request.map_id,
        module_monster_id: monster_ref,
        module_npc_id: npc_ref,
        grid_x: request.grid_x,
        grid_y: request.grid_y,
        label: label_ref,
        faction_color: color_ref,
        hidden: if request.hidden { 1 } else { 0 },
        vision_bright_ft: None,    // Default: unlimited in bright light
        vision_dim_ft: None,       // Default: unlimited in dim light
        vision_dark_ft: 0,         // Default: blind in darkness
        light_radius_ft: 0,        // Default: no light source
    };

    if let Err(e) = dal::insert_token_placement(&mut db, &placement) {
        return ApiResponse::err(e.to_string());
    }

    // Fetch and return with resolved names
    let token = match dal::get_token_placement(&mut db, &id) {
        Ok(p) => p,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let (token_type, name, size) = resolve_token_names(&mut db, &token);
    ApiResponse::ok(transform_token(token, token_type, name, size, grid_size_px))
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Build update struct - handle Option<Option<String>> properly
    // Outer None = don't update, Some(None) = set to NULL, Some(Some(v)) = set to v
    let label: Option<Option<&str>> = match &request.label {
        Some(inner) => Some(inner.as_deref()),
        None => None,
    };
    let faction_color: Option<Option<&str>> = match &request.faction_color {
        Some(inner) => Some(inner.as_deref()),
        None => None,
    };

    let update = UpdateTokenPlacement {
        grid_x: request.grid_x,
        grid_y: request.grid_y,
        label,
        faction_color,
        hidden: request.hidden.map(|h| if h { 1 } else { 0 }),
        vision_bright_ft: None,  // Not updated by this command
        vision_dim_ft: None,     // Not updated by this command
        vision_dark_ft: None,    // Not updated by this command
        light_radius_ft: None,   // Not updated by this command
    };

    if let Err(e) = dal::update_token_placement(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    // Fetch and return with resolved names
    let token = match dal::get_token_placement(&mut db, &id) {
        Ok(p) => p,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let grid_size_px = get_map_grid_size(&mut db, &state.paths.app_dir, &token.map_id);
    let (token_type, name, size) = resolve_token_names(&mut db, &token);
    ApiResponse::ok(transform_token(token, token_type, name, size, grid_size_px))
}

/// Update only the position of a token (optimized for drag operations).
#[tauri::command]
pub fn update_token_position(
    state: State<'_, AppState>,
    id: String,
    grid_x: i32,
    grid_y: i32,
) -> ApiResponse<TokenResponse> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let update = UpdateTokenPlacement::set_position(grid_x, grid_y);

    if let Err(e) = dal::update_token_placement(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    // Fetch and return with resolved names
    let token = match dal::get_token_placement(&mut db, &id) {
        Ok(p) => p,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let grid_size_px = get_map_grid_size(&mut db, &state.paths.app_dir, &token.map_id);
    let (token_type, name, size) = resolve_token_names(&mut db, &token);
    ApiResponse::ok(transform_token(token, token_type, name, size, grid_size_px))
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let update = UpdateTokenPlacement::set_vision(
        vision_bright_ft,
        vision_dim_ft,
        vision_dark_ft,
        light_radius_ft,
    );

    if let Err(e) = dal::update_token_placement(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    // Fetch and return with resolved names
    let token = match dal::get_token_placement(&mut db, &id) {
        Ok(p) => p,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let grid_size_px = get_map_grid_size(&mut db, &state.paths.app_dir, &token.map_id);
    let (token_type, name, size) = resolve_token_names(&mut db, &token);
    ApiResponse::ok(transform_token(token, token_type, name, size, grid_size_px))
}

/// Toggle a token's visibility (hidden from players).
#[tauri::command]
pub fn toggle_token_visibility(
    state: State<'_, AppState>,
    id: String,
) -> ApiResponse<TokenResponse> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Get current state
    let token = match dal::get_token_placement(&mut db, &id) {
        Ok(p) => p,
        Err(e) => return ApiResponse::err(format!("Token not found: {}", e)),
    };

    // Toggle hidden state
    let new_hidden = !token.is_hidden();
    let update = UpdateTokenPlacement::set_hidden(new_hidden);

    if let Err(e) = dal::update_token_placement(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    // Fetch updated token
    let updated_token = match dal::get_token_placement(&mut db, &id) {
        Ok(p) => p,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let grid_size_px = get_map_grid_size(&mut db, &state.paths.app_dir, &updated_token.map_id);
    let (token_type, name, size) = resolve_token_names(&mut db, &updated_token);
    ApiResponse::ok(transform_token(updated_token, token_type, name, size, grid_size_px))
}

/// Delete a token placement.
#[tauri::command]
pub fn delete_token(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match dal::delete_token_placement(&mut db, &id) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Resolve token type and name from monster/NPC references.
/// Returns (token_type, name, size) for a token placement.
fn resolve_token_names(
    db: &mut diesel::SqliteConnection,
    token: &TokenPlacement,
) -> (String, Option<String>, String) {
    if let Some(ref monster_id) = token.module_monster_id {
        let monster = dal::get_module_monster_optional(db, monster_id).ok().flatten();
        if let Some(m) = monster {
            // Look up the monster in the catalog to get its size
            let size = get_monster_by_name(db, &m.monster_name, &m.monster_source)
                .ok()
                .flatten()
                .and_then(|catalog_monster: mimir_core::models::catalog::Monster| {
                    // The monster's size is stored in the size field
                    catalog_monster.size.as_ref().map(|s| normalize_size_code(s))
                })
                .unwrap_or_else(|| "medium".to_string());

            (
                "monster".to_string(),
                Some(m.display_name.unwrap_or(m.monster_name)),
                size,
            )
        } else {
            ("monster".to_string(), None, "medium".to_string())
        }
    } else if let Some(ref npc_id) = token.module_npc_id {
        let npc = dal::get_module_npc_optional(db, npc_id).ok().flatten();
        (
            "npc".to_string(),
            npc.map(|n| n.name),
            "medium".to_string(), // NPCs default to medium
        )
    } else {
        // PC token - use label as name
        ("pc".to_string(), token.label.clone(), "medium".to_string())
    }
}

/// Normalize size codes (T, S, M, L, H, G) to full names
fn normalize_size_code(size: &str) -> String {
    match size.to_uppercase().as_str() {
        "T" => "tiny".to_string(),
        "S" => "small".to_string(),
        "M" => "medium".to_string(),
        "L" => "large".to_string(),
        "H" => "huge".to_string(),
        "G" => "gargantuan".to_string(),
        other => other.to_lowercase(),
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

    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
