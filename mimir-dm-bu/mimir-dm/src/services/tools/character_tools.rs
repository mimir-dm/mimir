//! Character tools for LLM interactions
//!
//! Provides read-only query access to character data during chat sessions

use async_trait::async_trait;
use mimir_dm_core::models::character::CharacterData;
use mimir_dm_core::services::player_service::PlayerService;
use mimir_dm_core::{services::CharacterService, DatabaseService};
use mimir_dm_llm::ToolTrait;
use serde_json::{json, Value};
use std::error::Error;
use std::sync::Arc;
use tracing::{debug, error};

// ============================================================================
// Helper Functions - Reduce code duplication across tools
// ============================================================================

/// Extract an i32 parameter from JSON arguments
fn extract_i32_param(arguments: &Value, param_name: &str) -> Result<i32, String> {
    arguments
        .get(param_name)
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or_else(|| format!("Missing '{}' parameter", param_name))
}

/// Format character abilities as JSON with scores and modifiers
fn format_abilities_json(char_data: &CharacterData) -> Value {
    json!({
        "strength": {
            "score": char_data.abilities.strength,
            "modifier": char_data.abilities.str_modifier()
        },
        "dexterity": {
            "score": char_data.abilities.dexterity,
            "modifier": char_data.abilities.dex_modifier()
        },
        "constitution": {
            "score": char_data.abilities.constitution,
            "modifier": char_data.abilities.con_modifier()
        },
        "intelligence": {
            "score": char_data.abilities.intelligence,
            "modifier": char_data.abilities.int_modifier()
        },
        "wisdom": {
            "score": char_data.abilities.wisdom,
            "modifier": char_data.abilities.wis_modifier()
        },
        "charisma": {
            "score": char_data.abilities.charisma,
            "modifier": char_data.abilities.cha_modifier()
        }
    })
}

/// Format a character summary for list endpoints
fn format_character_summary(id: i32, char_data: &CharacterData, include_player_id: Option<i32>) -> Value {
    let mut summary = json!({
        "id": id,
        "character_name": char_data.character_name,
        "level": char_data.level,
        "race": char_data.race,
        "class": char_data.primary_class_name(),
        "hit_points": {
            "current": char_data.current_hp,
            "maximum": char_data.max_hp
        }
    });

    if let Some(player_id) = include_player_id {
        summary["player_id"] = json!(player_id);
    }

    summary
}

/// Tool for retrieving full character data by ID
pub struct GetCharacterTool {
    db_service: Arc<DatabaseService>,
}

impl GetCharacterTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for GetCharacterTool {
    fn name(&self) -> &str {
        "get_character"
    }

    fn description(&self) -> &str {
        "Retrieve full character data including stats, abilities, spells, inventory, and equipment.

Usage:
- Provide character_id (numeric ID from list_campaign_characters)
- Returns complete character sheet in JSON format
- Always returns the latest version of the character
- Includes: race, class, level, abilities, HP, skills, spells, inventory

When to use:
- When you need detailed information about a specific character
- Before answering questions about character capabilities
- When comparing character stats or equipment
- For detailed character analysis

Output includes:
- Core stats: STR, DEX, CON, INT, WIS, CHA with modifiers
- Combat info: HP (current/max), proficiency bonus
- Skills, saving throws, proficiencies
- Prepared spells and spell slots (if caster)
- Equipment and inventory with quantities
- Character background and features"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "character_id": {
                    "type": "integer",
                    "description": "Numeric ID of the character"
                }
            },
            "required": ["character_id"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let character_id = extract_i32_param(&arguments, "character_id")?;

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let (character, char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Failed to retrieve character: {}", e))?;

        // Format character data for LLM consumption
        let result = json!({
            "id": character.id,
            "character_name": char_data.character_name,
            "player_id": character.player_id,
            "campaign_id": character.campaign_id,
            "level": char_data.level,
            "experience_points": char_data.experience_points,
            "race": char_data.race,
            "subrace": char_data.subrace,
            "class": char_data.primary_class_name(),
            "classes": char_data.classes.iter().map(|c| json!({
                "class_name": c.class_name,
                "level": c.level,
                "subclass": c.subclass,
                "hit_dice_type": c.hit_dice_type,
                "hit_dice_remaining": c.hit_dice_remaining
            })).collect::<Vec<_>>(),
            "background": char_data.background,
            "alignment": char_data.alignment,
            "abilities": format_abilities_json(&char_data),
            "hit_points": {
                "current": char_data.current_hp,
                "maximum": char_data.max_hp,
                "total_hit_dice_remaining": char_data.total_hit_dice_remaining()
            },
            "proficiency_bonus": char_data.proficiency_bonus(),
            "proficiencies": {
                "skills": char_data.proficiencies.skills,
                "saves": char_data.proficiencies.saves,
                "armor": char_data.proficiencies.armor,
                "weapons": char_data.proficiencies.weapons,
                "tools": char_data.proficiencies.tools,
                "languages": char_data.proficiencies.languages
            },
            "class_features": char_data.class_features,
            "feats": char_data.feats,
            "spells": {
                "known_spells": char_data.spells.known_spells,
                "prepared_spells": char_data.spells.prepared_spells,
                "cantrips": char_data.spells.cantrips,
                "spell_slots": char_data.spells.spell_slots
            },
            "inventory": char_data.inventory,
            "currency": char_data.currency,
            "equipped": {
                "armor": char_data.equipped.armor,
                "shield": char_data.equipped.shield,
                "main_hand": char_data.equipped.main_hand,
                "off_hand": char_data.equipped.off_hand
            },
            "personality": {
                "traits": char_data.personality.traits,
                "ideals": char_data.personality.ideals,
                "bonds": char_data.personality.bonds,
                "flaws": char_data.personality.flaws
            }
        });

        debug!(
            "Retrieved character: {} (ID: {})",
            char_data.character_name, character.id
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for listing all characters in a campaign
pub struct ListCampaignCharactersTool {
    db_service: Arc<DatabaseService>,
}

impl ListCampaignCharactersTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for ListCampaignCharactersTool {
    fn name(&self) -> &str {
        "list_campaign_characters"
    }

    fn description(&self) -> &str {
        "List all characters in the specified campaign with summary information.

Usage:
- Provide campaign_id to list characters
- Returns array of character summaries
- Each entry includes: id, name, level, class, race, HP

When to use:
- To discover what characters exist in a campaign
- Before using get_character to get character IDs
- For party composition overview
- When answering questions about the party

Output format:
- Array of character objects with summary data
- Sorted by character name
- Includes current/max HP for quick status check"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "campaign_id": {
                    "type": "integer",
                    "description": "ID of the campaign to list characters from"
                }
            },
            "required": ["campaign_id"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let campaign_id = extract_i32_param(&arguments, "campaign_id")?;

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let characters = char_service
            .list_characters_for_campaign(campaign_id)
            .map_err(|e| format!("Failed to list characters: {}", e))?;

        if characters.is_empty() {
            return Ok(json!({
                "campaign_id": campaign_id,
                "character_count": 0,
                "characters": [],
                "message": "No characters found in this campaign"
            })
            .to_string());
        }

        // Get full data for each character for summary
        let mut character_summaries = Vec::new();
        for character in characters {
            match char_service.get_character(character.id) {
                Ok((_, char_data)) => {
                    character_summaries.push(format_character_summary(
                        character.id,
                        &char_data,
                        character.player_id,
                    ));
                }
                Err(e) => {
                    error!(
                        "Failed to get character data for ID {}: {}",
                        character.id, e
                    );
                }
            }
        }

        let result = json!({
            "campaign_id": campaign_id,
            "character_count": character_summaries.len(),
            "characters": character_summaries
        });

        debug!(
            "Listed {} characters for campaign {}",
            character_summaries.len(),
            campaign_id
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for listing NPCs in a campaign
pub struct ListNpcsTool {
    db_service: Arc<DatabaseService>,
}

impl ListNpcsTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for ListNpcsTool {
    fn name(&self) -> &str {
        "list_npcs"
    }

    fn description(&self) -> &str {
        "List all NPCs (non-player characters) in the specified campaign.

Usage:
- Provide campaign_id to list NPCs
- Returns array of NPC summaries
- Each entry includes: id, name, level, class, race

When to use:
- To see what NPCs exist in a campaign
- When planning encounters or scenes
- To get NPC IDs for further queries
- When the DM needs to reference NPCs

Output format:
- Array of NPC objects with summary data
- Only includes characters marked as NPCs
- Sorted by character name"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "campaign_id": {
                    "type": "integer",
                    "description": "ID of the campaign to list NPCs from"
                }
            },
            "required": ["campaign_id"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let campaign_id = extract_i32_param(&arguments, "campaign_id")?;

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let npcs = char_service
            .list_npcs_for_campaign(campaign_id)
            .map_err(|e| format!("Failed to list NPCs: {}", e))?;

        if npcs.is_empty() {
            return Ok(json!({
                "campaign_id": campaign_id,
                "npc_count": 0,
                "npcs": [],
                "message": "No NPCs found in this campaign"
            })
            .to_string());
        }

        // Get full data for each NPC for summary
        let mut npc_summaries = Vec::new();
        for npc in npcs {
            match char_service.get_character(npc.id) {
                Ok((_, char_data)) => {
                    npc_summaries.push(format_character_summary(npc.id, &char_data, None));
                }
                Err(e) => {
                    error!("Failed to get NPC data for ID {}: {}", npc.id, e);
                }
            }
        }

        let result = json!({
            "campaign_id": campaign_id,
            "npc_count": npc_summaries.len(),
            "npcs": npc_summaries
        });

        debug!(
            "Listed {} NPCs for campaign {}",
            npc_summaries.len(),
            campaign_id
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for listing player characters (non-NPCs) in a campaign
pub struct ListPcsTool {
    db_service: Arc<DatabaseService>,
}

impl ListPcsTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for ListPcsTool {
    fn name(&self) -> &str {
        "list_player_characters"
    }

    fn description(&self) -> &str {
        "List all player characters (PCs, not NPCs) in the specified campaign.

Usage:
- Provide campaign_id to list player characters
- Returns array of PC summaries
- Each entry includes: id, name, level, class, race, HP, player_id

When to use:
- To see the party composition
- When planning encounters balanced for the party
- To get PC IDs for further queries
- When you need to reference specific player characters

Output format:
- Array of PC objects with summary data
- Only includes characters NOT marked as NPCs
- Sorted by character name"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "campaign_id": {
                    "type": "integer",
                    "description": "ID of the campaign to list player characters from"
                }
            },
            "required": ["campaign_id"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let campaign_id = extract_i32_param(&arguments, "campaign_id")?;

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let pcs = char_service
            .list_pcs_for_campaign(campaign_id)
            .map_err(|e| format!("Failed to list player characters: {}", e))?;

        if pcs.is_empty() {
            return Ok(json!({
                "campaign_id": campaign_id,
                "pc_count": 0,
                "player_characters": [],
                "message": "No player characters found in this campaign"
            })
            .to_string());
        }

        // Get full data for each PC for summary
        let mut pc_summaries = Vec::new();
        for pc in pcs {
            match char_service.get_character(pc.id) {
                Ok((_, char_data)) => {
                    pc_summaries.push(format_character_summary(pc.id, &char_data, pc.player_id));
                }
                Err(e) => {
                    error!("Failed to get PC data for ID {}: {}", pc.id, e);
                }
            }
        }

        let result = json!({
            "campaign_id": campaign_id,
            "pc_count": pc_summaries.len(),
            "player_characters": pc_summaries
        });

        debug!(
            "Listed {} player characters for campaign {}",
            pc_summaries.len(),
            campaign_id
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for quick stat lookups without full character data
pub struct GetCharacterStatsTool {
    db_service: Arc<DatabaseService>,
}

impl GetCharacterStatsTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for GetCharacterStatsTool {
    fn name(&self) -> &str {
        "get_character_stats"
    }

    fn description(&self) -> &str {
        "Get quick stat summary for a character (HP, abilities, skills, saves).

Usage:
- Provide character_id
- Returns focused stat block
- Faster than get_character for simple stat queries

When to use:
- Answering combat-related questions (HP, initiative)
- Quick ability score or modifier lookups
- Skill check bonuses
- Saving throw calculations

Output includes:
- Ability scores with modifiers
- HP (current/max), initiative
- Proficiency bonus
- Skill proficiencies
- Saving throw proficiencies"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "character_id": {
                    "type": "integer",
                    "description": "Numeric ID of the character"
                }
            },
            "required": ["character_id"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let character_id = extract_i32_param(&arguments, "character_id")?;

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let (character, char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Failed to retrieve character: {}", e))?;

        let result = json!({
            "character_id": character.id,
            "character_name": char_data.character_name,
            "level": char_data.level,
            "abilities": format_abilities_json(&char_data),
            "combat": {
                "hit_points": {
                    "current": char_data.current_hp,
                    "maximum": char_data.max_hp
                },
                "initiative_bonus": char_data.abilities.dex_modifier(),
                "proficiency_bonus": char_data.proficiency_bonus()
            },
            "proficiencies": {
                "skills": char_data.proficiencies.skills,
                "saves": char_data.proficiencies.saves
            }
        });

        debug!(
            "Retrieved stats for character: {} (ID: {})",
            char_data.character_name, character.id
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for checking available spell slots for spellcasting characters
pub struct CheckSpellSlotsTool {
    db_service: Arc<DatabaseService>,
}

impl CheckSpellSlotsTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for CheckSpellSlotsTool {
    fn name(&self) -> &str {
        "check_spell_slots"
    }

    fn description(&self) -> &str {
        "Check available spell slots for a spellcasting character.

Usage:
- Provide character_id
- Returns spell slot availability by level
- Shows current/maximum slots for each level
- Returns empty if character is not a spellcaster

When to use:
- Before/during spell casting
- Answering questions about spellcasting resources
- Planning spell usage for encounters
- Determining if character can cast a specific spell

Output format:
- Object with spell levels as keys (1-9)
- Each level shows current and maximum slots
- Only includes levels the character has access to"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "character_id": {
                    "type": "integer",
                    "description": "Numeric ID of the character"
                }
            },
            "required": ["character_id"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let character_id = extract_i32_param(&arguments, "character_id")?;

        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let mut char_service = CharacterService::new(&mut conn);
        let (character, char_data) = char_service
            .get_character(character_id)
            .map_err(|e| format!("Failed to retrieve character: {}", e))?;

        let result = json!({
            "character_id": character.id,
            "character_name": char_data.character_name,
            "class": char_data.primary_class_name(),
            "level": char_data.level,
            "spell_slots": char_data.spells.spell_slots,
            "prepared_spells_count": char_data.spells.prepared_spells.len(),
            "prepared_spells": char_data.spells.prepared_spells,
            "cantrips": char_data.spells.cantrips,
            "is_spellcaster": !char_data.spells.spell_slots.is_empty()
        });

        debug!(
            "Checked spell slots for character: {} (ID: {})",
            char_data.character_name, character.id
        );
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

/// Tool for listing all players
pub struct ListPlayersTool {
    db_service: Arc<DatabaseService>,
}

impl ListPlayersTool {
    pub fn new(db_service: Arc<DatabaseService>) -> Self {
        Self { db_service }
    }
}

#[async_trait]
impl ToolTrait for ListPlayersTool {
    fn name(&self) -> &str {
        "list_players"
    }

    fn description(&self) -> &str {
        "List all players in the system with their IDs and names.

When to use:
- Before creating a character to find the correct player_id
- To see which players exist in the system
- To look up a player by name

Output:
- Array of player objects with id, name, and email"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn execute(&self, _arguments: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut conn = self
            .db_service
            .get_connection()
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let mut player_service = PlayerService::new(&mut conn);
        let players = player_service
            .list_players()
            .map_err(|e| format!("Failed to list players: {}", e))?;

        let result: Vec<Value> = players
            .iter()
            .map(|p| {
                json!({
                    "id": p.id,
                    "name": p.name,
                    "email": p.email
                })
            })
            .collect();

        debug!("Listed {} players", result.len());
        Ok(serde_json::to_string_pretty(&result)?)
    }
}
