//! Character Commands
//!
//! Tauri commands for character management (PCs and NPCs).

use mimir_core::models::campaign::{Character, CharacterInventory};
use mimir_core::services::{
    AddInventoryInput, CharacterService, CreateCharacterInput, UpdateCharacterInput,
};
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

// =============================================================================
// List Commands
// =============================================================================

/// List all characters for a campaign.
#[tauri::command]
pub fn list_characters(
    state: State<'_, AppState>,
    campaign_id: String,
) -> ApiResponse<Vec<Character>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CharacterService::new(&mut db).list_for_campaign(&campaign_id);
    to_api_response(result)
}

/// List only player characters for a campaign.
#[tauri::command]
pub fn list_pcs(state: State<'_, AppState>, campaign_id: String) -> ApiResponse<Vec<Character>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CharacterService::new(&mut db).list_pcs(&campaign_id);
    to_api_response(result)
}

/// List only NPCs for a campaign.
#[tauri::command]
pub fn list_npcs(state: State<'_, AppState>, campaign_id: String) -> ApiResponse<Vec<Character>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CharacterService::new(&mut db).list_npcs(&campaign_id);
    to_api_response(result)
}

// =============================================================================
// CRUD Commands
// =============================================================================

/// Get a character by ID.
#[tauri::command]
pub fn get_character(state: State<'_, AppState>, id: String) -> ApiResponse<Character> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CharacterService::new(&mut db).get(&id);
    match result {
        Ok(Some(character)) => ApiResponse::ok(character),
        Ok(None) => ApiResponse::err(format!("Character not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for creating a new PC.
#[derive(Debug, serde::Deserialize)]
pub struct CreatePcRequest {
    pub campaign_id: String,
    pub name: String,
    pub player_name: String,
    pub race_name: Option<String>,
    pub race_source: Option<String>,
    pub background_name: Option<String>,
    pub background_source: Option<String>,
    pub ability_scores: Option<[i32; 6]>,
}

/// Create a new player character.
#[tauri::command]
pub fn create_pc(state: State<'_, AppState>, request: CreatePcRequest) -> ApiResponse<Character> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut input =
        CreateCharacterInput::new_pc(&request.campaign_id, &request.name, &request.player_name);

    // Set race if both name and source provided
    if let (Some(name), Some(source)) = (&request.race_name, &request.race_source) {
        input = input.with_race(name, source);
    }

    // Set background if both name and source provided
    if let (Some(name), Some(source)) = (&request.background_name, &request.background_source) {
        input = input.with_background(name, source);
    }

    // Set ability scores if provided
    if let Some(scores) = request.ability_scores {
        input = input.with_ability_scores(scores);
    }

    let result = CharacterService::new(&mut db).create(input);
    to_api_response(result)
}

/// Request for creating a new NPC.
#[derive(Debug, serde::Deserialize)]
pub struct CreateNpcRequest {
    pub campaign_id: String,
    pub name: String,
    pub race_name: Option<String>,
    pub race_source: Option<String>,
    pub role: Option<String>,
    pub location: Option<String>,
    pub faction: Option<String>,
}

/// Create a new NPC.
#[tauri::command]
pub fn create_npc(state: State<'_, AppState>, request: CreateNpcRequest) -> ApiResponse<Character> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut input = CreateCharacterInput::new_npc(&request.campaign_id, &request.name);

    // Set race if both name and source provided
    if let (Some(name), Some(source)) = (&request.race_name, &request.race_source) {
        input = input.with_race(name, source);
    }

    // Create the NPC first
    let result = CharacterService::new(&mut db).create(input);
    let character = match result {
        Ok(c) => c,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // Update with NPC-specific fields if provided
    if request.role.is_some() || request.location.is_some() || request.faction.is_some() {
        let update = UpdateCharacterInput::set_npc_info(
            request.role,
            request.location,
            request.faction,
        );
        let result = CharacterService::new(&mut db).update(&character.id, update);
        return to_api_response(result);
    }

    ApiResponse::ok(character)
}

/// Request for updating a character.
#[derive(Debug, serde::Deserialize)]
pub struct UpdateCharacterRequest {
    pub name: Option<String>,
    pub player_name: Option<Option<String>>,
    pub race_name: Option<Option<String>>,
    pub race_source: Option<Option<String>>,
    pub background_name: Option<Option<String>>,
    pub background_source: Option<Option<String>>,
    pub ability_scores: Option<[i32; 6]>,
    pub currency: Option<[i32; 5]>,
    pub traits: Option<Option<String>>,
    pub ideals: Option<Option<String>>,
    pub bonds: Option<Option<String>>,
    pub flaws: Option<Option<String>>,
    pub role: Option<Option<String>>,
    pub location: Option<Option<String>>,
    pub faction: Option<Option<String>>,
}

/// Update a character.
#[tauri::command]
pub fn update_character(
    state: State<'_, AppState>,
    id: String,
    request: UpdateCharacterRequest,
) -> ApiResponse<Character> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let input = UpdateCharacterInput {
        name: request.name,
        player_name: request.player_name,
        race_name: request.race_name,
        race_source: request.race_source,
        background_name: request.background_name,
        background_source: request.background_source,
        ability_scores: request.ability_scores,
        currency: request.currency,
        traits: request.traits,
        ideals: request.ideals,
        bonds: request.bonds,
        flaws: request.flaws,
        role: request.role,
        location: request.location,
        faction: request.faction,
    };

    let result = CharacterService::new(&mut db).update(&id, input);
    to_api_response(result)
}

/// Delete a character permanently.
#[tauri::command]
pub fn delete_character(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CharacterService::new(&mut db).delete(&id);
    to_api_response(result)
}

// =============================================================================
// Inventory Commands
// =============================================================================

/// Get a character's inventory.
#[tauri::command]
pub fn get_character_inventory(
    state: State<'_, AppState>,
    character_id: String,
) -> ApiResponse<Vec<CharacterInventory>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CharacterService::new(&mut db).get_inventory(&character_id);
    to_api_response(result)
}

/// Get equipped items for a character.
#[tauri::command]
pub fn get_equipped_items(
    state: State<'_, AppState>,
    character_id: String,
) -> ApiResponse<Vec<CharacterInventory>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CharacterService::new(&mut db).get_equipped_items(&character_id);
    to_api_response(result)
}

/// Get attuned items for a character.
#[tauri::command]
pub fn get_attuned_items(
    state: State<'_, AppState>,
    character_id: String,
) -> ApiResponse<Vec<CharacterInventory>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CharacterService::new(&mut db).get_attuned_items(&character_id);
    to_api_response(result)
}

/// Request for adding an item to inventory.
#[derive(Debug, serde::Deserialize)]
pub struct AddInventoryRequest {
    pub item_name: String,
    pub item_source: String,
    pub quantity: Option<i32>,
    pub equipped: Option<bool>,
    pub attuned: Option<bool>,
    pub notes: Option<String>,
}

/// Add an item to a character's inventory.
#[tauri::command]
pub fn add_inventory_item(
    state: State<'_, AppState>,
    character_id: String,
    request: AddInventoryRequest,
) -> ApiResponse<CharacterInventory> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut input = AddInventoryInput::new(&request.item_name, &request.item_source);

    if let Some(qty) = request.quantity {
        input = input.with_quantity(qty);
    }
    if request.equipped.unwrap_or(false) {
        input = input.equipped();
    }
    if request.attuned.unwrap_or(false) {
        input = input.attuned();
    }
    if let Some(notes) = request.notes {
        input = input.with_notes(notes);
    }

    let result = CharacterService::new(&mut db).add_to_inventory(&character_id, input);
    to_api_response(result)
}

/// Remove an item from a character's inventory.
#[tauri::command]
pub fn remove_inventory_item(state: State<'_, AppState>, inventory_id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CharacterService::new(&mut db).remove_from_inventory(&inventory_id);
    to_api_response(result)
}

/// Request for updating an inventory item.
#[derive(Debug, serde::Deserialize)]
pub struct UpdateInventoryRequest {
    pub quantity: Option<i32>,
    pub equipped: Option<bool>,
    pub attuned: Option<bool>,
}

/// Update an inventory item (quantity, equipped, attuned).
#[tauri::command]
pub fn update_inventory_item(
    state: State<'_, AppState>,
    inventory_id: String,
    request: UpdateInventoryRequest,
) -> ApiResponse<CharacterInventory> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CharacterService::new(&mut db).update_inventory_item(
        &inventory_id,
        request.quantity,
        request.equipped,
        request.attuned,
    );
    to_api_response(result)
}

