//! Character management commands.
//!
//! Provides Tauri commands for creating, updating, and managing player characters.
//! Includes support for character creation wizards, leveling up, spell management,
//! inventory tracking, and character sheet rendering.

use crate::state::AppState;
use mimir_dm_core::models::catalog::Spell;
use mimir_dm_core::models::character::data::ClassLevel;
use mimir_dm_core::models::character::data::{InventoryItem, Personality};
use mimir_dm_core::models::character::{
    Character, CharacterData, CharacterVersion, LegendaryAction, SpellReference,
};
use mimir_dm_core::services::character::creation::{AbilityScoreMethod, CharacterBuilder};
use mimir_dm_core::services::character::level_up::{AsiOrFeat, HpGainMethod, LevelUpOptions};
use mimir_dm_core::services::character::renderer::{CharacterRenderer, MarkdownRenderer};
use mimir_dm_core::services::character::spell_management::RestType;
use mimir_dm_core::services::CharacterService;
use mimir_dm_core::services::{ClassService, ItemService, SpellService};
use std::collections::HashMap;
use tauri::State;
use tracing::error;

// Import types from the types module
use super::types::{
    CreateCharacterRequest, CurrencyUpdate, FeatureDetail, FeatureReferenceInput, LevelUpRequest,
    SpellReferenceInput,
};

/// Create a minimal character for MVP (placeholder until full wizard is implemented).
///
/// Creates a character with default/placeholder ability scores and basic information.
/// Used for quick character creation before the full wizard is available.
///
/// # Parameters
/// - `player_id` - The ID of the player who owns this character
/// - `character_name` - The character's display name
/// - `race` - The character's race (e.g., "Human", "Elf")
/// - `class` - The character's starting class (e.g., "Fighter", "Wizard")
/// - `background` - The character's background (e.g., "Soldier", "Sage")
/// - `campaign_id` - Optional campaign to assign the character to
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The created `Character` record with database ID.
///
/// # Errors
/// Returns an error string if database operations fail.
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn create_minimal_character(
    player_id: Option<i32>,
    character_name: String,
    race: String,
    class: String,
    background: String,
    campaign_id: Option<i32>,
    is_npc: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Character, String> {
    use chrono::Utc;
    use mimir_dm_core::models::character::data::{
        AbilityScores, Appearance, CharacterData, Currency, EquippedItems, Personality, Proficiencies,
        RoleplayNotes, SpellData,
    };

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    // Create minimal character data with placeholder values
    let character_data = CharacterData {
        character_name: character_name.clone(),
        player_id,
        level: 1,
        experience_points: 0,
        version: 1,
        snapshot_reason: Some("Initial character creation".to_string()),
        created_at: Utc::now().to_rfc3339(),
        race,
        subrace: None,
        classes: vec![ClassLevel {
            class_name: class,
            level: 1,
            subclass: None,
            hit_dice_type: "d8".to_string(),
            hit_dice_remaining: 1,
        }],
        background,
        alignment: None,
        abilities: AbilityScores {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        },
        max_hp: 10,
        current_hp: 10,
        speed: 30,
        proficiencies: Proficiencies {
            skills: vec![],
            saves: vec![],
            armor: vec![],
            weapons: vec![],
            tools: vec![],
            languages: vec![],
        },
        class_features: vec![],
        feats: vec![],
        spells: SpellData::default(),
        inventory: vec![],
        currency: Currency::default(),
        equipped: EquippedItems::default(),
        personality: Personality::default(),
        player_name: None,
        appearance: Appearance::default(),
        backstory: None,
        background_feature: None,
        roleplay_notes: RoleplayNotes::default(),
        npc_role: None,
        npc_location: None,
        npc_faction: None,
        npc_notes: None,
        legendary_actions: Vec::new(),
        legendary_action_count: None,
    };

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .create_character(campaign_id, player_id, is_npc.unwrap_or(false), "", character_data)
        .map_err(|e| format!("Failed to create character: {}", e))
}

/// Create a new character with full builder pattern.
///
/// Creates a fully specified character using the CharacterBuilder pattern.
/// Supports all character creation options including race, class, background,
/// ability scores, skills, equipment, and starting spells.
///
/// # Parameters
/// - `request` - Complete character creation request with all options
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The created `CharacterData` with all computed values.
///
/// # Errors
/// Returns an error string if validation fails or database operations fail.
#[tauri::command]
pub async fn create_character(
    request: CreateCharacterRequest,
    state: State<'_, AppState>,
) -> Result<CharacterData, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut builder = CharacterBuilder::new(&mut conn);

    // Set identity
    builder = builder.set_identity(request.character_name, request.player_id);

    // Set race - NPCs can use monster/creature races that aren't in the race catalog
    if request.is_npc.unwrap_or(false) {
        // Use set_race_name_only for NPCs since they may use monster types
        builder = builder.set_race_name_only(&request.race, &request.race_source);
    } else {
        // PCs must use validated races from the catalog
        builder = builder
            .set_race(&request.race, &request.race_source, request.subrace)
            .map_err(|e| format!("Failed to set race: {}", e))?;
    }

    // Set class
    builder = builder
        .set_class(&request.class, &request.class_source, request.subclass)
        .map_err(|e| format!("Failed to set class: {}", e))?;

    // Set background
    builder = builder
        .set_background(&request.background, &request.background_source)
        .map_err(|e| format!("Failed to set background: {}", e))?;

    // Set ability scores - both StandardArray and PointBuy require specifying assignment
    let ability_scores = request
        .ability_scores
        .as_ref()
        .ok_or_else(|| "Ability scores must be specified".to_string())?;

    let ability_method = match request.ability_score_method.as_str() {
        "standard_array" => AbilityScoreMethod::StandardArray {
            strength: ability_scores.strength,
            dexterity: ability_scores.dexterity,
            constitution: ability_scores.constitution,
            intelligence: ability_scores.intelligence,
            wisdom: ability_scores.wisdom,
            charisma: ability_scores.charisma,
        },
        "point_buy" => AbilityScoreMethod::PointBuy {
            strength: ability_scores.strength,
            dexterity: ability_scores.dexterity,
            constitution: ability_scores.constitution,
            intelligence: ability_scores.intelligence,
            wisdom: ability_scores.wisdom,
            charisma: ability_scores.charisma,
        },
        "manual" => AbilityScoreMethod::Manual {
            strength: ability_scores.strength,
            dexterity: ability_scores.dexterity,
            constitution: ability_scores.constitution,
            intelligence: ability_scores.intelligence,
            wisdom: ability_scores.wisdom,
            charisma: ability_scores.charisma,
        },
        _ => {
            return Err(format!(
                "Invalid ability score method: {}",
                request.ability_score_method
            ))
        }
    };
    builder = builder
        .set_ability_scores(ability_method)
        .map_err(|e| format!("Failed to set ability scores: {}", e))?;

    // Set optional fields
    if let Some(alignment) = request.alignment {
        builder = builder.set_alignment(alignment);
    }

    if let Some(personality) = request.personality {
        builder = builder.set_personality(Personality {
            traits: personality.traits,
            ideals: personality.ideals,
            bonds: personality.bonds,
            flaws: personality.flaws,
        });
    }

    if let Some(skills) = request.skill_proficiencies {
        for skill in skills {
            builder = builder.add_skill_proficiency(skill);
        }
    }

    if let Some(equipment) = request.equipment {
        for item in equipment {
            builder = builder.add_equipment(InventoryItem {
                name: item.name,
                source: item.source,
                quantity: item.quantity,
                weight: item.weight,
                value: item.value,
                notes: item.notes,
            });
        }
    }

    // Build and validate
    let mut character_data = builder
        .build()
        .map_err(|e| format!("Failed to build character: {}", e))?;

    // Set spells if provided
    if let Some(cantrips) = request.cantrips {
        character_data.spells.cantrips = cantrips.into_iter().map(|s| s.into()).collect();
    }
    if let Some(known_spells) = request.known_spells {
        character_data.spells.known_spells = known_spells.into_iter().map(|s| s.into()).collect();
    }

    // Set NPC fields if this is an NPC
    if request.is_npc.unwrap_or(false) {
        character_data.npc_role = request.npc_role;
        character_data.npc_location = request.npc_location;
        character_data.npc_faction = request.npc_faction;
        character_data.npc_notes = request.npc_notes;

        // Set legendary actions for boss NPCs
        if let Some(legendary_actions) = request.legendary_actions {
            character_data.legendary_actions = legendary_actions
                .into_iter()
                .map(|a| LegendaryAction {
                    name: a.name,
                    cost: a.cost,
                    description: a.description,
                })
                .collect();
        }
        character_data.legendary_action_count = request.legendary_action_count;
    }

    // Persist to database using CharacterService
    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .create_character(
            None, // campaign_id - not assigned yet
            request.player_id,
            request.is_npc.unwrap_or(false),
            "", // base_directory - empty for unassigned characters
            character_data.clone(),
        )
        .map_err(|e| format!("Failed to save character: {}", e))?;

    Ok(character_data)
}

/// Store a created character in the database.
///
/// Persists character data to the database with optional campaign assignment.
/// Creates the initial character version record.
///
/// # Parameters
/// - `campaign_id` - Optional campaign to assign the character to
/// - `player_id` - Optional ID of the player who owns this character (None for NPCs)
/// - `is_npc` - Whether this is an NPC (true) or PC (false)
/// - `base_directory` - Optional file directory for character files
/// - `character_data` - The complete character data to store
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The created `Character` record with database ID.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn store_character(
    campaign_id: Option<i32>,
    player_id: Option<i32>,
    is_npc: Option<bool>,
    base_directory: Option<String>,
    character_data: CharacterData,
    state: State<'_, AppState>,
) -> Result<Character, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    // Use empty string if no directory provided (for unassigned characters)
    let directory = base_directory.unwrap_or_default();

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .create_character(campaign_id, player_id, is_npc.unwrap_or(false), &directory, character_data)
        .map_err(|e| format!("Failed to store character: {}", e))
}

/// Get character by ID.
///
/// Retrieves a character record and its current data from the database.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `state` - Application state containing the database connection
///
/// # Returns
/// A tuple of (`Character`, `CharacterData`) with the record and current data.
///
/// # Errors
/// Returns an error string if the character is not found or database operations fail.
#[tauri::command]
pub async fn get_character(
    character_id: i32,
    state: State<'_, AppState>,
) -> Result<(Character, CharacterData), String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .get_character(character_id)
        .map_err(|e| format!("Failed to get character: {}", e))
}

/// Get spell slots for a character based on class rules.
///
/// Calculates available spell slots based on the character's class levels
/// and spellcasting progression rules.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `state` - Application state containing the database connection
///
/// # Returns
/// A HashMap mapping spell level to `SpellSlots` with max and current slots.
///
/// # Errors
/// Returns an error string if character lookup or calculation fails.
#[tauri::command]
pub async fn get_character_spell_slots(
    character_id: i32,
    state: State<'_, AppState>,
) -> Result<
    std::collections::HashMap<i32, mimir_dm_core::models::character::data::SpellSlots>,
    String,
> {
    use mimir_dm_core::services::character::calculate_spell_slots;

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    // Get character data
    let mut char_service = CharacterService::new(&mut conn);
    let (_, char_data) = char_service
        .get_character(character_id)
        .map_err(|e| format!("Failed to get character: {}", e))?;

    // Calculate spell slots from class rules
    calculate_spell_slots(&mut conn, &char_data)
        .map_err(|e| format!("Failed to calculate spell slots: {}", e))
}

/// List all characters (including unassigned).
///
/// Retrieves all characters in the database regardless of campaign assignment.
///
/// # Parameters
/// - `state` - Application state containing the database connection
///
/// # Returns
/// A vector of all `Character` records.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn list_all_characters(state: State<'_, AppState>) -> Result<Vec<Character>, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .list_all_characters()
        .map_err(|e| format!("Failed to list characters: {}", e))
}

/// List all characters for a campaign.
///
/// Retrieves all characters assigned to a specific campaign.
///
/// # Parameters
/// - `campaign_id` - The database ID of the campaign
/// - `state` - Application state containing the database connection
///
/// # Returns
/// A vector of `Character` records assigned to the campaign.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn list_characters_for_campaign(
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Character>, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .list_characters_for_campaign(campaign_id)
        .map_err(|e| format!("Failed to list characters: {}", e))
}

/// List all NPCs for a campaign.
///
/// Retrieves all characters marked as NPCs assigned to a specific campaign.
///
/// # Parameters
/// - `campaign_id` - The database ID of the campaign
/// - `state` - Application state containing the database connection
///
/// # Returns
/// A vector of `Character` records that are NPCs in the campaign.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn list_npcs_for_campaign(
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Character>, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .list_npcs_for_campaign(campaign_id)
        .map_err(|e| format!("Failed to list NPCs: {}", e))
}

/// List all player characters (non-NPCs) for a campaign.
///
/// Retrieves all characters that are NOT NPCs assigned to a specific campaign.
///
/// # Parameters
/// - `campaign_id` - The database ID of the campaign
/// - `state` - Application state containing the database connection
///
/// # Returns
/// A vector of `Character` records that are player characters in the campaign.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn list_pcs_for_campaign(
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<Vec<Character>, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .list_pcs_for_campaign(campaign_id)
        .map_err(|e| format!("Failed to list player characters: {}", e))
}

/// Create an NPC (convenience wrapper around create_minimal_character).
///
/// Creates a minimal NPC character for quick NPC generation. This is a convenience
/// command that sets `is_npc = true` by default.
///
/// # Parameters
/// - `character_name` - The NPC's display name
/// - `race` - The NPC's race (e.g., "Human", "Elf")
/// - `class` - The NPC's class (e.g., "Fighter", "Commoner")
/// - `background` - The NPC's background (e.g., "Criminal", "Noble")
/// - `campaign_id` - The campaign to assign the NPC to
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The created `Character` record with database ID.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn create_npc(
    character_name: String,
    race: String,
    class: String,
    background: String,
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<Character, String> {
    create_minimal_character(
        None, // No player for NPCs
        character_name,
        race,
        class,
        background,
        Some(campaign_id),
        Some(true), // is_npc = true
        state,
    )
    .await
}

/// Get all versions of a character.
///
/// Retrieves the version history for a character, showing snapshots
/// created during level-ups, equipment changes, and other updates.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `state` - Application state containing the database connection
///
/// # Returns
/// A vector of `CharacterVersion` records with version metadata.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn get_character_versions(
    character_id: i32,
    state: State<'_, AppState>,
) -> Result<Vec<CharacterVersion>, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .get_character_versions(character_id)
        .map_err(|e| format!("Failed to get character versions: {}", e))
}

/// Get a specific character version.
///
/// Retrieves a specific historical version of character data.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `version` - The version number to retrieve
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The `CharacterData` for the specified version.
///
/// # Errors
/// Returns an error string if the version is not found or database operations fail.
#[tauri::command]
pub async fn get_character_version(
    character_id: i32,
    version: i32,
    state: State<'_, AppState>,
) -> Result<CharacterData, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .get_character_version(character_id, version)
        .map_err(|e| format!("Failed to get character version: {}", e))
}

/// Update character data directly.
///
/// Replaces the character's data with the provided values, creating a new
/// version snapshot for history tracking.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `character_data` - The complete updated character data
/// - `snapshot_reason` - Optional description of why this update was made
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The new `CharacterVersion` record created by this update.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn update_character(
    character_id: i32,
    character_data: CharacterData,
    snapshot_reason: Option<String>,
    state: State<'_, AppState>,
) -> Result<CharacterVersion, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .update_character(character_id, character_data, snapshot_reason)
        .map_err(|e| format!("Failed to update character: {}", e))
}

/// Delete a character.
///
/// Permanently removes a character and all its version history from the database.
///
/// # Parameters
/// - `character_id` - The database ID of the character to delete
/// - `state` - Application state containing the database connection
///
/// # Returns
/// Unit `()` on success.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn delete_character(character_id: i32, state: State<'_, AppState>) -> Result<(), String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .delete_character(character_id)
        .map_err(|e| format!("Failed to delete character: {}", e))
}

/// Assign a character to a campaign.
///
/// Links a character to a campaign and sets up the character's file directory
/// within the campaign's folder structure.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `campaign_id` - The database ID of the campaign to assign to
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The updated `Character` record with campaign assignment.
///
/// # Errors
/// Returns an error string if the campaign is not found or database operations fail.
#[tauri::command]
pub async fn assign_character_to_campaign(
    character_id: i32,
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<Character, String> {
    use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;

    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    // Get campaign directory
    let campaign_directory = {
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .find_by_id(campaign_id)
            .map_err(|e| format!("Failed to find campaign: {}", e))?
            .ok_or_else(|| format!("Campaign with id {} not found", campaign_id))?;
        campaign.directory_path
    };

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .assign_to_campaign(character_id, campaign_id, &campaign_directory)
        .map_err(|e| format!("Failed to assign character to campaign: {}", e))
}

/// Level up a character.
///
/// Advances a character to the next level in the specified class, handling
/// HP increases, ability score improvements or feat selection, subclass
/// choice, and spell updates.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `request` - Level up options including class, HP method, ASI/feat choice
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The new `CharacterVersion` record created by the level up.
///
/// # Errors
/// Returns an error string if validation fails or database operations fail.
#[tauri::command]
pub async fn level_up_character(
    character_id: i32,
    request: LevelUpRequest,
    state: State<'_, AppState>,
) -> Result<CharacterVersion, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    // Parse ability score improvement if provided
    let asi_or_feat = if let Some(asi_json) = request.ability_score_improvement {
        Some(
            serde_json::from_str::<AsiOrFeat>(&asi_json)
                .map_err(|e| format!("Invalid ability score improvement: {}", e))?,
        )
    } else {
        request.feat.map(AsiOrFeat::Feat)
    };

    // Determine HP gain method
    let hp_method = if request.take_average_hp {
        HpGainMethod::Average
    } else if let Some(roll) = request.hit_points_roll {
        HpGainMethod::Roll(roll)
    } else {
        HpGainMethod::Average // default to average if not specified
    };

    // Build level up options
    let options = LevelUpOptions {
        class_name: request.class_name,
        class_source: request.class_source,
        hp_method,
        asi_or_feat,
        subclass_choice: request.subclass,
        snapshot_reason: None,
    };

    let mut char_service = CharacterService::new(&mut conn);
    let result = char_service
        .level_up_character(character_id, options)
        .map_err(|e| format!("Failed to level up character: {}", e))?;

    // Update spells if provided
    if request.new_known_spells.is_some() || request.new_cantrips.is_some() {
        // Get current character data
        let (_, mut char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Failed to get character for spell update: {}", e))?;

        // Update cantrips if provided
        if let Some(cantrips) = request.new_cantrips {
            char_data.spells.cantrips = cantrips.into_iter().map(|s| s.into()).collect();
        }

        // Update known spells if provided
        if let Some(known) = request.new_known_spells {
            char_data.spells.known_spells = known.into_iter().map(|s| s.into()).collect();
        }

        // Save the updated character
        char_service
            .update_character(character_id, char_data, Some("Spell selection".to_string()))
            .map_err(|e| format!("Failed to update spells: {}", e))?;
    }

    Ok(result)
}

/// Add a spell to known spells.
///
/// Adds a spell to the character's known spells or cantrips list.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `spell_name` - Name of the spell to add
/// - `spell_source` - Source book of the spell (e.g., "PHB")
/// - `is_cantrip` - Whether this is a cantrip (true) or leveled spell (false)
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The new `CharacterVersion` record created by this change.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn add_spell_to_known(
    character_id: i32,
    spell_name: String,
    spell_source: String,
    is_cantrip: bool,
    state: State<'_, AppState>,
) -> Result<CharacterVersion, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .add_spell_to_known(character_id, &spell_name, &spell_source, is_cantrip)
        .map_err(|e| format!("Failed to add spell: {}", e))
}

/// Prepare spells for the day.
///
/// Sets the character's list of prepared spells for classes that prepare
/// spells daily (Clerics, Druids, Paladins, Wizards).
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `spell_names` - List of spell names to prepare
/// - `spellcasting_ability` - The ability used for spellcasting (e.g., "wisdom")
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The new `CharacterVersion` record created by this change.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn prepare_spells(
    character_id: i32,
    spells: Vec<SpellReferenceInput>,
    spellcasting_ability: String,
    state: State<'_, AppState>,
) -> Result<CharacterVersion, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let spell_refs: Vec<SpellReference> = spells.into_iter().map(|s| s.into()).collect();
    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .prepare_spells(character_id, spell_refs, &spellcasting_ability)
        .map_err(|e| format!("Failed to prepare spells: {}", e))
}

/// Cast a spell (expends spell slot).
///
/// Records spell casting and expends the appropriate spell slot.
/// Ritual casting does not expend a slot.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `spell_name` - Name of the spell being cast
/// - `spell_level` - Level to cast the spell at (for upcasting)
/// - `is_ritual` - Whether casting as a ritual (no slot expended)
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The new `CharacterVersion` record with updated spell slots.
///
/// # Errors
/// Returns an error string if no slot is available or database operations fail.
#[tauri::command]
pub async fn cast_spell(
    character_id: i32,
    spell_name: String,
    spell_level: i32,
    is_ritual: bool,
    state: State<'_, AppState>,
) -> Result<CharacterVersion, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .cast_spell(character_id, &spell_name, spell_level, is_ritual)
        .map_err(|e| format!("Failed to cast spell: {}", e))
}

/// Take a rest to recover resources.
///
/// Processes rest effects: short rests allow hit dice spending, long rests
/// restore HP, spell slots, and other daily resources.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `rest_type` - Type of rest ("short" or "long")
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The new `CharacterVersion` record with restored resources.
///
/// # Errors
/// Returns an error string if rest type is invalid or database operations fail.
#[tauri::command]
pub async fn take_rest(
    character_id: i32,
    rest_type: String, // "short" or "long"
    state: State<'_, AppState>,
) -> Result<CharacterVersion, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let rest = match rest_type.as_str() {
        "short" => RestType::Short,
        "long" => RestType::Long,
        _ => return Err(format!("Invalid rest type: {}", rest_type)),
    };

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .rest(character_id, rest)
        .map_err(|e| format!("Failed to rest: {}", e))
}

/// Add an item to character inventory.
///
/// Adds an item to the character's inventory with the specified quantity.
/// If the item already exists, the quantities are combined.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `item_name` - Name of the item to add
/// - `item_source` - Source book of the item (e.g., "PHB")
/// - `quantity` - Number of items to add
/// - `notes` - Optional notes about the item
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The new `CharacterVersion` record with updated inventory.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn add_item_to_inventory(
    character_id: i32,
    item_name: String,
    item_source: String,
    quantity: i32,
    notes: Option<String>,
    state: State<'_, AppState>,
) -> Result<CharacterVersion, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .add_item(character_id, &item_name, &item_source, quantity, notes)
        .map_err(|e| format!("Failed to add item: {}", e))
}

/// Remove an item from character inventory.
///
/// Removes the specified quantity of an item. If quantity exceeds
/// current amount, the item is removed entirely.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `item_name` - Name of the item to remove
/// - `quantity` - Number of items to remove
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The new `CharacterVersion` record with updated inventory.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn remove_item_from_inventory(
    character_id: i32,
    item_name: String,
    quantity: i32,
    state: State<'_, AppState>,
) -> Result<CharacterVersion, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .remove_item(character_id, &item_name, quantity)
        .map_err(|e| format!("Failed to remove item: {}", e))
}

/// Update character currency.
///
/// Sets the character's currency values. Omitted denominations remain unchanged.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `currency` - Currency values to update (copper, silver, electrum, gold, platinum)
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The new `CharacterVersion` record with updated currency.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn update_character_currency(
    character_id: i32,
    currency: CurrencyUpdate,
    state: State<'_, AppState>,
) -> Result<CharacterVersion, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .update_currency(
            character_id,
            currency.copper.unwrap_or(0),
            currency.silver.unwrap_or(0),
            currency.electrum.unwrap_or(0),
            currency.gold.unwrap_or(0),
            currency.platinum.unwrap_or(0),
        )
        .map_err(|e| format!("Failed to update currency: {}", e))
}

/// Update character equipped items.
///
/// Sets the character's currently equipped armor, shield, and weapons.
/// Omitted slots remain unchanged.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `armor` - Optional armor item name
/// - `shield` - Optional shield item name
/// - `main_hand` - Optional main hand weapon name
/// - `off_hand` - Optional off hand weapon/shield name
/// - `state` - Application state containing the database connection
///
/// # Returns
/// The new `CharacterVersion` record with updated equipment.
///
/// # Errors
/// Returns an error string if database operations fail.
#[tauri::command]
pub async fn update_character_equipped(
    character_id: i32,
    armor: Option<String>,
    shield: Option<String>,
    main_hand: Option<String>,
    off_hand: Option<String>,
    state: State<'_, AppState>,
) -> Result<CharacterVersion, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    char_service
        .update_equipped(character_id, armor, shield, main_hand, off_hand)
        .map_err(|e| format!("Failed to update equipped items: {}", e))
}

/// Render character sheet as markdown.
///
/// Generates a complete markdown representation of the character sheet
/// with full spell and item details fetched from the catalog.
///
/// # Parameters
/// - `character_id` - The database ID of the character
/// - `state` - Application state containing the database connection
///
/// # Returns
/// A formatted markdown string representing the character sheet.
///
/// # Errors
/// Returns an error string if character lookup fails or database operations fail.
#[tauri::command]
pub async fn render_character_sheet(
    character_id: i32,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut char_service = CharacterService::new(&mut conn);
    let (_character, char_data) = char_service
        .get_character(character_id)
        .map_err(|e| format!("Failed to get character: {}", e))?;

    // Fetch spell details for all character spells
    let mut spell_details: HashMap<String, Spell> = HashMap::new();

    // Collect all spell references
    let mut all_spells: Vec<&SpellReference> = Vec::new();
    all_spells.extend(char_data.spells.cantrips.iter());
    all_spells.extend(char_data.spells.known_spells.iter());

    // Fetch details for each spell from catalog (now we have source info!)
    for spell_ref in all_spells {
        if let Ok(Some(spell)) =
            SpellService::get_spell_details(&mut conn, &spell_ref.name, &spell_ref.source)
        {
            spell_details.insert(spell_ref.name.clone(), spell);
        }
    }

    // Fetch item details for all inventory items
    use mimir_dm_core::models::catalog::Item;
    let mut item_details: HashMap<String, Item> = HashMap::new();
    let mut item_service = ItemService::new(&mut conn);

    for item in &char_data.inventory {
        let source = item.source.as_deref().unwrap_or("PHB");
        let key = format!("{}:{}", item.name, source);

        if let Ok(Some(details)) = item_service.get_item_by_name_and_source(&item.name, source) {
            item_details.insert(key, details);
        }
    }

    let renderer = MarkdownRenderer::new();
    Ok(renderer.render_with_details(&char_data, &spell_details, &item_details))
}

/// Write text to a file.
///
/// Utility command to write text content to a file on disk.
/// Used for exporting character sheets and other documents.
///
/// # Parameters
/// - `path` - The file path to write to
/// - `contents` - The text content to write
///
/// # Returns
/// Unit `()` on success.
///
/// # Errors
/// Returns an error string if the file cannot be written.
#[tauri::command]
pub async fn write_text_file(path: String, contents: String) -> Result<(), String> {
    std::fs::write(&path, contents).map_err(|e| format!("Failed to write file: {}", e))
}

/// Helper to extract description text from typed entries array
fn extract_description_from_entries(entries: &[mimir_dm_core::models::catalog::types::Entry]) -> String {
    use mimir_dm_core::models::catalog::types::{Entry, EntryObject};
    let mut descriptions: Vec<String> = Vec::new();

    for entry in entries {
        match entry {
            Entry::Text(s) => {
                descriptions.push(s.clone());
            }
            Entry::Object(obj) => match obj {
                EntryObject::Entries { entries, .. } => {
                    let sub_desc = extract_description_from_entries(entries);
                    if !sub_desc.is_empty() {
                        descriptions.push(sub_desc);
                    }
                }
                EntryObject::List { items, .. } => {
                    for item in items {
                        if let Entry::Text(s) = item {
                            descriptions.push(format!("- {}", s));
                        }
                    }
                }
                EntryObject::Item { name, entries, .. } => {
                    if let Some(entries) = entries {
                        let sub_desc = extract_description_from_entries(entries);
                        if !sub_desc.is_empty() {
                            descriptions.push(format!("**{}:** {}", name, sub_desc));
                        }
                    }
                }
                _ => {}
            },
        }
    }

    descriptions.join("\n\n")
}

/// Get feature details from the catalog.
///
/// Fetches full feature details including descriptions for a list of feature references.
///
/// # Parameters
/// - `features` - Array of feature references to look up
/// - `state` - Application state containing the database connection
///
/// # Returns
/// Array of feature details with descriptions.
#[tauri::command]
pub async fn get_feature_details(
    features: Vec<FeatureReferenceInput>,
    state: State<'_, AppState>,
) -> Result<Vec<FeatureDetail>, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut class_service = ClassService::new(&mut conn);
    let mut details: Vec<FeatureDetail> = Vec::new();

    for feature_ref in features {
        if let Some(ref subclass_name) = feature_ref.subclass_name {
            // Try to fetch as subclass feature
            match class_service.get_subclass_feature(
                &feature_ref.name,
                &feature_ref.class_name,
                subclass_name,
                &feature_ref.source,
            ) {
                Ok(Some(feature)) => {
                    let description = extract_description_from_entries(&feature.entries);
                    details.push(FeatureDetail {
                        name: feature.name,
                        class_name: feature.class_name,
                        subclass_name: feature.subclass_short_name,
                        source: feature.source,
                        level: feature.level as i32,
                        description,
                    });
                }
                Ok(None) => {
                    // Feature not found in catalog, return with empty description
                    details.push(FeatureDetail {
                        name: feature_ref.name,
                        class_name: feature_ref.class_name,
                        subclass_name: feature_ref.subclass_name,
                        source: feature_ref.source,
                        level: feature_ref.level,
                        description: String::new(),
                    });
                }
                Err(e) => {
                    error!("Failed to fetch subclass feature {}: {}", feature_ref.name, e);
                }
            }
        } else {
            // Fetch as class feature
            match class_service.get_class_feature(
                &feature_ref.name,
                &feature_ref.class_name,
                &feature_ref.source,
            ) {
                Ok(Some(feature)) => {
                    let description = extract_description_from_entries(&feature.entries);
                    details.push(FeatureDetail {
                        name: feature.name,
                        class_name: feature.class_name,
                        subclass_name: None,
                        source: feature.source,
                        level: feature.level as i32,
                        description,
                    });
                }
                Ok(None) => {
                    // Feature not found in catalog, return with empty description
                    details.push(FeatureDetail {
                        name: feature_ref.name,
                        class_name: feature_ref.class_name,
                        subclass_name: None,
                        source: feature_ref.source,
                        level: feature_ref.level,
                        description: String::new(),
                    });
                }
                Err(e) => {
                    error!("Failed to fetch class feature {}: {}", feature_ref.name, e);
                }
            }
        }
    }

    Ok(details)
}
