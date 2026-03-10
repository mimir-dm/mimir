//! Character Commands
//!
//! Tauri commands for character management (PCs and NPCs).

use mimir_core::dal::campaign as dal;
use mimir_core::models::campaign::{
    CharacterInventory, CharacterResponse, CharacterSpell, NewCharacterSpell, UpdateCharacterSpell,
};
use mimir_core::services::{
    AddInventoryInput, CharacterService, CreateCharacterInput, LevelUpRequest, LevelUpResult,
    UpdateCharacterInput,
};
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

// =============================================================================
// List Commands
// =============================================================================

/// List all characters for a campaign (with classes and proficiencies).
#[tauri::command]
pub fn list_characters(
    state: State<'_, AppState>,
    campaign_id: String,
) -> ApiResponse<Vec<CharacterResponse>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let mut service = CharacterService::new(&mut db);
    let characters = match service.list_for_campaign(&campaign_id) {
        Ok(chars) => chars,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    match service.enrich_many(characters) {
        Ok(result) => ApiResponse::ok(result),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List only player characters for a campaign (with classes and proficiencies).
#[tauri::command]
pub fn list_pcs(state: State<'_, AppState>, campaign_id: String) -> ApiResponse<Vec<CharacterResponse>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let mut service = CharacterService::new(&mut db);
    let characters = match service.list_pcs(&campaign_id) {
        Ok(chars) => chars,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    match service.enrich_many(characters) {
        Ok(result) => ApiResponse::ok(result),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List only NPCs for a campaign (with classes and proficiencies).
#[tauri::command]
pub fn list_npcs(state: State<'_, AppState>, campaign_id: String) -> ApiResponse<Vec<CharacterResponse>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let mut service = CharacterService::new(&mut db);
    let characters = match service.list_npcs(&campaign_id) {
        Ok(chars) => chars,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    match service.enrich_many(characters) {
        Ok(result) => ApiResponse::ok(result),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List unassigned player characters (no campaign).
#[tauri::command]
pub fn list_unassigned_pcs(state: State<'_, AppState>) -> ApiResponse<Vec<CharacterResponse>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let characters = match dal::list_unassigned_pcs(&mut db) {
        Ok(chars) => chars,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    match CharacterService::new(&mut db).enrich_many(characters) {
        Ok(result) => ApiResponse::ok(result),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// CRUD Commands
// =============================================================================

/// Get a character by ID (with classes and proficiencies).
#[tauri::command]
pub fn get_character(state: State<'_, AppState>, id: String) -> ApiResponse<CharacterResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match CharacterService::new(&mut db).get_enriched(&id) {
        Ok(Some(response)) => ApiResponse::ok(response),
        Ok(None) => ApiResponse::err(format!("Character not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for creating a new PC.
#[derive(Debug, serde::Deserialize)]
pub struct CreatePcRequest {
    pub campaign_id: Option<String>,
    pub name: String,
    pub player_name: String,
    pub race_name: Option<String>,
    pub race_source: Option<String>,
    pub background_name: Option<String>,
    pub background_source: Option<String>,
    pub ability_scores: Option<[i32; 6]>,
    pub class_name: Option<String>,
    pub class_source: Option<String>,
    pub selected_skills: Option<Vec<String>>,
}

/// Create a new player character.
#[tauri::command]
pub fn create_pc(state: State<'_, AppState>, request: CreatePcRequest) -> ApiResponse<CharacterResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let mut input =
        CreateCharacterInput::new_pc(request.campaign_id.as_deref(), &request.name, &request.player_name);

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

    // Set class if both name and source provided
    if let (Some(name), Some(source)) = (&request.class_name, &request.class_source) {
        input = input.with_class(name, source);
    }

    // Set selected skills if provided
    if let Some(skills) = request.selected_skills {
        input = input.with_skills(skills);
    }

    match CharacterService::new(&mut db).create(input) {
        Ok(character) => {
            // Enrich response with proficiencies
            match CharacterService::new(&mut db).enrich(character) {
                Ok(response) => ApiResponse::ok(response),
                Err(e) => ApiResponse::err(e.to_string()),
            }
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for creating a new NPC.
#[derive(Debug, serde::Deserialize)]
pub struct CreateNpcRequest {
    pub campaign_id: Option<String>,
    pub name: String,
    pub race_name: Option<String>,
    pub race_source: Option<String>,
    pub role: Option<String>,
    pub location: Option<String>,
    pub faction: Option<String>,
}

/// Create a new NPC.
#[tauri::command]
pub fn create_npc(state: State<'_, AppState>, request: CreateNpcRequest) -> ApiResponse<CharacterResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let mut input = CreateCharacterInput::new_npc(request.campaign_id.as_deref(), &request.name);

    // Set race if both name and source provided
    if let (Some(name), Some(source)) = (&request.race_name, &request.race_source) {
        input = input.with_race(name, source);
    }

    // Create the NPC first
    let character = match CharacterService::new(&mut db).create(input) {
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
        match CharacterService::new(&mut db).update(&character.id, update) {
            Ok(updated) => {
                return ApiResponse::ok(CharacterResponse::from_character(updated, vec![], vec![]))
            }
            Err(e) => return ApiResponse::err(e.to_string()),
        }
    }

    ApiResponse::ok(CharacterResponse::from_character(character, vec![], vec![]))
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
) -> ApiResponse<CharacterResponse> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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

    let mut service = CharacterService::new(&mut db);
    match service.update(&id, input) {
        Ok(character) => match service.enrich(character) {
            Ok(response) => ApiResponse::ok(response),
            Err(e) => ApiResponse::err(e.to_string()),
        },
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete a character permanently.
#[tauri::command]
pub fn delete_character(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CharacterService::new(&mut db).delete(&id);
    to_api_response(result)
}

/// Assign a character to a campaign.
#[tauri::command]
pub fn assign_character_to_campaign(
    state: State<'_, AppState>,
    character_id: String,
    campaign_id: String,
) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match dal::assign_character_to_campaign(&mut db, &character_id, &campaign_id) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Level Up Commands
// =============================================================================

/// Level up a character.
///
/// Handles HP calculation, multiclass validation, class level updates, ASI/feats, and subclass selection.
#[tauri::command]
pub fn level_up_character(
    state: State<'_, AppState>,
    character_id: String,
    request: LevelUpRequest,
) -> ApiResponse<LevelUpResult> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CharacterService::new(&mut db).level_up(&character_id, request);
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CharacterService::new(&mut db).update_inventory_item(
        &inventory_id,
        request.quantity,
        request.equipped,
        request.attuned,
    );
    to_api_response(result)
}

// =============================================================================
// Character Source Commands
// =============================================================================

use mimir_core::dal::campaign::{
    delete_all_character_sources, delete_character_source_by_code, insert_character_source,
    list_character_source_codes,
};
use mimir_core::models::campaign::{CharacterSource, NewCharacterSource};

/// List allowed source codes for a character.
#[tauri::command]
pub fn list_character_sources(
    state: State<'_, AppState>,
    character_id: String,
) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match list_character_source_codes(&mut db, &character_id) {
        Ok(codes) => ApiResponse::ok(codes),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Add a source to a character's allowed sources.
#[tauri::command]
pub fn add_character_source(
    state: State<'_, AppState>,
    character_id: String,
    source_code: String,
) -> ApiResponse<CharacterSource> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::<CharacterSource>::err(e),
    };

    let id = uuid::Uuid::new_v4().to_string();
    let source = NewCharacterSource::new(&id, &character_id, &source_code);

    match insert_character_source(&mut db, &source) {
        Ok(_) => ApiResponse::ok(CharacterSource {
            id,
            character_id,
            source_code,
        }),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Remove a source from a character's allowed sources.
#[tauri::command]
pub fn remove_character_source(
    state: State<'_, AppState>,
    character_id: String,
    source_code: String,
) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match delete_character_source_by_code(&mut db, &character_id, &source_code) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Set the complete list of allowed sources for a character (replaces existing).
#[tauri::command]
pub fn set_character_sources(
    state: State<'_, AppState>,
    character_id: String,
    source_codes: Vec<String>,
) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::<Vec<String>>::err(e),
    };

    // Delete all existing sources
    if let Err(e) = delete_all_character_sources(&mut db, &character_id) {
        return ApiResponse::err(format!("Failed to clear sources: {}", e));
    }

    // Insert all new sources
    for source_code in &source_codes {
        let id = uuid::Uuid::new_v4().to_string();
        let source = NewCharacterSource::new(&id, &character_id, source_code);
        if let Err(e) = insert_character_source(&mut db, &source) {
            return ApiResponse::err(format!("Failed to add source {}: {}", source_code, e));
        }
    }

    ApiResponse::ok(source_codes)
}

// =============================================================================
// Spell Commands
// =============================================================================

/// List all spells a character knows.
#[tauri::command]
pub fn list_character_spells(
    state: State<'_, AppState>,
    character_id: String,
) -> ApiResponse<Vec<CharacterSpell>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match dal::list_character_spells(&mut db, &character_id) {
        Ok(spells) => ApiResponse::ok(spells),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Add a spell to a character's known spells.
#[tauri::command]
pub fn add_character_spell(
    state: State<'_, AppState>,
    character_id: String,
    spell_name: String,
    spell_source: String,
    source_class: String,
    prepared: Option<bool>,
) -> ApiResponse<CharacterSpell> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Check if character already knows this spell from the same class
    if let Ok(Some(_)) =
        dal::find_character_spell_by_name(&mut db, &character_id, &spell_name, &source_class)
    {
        return ApiResponse::err(format!(
            "Character already knows {} from {}",
            spell_name, source_class
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let mut spell = NewCharacterSpell::new(&id, &character_id, &spell_name, &spell_source, &source_class);
    if prepared.unwrap_or(false) {
        spell = spell.prepared();
    }

    match dal::insert_character_spell(&mut db, &spell) {
        Ok(_) => ApiResponse::ok(CharacterSpell {
            id,
            character_id,
            spell_name,
            spell_source,
            source_class,
            prepared: if prepared.unwrap_or(false) { 1 } else { 0 },
        }),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Remove a spell from a character's known spells.
#[tauri::command]
pub fn remove_character_spell(
    state: State<'_, AppState>,
    character_id: String,
    spell_name: String,
    source_class: Option<String>,
) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    if let Some(class) = source_class {
        // Remove specific class's version
        match dal::find_character_spell_by_name(&mut db, &character_id, &spell_name, &class) {
            Ok(Some(spell)) => match dal::delete_character_spell(&mut db, &spell.id) {
                Ok(_) => ApiResponse::ok(()),
                Err(e) => ApiResponse::err(e.to_string()),
            },
            Ok(None) => ApiResponse::err(format!(
                "Character doesn't know {} from {}",
                spell_name, class
            )),
            Err(e) => ApiResponse::err(e.to_string()),
        }
    } else {
        // Remove all instances of this spell (any class)
        let spells = match dal::list_character_spells(&mut db, &character_id) {
            Ok(s) => s,
            Err(e) => return ApiResponse::err(e.to_string()),
        };
        let matching: Vec<_> = spells
            .iter()
            .filter(|s| s.spell_name.to_lowercase() == spell_name.to_lowercase())
            .collect();
        if matching.is_empty() {
            return ApiResponse::err(format!("Character doesn't know {}", spell_name));
        }
        for spell in matching {
            if let Err(e) = dal::delete_character_spell(&mut db, &spell.id) {
                return ApiResponse::err(e.to_string());
            }
        }
        ApiResponse::ok(())
    }
}

/// Toggle a spell's prepared status.
#[tauri::command]
pub fn toggle_spell_prepared(
    state: State<'_, AppState>,
    spell_id: String,
) -> ApiResponse<CharacterSpell> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get current state
    let spell = match dal::get_character_spell_optional(&mut db, &spell_id) {
        Ok(Some(s)) => s,
        Ok(None) => return ApiResponse::err(format!("Spell entry not found: {}", spell_id)),
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let new_prepared = !spell.is_prepared();
    let update = UpdateCharacterSpell::set_prepared(new_prepared);

    match dal::update_character_spell(&mut db, &spell_id, &update) {
        Ok(_) => ApiResponse::ok(CharacterSpell {
            prepared: if new_prepared { 1 } else { 0 },
            ..spell
        }),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}
