//! Character and NPC management tools for MCP
//!
//! Provides tools for listing, creating, and managing characters and NPCs
//! within the active campaign.

use crate::context::McpContext;
use crate::error::McpError;
use mimir_dm_core::models::character::data::{
    AbilityScores, CharacterData, ClassLevel, Currency, EquippedItems, Personality, Proficiencies,
    SpellData,
};
use mimir_dm_core::services::character::CharacterService;
use mimir_dm_core::services::ModuleNpcService;
use rust_mcp_sdk::schema::{Tool, ToolInputSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Create a simple properties map for tool input schema
fn create_properties(
    props: Vec<(&str, &str, &str)>,
) -> Option<HashMap<String, serde_json::Map<String, serde_json::Value>>> {
    let mut map = HashMap::new();
    for (name, prop_type, description) in props {
        let mut inner = serde_json::Map::new();
        inner.insert(
            "type".to_string(),
            serde_json::Value::String(prop_type.to_string()),
        );
        inner.insert(
            "description".to_string(),
            serde_json::Value::String(description.to_string()),
        );
        map.insert(name.to_string(), inner);
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

/// Character list item for responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterListItem {
    pub id: i32,
    pub name: String,
    pub is_npc: bool,
    pub level: i32,
    pub class: Option<String>,
    pub race: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for list_characters tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListCharactersInput {
    /// Filter by character type: "pc", "npc", or "all" (default: "all")
    #[serde(default)]
    pub character_type: Option<String>,
}

impl ListCharactersInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "list_characters".to_string(),
            description: Some(
                "List characters in the active campaign. Filter by type: 'pc' for player characters, 'npc' for NPCs, or 'all' (default)."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec![],
                create_properties(vec![(
                    "character_type",
                    "string",
                    "Filter by type: 'pc', 'npc', or 'all' (default: 'all')",
                )]),
                None,
            ),
            title: None,
            annotations: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            meta: None,
        }
    }

    /// Execute the list_characters tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<Vec<CharacterListItem>, McpError> {
        let campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;
        let mut service = CharacterService::new(&mut conn);

        let characters = match self.character_type.as_deref() {
            Some("pc") => service
                .list_pcs_for_campaign(campaign.id)
                .map_err(|e| McpError::Service(e.to_string()))?,
            Some("npc") => service
                .list_npcs_for_campaign(campaign.id)
                .map_err(|e| McpError::Service(e.to_string()))?,
            _ => service
                .list_characters_for_campaign(campaign.id)
                .map_err(|e| McpError::Service(e.to_string()))?,
        };

        let items: Vec<CharacterListItem> = characters
            .into_iter()
            .map(|c| CharacterListItem {
                id: c.id,
                name: c.character_name,
                is_npc: c.is_npc,
                level: c.current_level,
                class: c.class,
                race: c.race,
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect();

        Ok(items)
    }
}

/// Input for get_character tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCharacterInput {
    /// The character ID to retrieve
    pub character_id: i32,

    /// Include version history (default: false)
    #[serde(default)]
    pub include_versions: bool,
}

impl GetCharacterInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "get_character".to_string(),
            description: Some(
                "Get detailed information about a character, including their full data and optionally version history."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["character_id".to_string()],
                create_properties(vec![
                    ("character_id", "integer", "The character ID to retrieve"),
                    (
                        "include_versions",
                        "boolean",
                        "Include version history (default: false)",
                    ),
                ]),
                None,
            ),
            title: None,
            annotations: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            meta: None,
        }
    }

    /// Execute the get_character tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<GetCharacterResponse, McpError> {
        let _campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;
        let mut service = CharacterService::new(&mut conn);

        let (character, data) = service
            .get_character(self.character_id)
            .map_err(|e| McpError::Service(e.to_string()))?;

        let versions = if self.include_versions {
            let version_list = service
                .get_character_versions(self.character_id)
                .map_err(|e| McpError::Service(e.to_string()))?;

            Some(
                version_list
                    .into_iter()
                    .map(|v| CharacterVersionSummary {
                        version_number: v.version_number,
                        level: v.level,
                        snapshot_reason: v.snapshot_reason,
                        created_at: v.created_at,
                    })
                    .collect(),
            )
        } else {
            None
        };

        // Calculate derived values before building response
        let armor_class = calculate_ac(&data);
        let classes_str = data
            .classes
            .iter()
            .map(|c| format!("{} {}", c.class_name, c.level))
            .collect::<Vec<_>>()
            .join(" / ");

        Ok(GetCharacterResponse {
            id: character.id,
            name: character.character_name,
            is_npc: character.is_npc,
            level: data.level,
            race: data.race,
            classes: classes_str,
            background: data.background,
            alignment: data.alignment,
            abilities: AbilitySummary {
                strength: data.abilities.strength,
                dexterity: data.abilities.dexterity,
                constitution: data.abilities.constitution,
                intelligence: data.abilities.intelligence,
                wisdom: data.abilities.wisdom,
                charisma: data.abilities.charisma,
            },
            max_hp: data.max_hp,
            current_hp: data.current_hp,
            armor_class,
            speed: data.speed,
            proficiencies: ProficiencySummary {
                skills: data.proficiencies.skills,
                saves: data.proficiencies.saves,
                languages: data.proficiencies.languages,
            },
            feats: data.feats,
            npc_role: data.npc_role,
            npc_location: data.npc_location,
            npc_faction: data.npc_faction,
            npc_notes: data.npc_notes,
            backstory: data.backstory,
            personality: PersonalitySummary {
                traits: data.personality.traits,
                ideals: data.personality.ideals,
                bonds: data.personality.bonds,
                flaws: data.personality.flaws,
            },
            versions,
        })
    }
}

/// Simple AC calculation
fn calculate_ac(data: &CharacterData) -> i32 {
    // Base AC = 10 + DEX mod
    let dex_mod = (data.abilities.dexterity - 10) / 2;
    10 + dex_mod
}

/// Response from get_character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCharacterResponse {
    pub id: i32,
    pub name: String,
    pub is_npc: bool,
    pub level: i32,
    pub race: String,
    pub classes: String,
    pub background: String,
    pub alignment: Option<String>,
    pub abilities: AbilitySummary,
    pub max_hp: i32,
    pub current_hp: i32,
    pub armor_class: i32,
    pub speed: i32,
    pub proficiencies: ProficiencySummary,
    pub feats: Vec<String>,
    pub npc_role: Option<String>,
    pub npc_location: Option<String>,
    pub npc_faction: Option<String>,
    pub npc_notes: Option<String>,
    pub backstory: Option<String>,
    pub personality: PersonalitySummary,
    pub versions: Option<Vec<CharacterVersionSummary>>,
}

/// Ability score summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilitySummary {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

/// Proficiency summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProficiencySummary {
    pub skills: Vec<String>,
    pub saves: Vec<String>,
    pub languages: Vec<String>,
}

/// Personality summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalitySummary {
    pub traits: Option<String>,
    pub ideals: Option<String>,
    pub bonds: Option<String>,
    pub flaws: Option<String>,
}

/// Character version summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterVersionSummary {
    pub version_number: i32,
    pub level: i32,
    pub snapshot_reason: Option<String>,
    pub created_at: String,
}

/// Input for create_character tool
/// Creates both NPCs (default) and player characters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCharacterInput {
    /// Character name (required)
    pub name: String,

    /// Race (e.g., "Human", "Elf", "Dwarf")
    pub race: String,

    /// Class (e.g., "Fighter", "Wizard", "Commoner")
    #[serde(default)]
    pub class: Option<String>,

    /// Whether this is an NPC (default: true for backwards compatibility)
    #[serde(default = "default_true")]
    pub is_npc: bool,

    /// Player ID (optional, for player characters)
    #[serde(default)]
    pub player_id: Option<i32>,

    /// Player name (optional, for display purposes)
    #[serde(default)]
    pub player_name: Option<String>,

    /// Character level (default: 1)
    #[serde(default = "default_one")]
    pub level: i32,

    /// Background (e.g., "Soldier", "Noble", "Acolyte")
    #[serde(default)]
    pub background: Option<String>,

    /// NPC's role in the story (e.g., "quest_giver", "merchant", "antagonist")
    #[serde(default)]
    pub role: Option<String>,

    /// Location where the character can be found
    #[serde(default)]
    pub location: Option<String>,

    /// Faction or organization affiliation
    #[serde(default)]
    pub faction: Option<String>,

    /// Notes about the character
    #[serde(default)]
    pub notes: Option<String>,

    /// Alignment (e.g., "Lawful Good", "Chaotic Neutral")
    #[serde(default)]
    pub alignment: Option<String>,

    /// Backstory
    #[serde(default)]
    pub backstory: Option<String>,
}

fn default_true() -> bool {
    true
}

impl CreateCharacterInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "create_character".to_string(),
            description: Some(
                "Create a new character in the active campaign. Can create both NPCs (is_npc=true, default) and player characters (is_npc=false). For PCs, optionally provide player_id and player_name."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["name".to_string(), "race".to_string()],
                create_properties(vec![
                    ("name", "string", "Character name (required)"),
                    ("race", "string", "Race (e.g., Human, Elf, Dwarf)"),
                    (
                        "class",
                        "string",
                        "Class (e.g., Fighter, Wizard, Commoner)",
                    ),
                    (
                        "is_npc",
                        "boolean",
                        "Whether this is an NPC (default: true). Set to false for player characters.",
                    ),
                    (
                        "player_id",
                        "integer",
                        "Player ID for player characters (optional)",
                    ),
                    (
                        "player_name",
                        "string",
                        "Player name for display (optional)",
                    ),
                    (
                        "level",
                        "integer",
                        "Character level (default: 1)",
                    ),
                    (
                        "background",
                        "string",
                        "Background (e.g., Soldier, Noble, Acolyte)",
                    ),
                    (
                        "role",
                        "string",
                        "Role in the story for NPCs (e.g., quest_giver, merchant, antagonist)",
                    ),
                    ("location", "string", "Location where the character can be found"),
                    ("faction", "string", "Faction or organization affiliation"),
                    ("notes", "string", "Notes about the character"),
                    (
                        "alignment",
                        "string",
                        "Alignment (e.g., Lawful Good, Chaotic Neutral)",
                    ),
                    ("backstory", "string", "Character backstory"),
                ]),
                None,
            ),
            title: None,
            annotations: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            meta: None,
        }
    }

    /// Execute the create_character tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<CreateCharacterResponse, McpError> {
        let campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;
        let mut service = CharacterService::new(&mut conn);

        // Build CharacterData
        let class_name = self.class.clone().unwrap_or_else(|| {
            if self.is_npc {
                "Commoner".to_string()
            } else {
                "Fighter".to_string()
            }
        });
        let background = self.background.clone().unwrap_or_else(|| {
            if self.is_npc {
                "NPC".to_string()
            } else {
                "Adventurer".to_string()
            }
        });
        let snapshot_reason = if self.is_npc {
            "NPC created via MCP".to_string()
        } else {
            "Player character created via MCP".to_string()
        };

        let character_data = CharacterData {
            character_name: self.name.clone(),
            player_id: self.player_id,
            level: self.level,
            experience_points: 0,
            version: 1,
            snapshot_reason: Some(snapshot_reason),
            created_at: chrono::Utc::now().to_rfc3339(),
            race: self.race.clone(),
            subrace: None,
            classes: vec![ClassLevel {
                class_name: class_name.clone(),
                level: self.level,
                subclass: None,
                hit_dice_type: "d8".to_string(),
                hit_dice_remaining: self.level,
            }],
            background,
            alignment: self.alignment.clone(),
            abilities: AbilityScores {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            max_hp: 8 + (self.level - 1) * 5, // Base HP + average per level
            current_hp: 8 + (self.level - 1) * 5,
            proficiencies: Proficiencies {
                skills: vec![],
                saves: vec![],
                armor: vec![],
                weapons: vec![],
                tools: vec![],
                languages: vec!["Common".to_string()],
            },
            class_features: vec![],
            feats: vec![],
            spells: SpellData::default(),
            inventory: vec![],
            currency: Currency::default(),
            speed: 30,
            equipped: EquippedItems::default(),
            personality: Personality::default(),
            player_name: self.player_name.clone(),
            appearance: Default::default(),
            backstory: self.backstory.clone(),
            background_feature: None,
            roleplay_notes: Default::default(),
            npc_role: if self.is_npc { self.role.clone() } else { None },
            npc_location: if self.is_npc { self.location.clone() } else { None },
            npc_faction: if self.is_npc { self.faction.clone() } else { None },
            npc_notes: if self.is_npc { self.notes.clone() } else { None },
            legendary_actions: Vec::new(),
            legendary_action_count: None,
        };

        let character = service
            .create_character(
                Some(campaign.id),
                self.player_id,
                self.is_npc,
                &campaign.directory_path,
                character_data,
            )
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(CreateCharacterResponse {
            success: true,
            character_id: character.id,
            name: character.character_name,
            race: self.race.clone(),
            class: class_name,
            level: self.level,
            is_npc: self.is_npc,
            player_id: self.player_id,
            role: self.role.clone(),
            location: self.location.clone(),
        })
    }
}

/// Response from create_character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCharacterResponse {
    pub success: bool,
    pub character_id: i32,
    pub name: String,
    pub race: String,
    pub class: String,
    pub level: i32,
    pub is_npc: bool,
    pub player_id: Option<i32>,
    pub role: Option<String>,
    pub location: Option<String>,
}

/// Input for assign_npc_to_module tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignNpcToModuleInput {
    /// The NPC's character ID
    pub character_id: i32,

    /// The module ID to assign the NPC to
    pub module_id: i32,

    /// Role in this module (e.g., "quest_giver", "antagonist", "ally", "merchant")
    #[serde(default)]
    pub role: Option<String>,

    /// Encounter or scene tag for grouping (e.g., "tavern_scene", "boss_fight")
    #[serde(default)]
    pub encounter_tag: Option<String>,

    /// Notes specific to this module assignment
    #[serde(default)]
    pub notes: Option<String>,
}

impl AssignNpcToModuleInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "assign_npc_to_module".to_string(),
            description: Some(
                "Assign an NPC to a module with a specific role and optional encounter tag. NPCs can appear in multiple modules with different roles."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["character_id".to_string(), "module_id".to_string()],
                create_properties(vec![
                    ("character_id", "integer", "The NPC's character ID"),
                    ("module_id", "integer", "The module ID to assign the NPC to"),
                    (
                        "role",
                        "string",
                        "Role in this module (e.g., quest_giver, antagonist, ally, merchant)",
                    ),
                    (
                        "encounter_tag",
                        "string",
                        "Encounter or scene tag for grouping (e.g., tavern_scene, boss_fight)",
                    ),
                    ("notes", "string", "Notes specific to this module assignment"),
                ]),
                None,
            ),
            title: None,
            annotations: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            meta: None,
        }
    }

    /// Execute the assign_npc_to_module tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<AssignNpcResponse, McpError> {
        let _campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;
        let mut service = ModuleNpcService::new(&mut conn);

        let module_npc = service
            .add_npc(
                self.module_id,
                self.character_id,
                self.role.clone(),
                self.encounter_tag.clone(),
                self.notes.clone(),
            )
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(AssignNpcResponse {
            success: true,
            module_npc_id: module_npc.id,
            character_id: module_npc.character_id,
            module_id: module_npc.module_id,
            role: module_npc.role,
            encounter_tag: module_npc.encounter_tag,
        })
    }
}

/// Response from assign_npc_to_module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignNpcResponse {
    pub success: bool,
    pub module_npc_id: i32,
    pub character_id: i32,
    pub module_id: i32,
    pub role: Option<String>,
    pub encounter_tag: Option<String>,
}

/// Input for add_item_to_character tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddItemToCharacterInput {
    /// The character ID to add the item to
    pub character_id: i32,

    /// Item name (must exist in catalog)
    pub item_name: String,

    /// Item source book (e.g., "PHB", "DMG")
    pub item_source: String,

    /// Quantity of this item (default: 1)
    #[serde(default = "default_one")]
    pub quantity: i32,

    /// Notes about the item
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_one() -> i32 {
    1
}

impl AddItemToCharacterInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "add_item_to_character".to_string(),
            description: Some(
                "Add an item from the catalog to a character's inventory. Creates a new version of the character."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec![
                    "character_id".to_string(),
                    "item_name".to_string(),
                    "item_source".to_string(),
                ],
                create_properties(vec![
                    ("character_id", "integer", "The character ID to add the item to"),
                    ("item_name", "string", "Item name (must exist in catalog)"),
                    ("item_source", "string", "Item source book (e.g., PHB, DMG)"),
                    ("quantity", "integer", "Quantity of this item (default: 1)"),
                    ("notes", "string", "Notes about the item"),
                ]),
                None,
            ),
            title: None,
            annotations: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            meta: None,
        }
    }

    /// Execute the add_item_to_character tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<AddCharacterItemResponse, McpError> {
        let _campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;
        let mut service = CharacterService::new(&mut conn);

        let version = service
            .add_item(
                self.character_id,
                &self.item_name,
                &self.item_source,
                self.quantity,
                self.notes.clone(),
            )
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(AddCharacterItemResponse {
            success: true,
            character_id: self.character_id,
            item_name: self.item_name.clone(),
            quantity: self.quantity,
            new_version: version.version_number,
        })
    }
}

/// Response from add_item_to_character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCharacterItemResponse {
    pub success: bool,
    pub character_id: i32,
    pub item_name: String,
    pub quantity: i32,
    pub new_version: i32,
}

/// Input for update_character_currency tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCharacterCurrencyInput {
    /// The character ID to update
    pub character_id: i32,

    /// Copper pieces to add (use negative to subtract)
    #[serde(default)]
    pub copper: i32,

    /// Silver pieces to add (use negative to subtract)
    #[serde(default)]
    pub silver: i32,

    /// Electrum pieces to add (use negative to subtract)
    #[serde(default)]
    pub electrum: i32,

    /// Gold pieces to add (use negative to subtract)
    #[serde(default)]
    pub gold: i32,

    /// Platinum pieces to add (use negative to subtract)
    #[serde(default)]
    pub platinum: i32,
}

impl UpdateCharacterCurrencyInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "update_character_currency".to_string(),
            description: Some(
                "Update a character's currency. Use positive values to add, negative to subtract. Creates a new version."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["character_id".to_string()],
                create_properties(vec![
                    ("character_id", "integer", "The character ID to update"),
                    ("copper", "integer", "Copper pieces to add (negative to subtract)"),
                    ("silver", "integer", "Silver pieces to add (negative to subtract)"),
                    ("electrum", "integer", "Electrum pieces to add (negative to subtract)"),
                    ("gold", "integer", "Gold pieces to add (negative to subtract)"),
                    ("platinum", "integer", "Platinum pieces to add (negative to subtract)"),
                ]),
                None,
            ),
            title: None,
            annotations: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            meta: None,
        }
    }

    /// Execute the update_character_currency tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<UpdateCurrencyResponse, McpError> {
        let _campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;
        let mut service = CharacterService::new(&mut conn);

        let version = service
            .update_currency(
                self.character_id,
                self.copper,
                self.silver,
                self.electrum,
                self.gold,
                self.platinum,
            )
            .map_err(|e| McpError::Service(e.to_string()))?;

        // Get updated character to return current totals
        let (_char, data) = service
            .get_character(self.character_id)
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(UpdateCurrencyResponse {
            success: true,
            character_id: self.character_id,
            new_version: version.version_number,
            current_currency: CurrencySummary {
                copper: data.currency.copper,
                silver: data.currency.silver,
                electrum: data.currency.electrum,
                gold: data.currency.gold,
                platinum: data.currency.platinum,
            },
        })
    }
}

/// Response from update_character_currency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCurrencyResponse {
    pub success: bool,
    pub character_id: i32,
    pub new_version: i32,
    pub current_currency: CurrencySummary,
}

/// Currency summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencySummary {
    pub copper: i32,
    pub silver: i32,
    pub electrum: i32,
    pub gold: i32,
    pub platinum: i32,
}

/// Input for edit_character tool
/// Updates character attributes and creates a new version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditCharacterInput {
    /// The character ID to edit (required)
    pub character_id: i32,

    /// New character name
    #[serde(default)]
    pub name: Option<String>,

    /// New race
    #[serde(default)]
    pub race: Option<String>,

    /// New alignment
    #[serde(default)]
    pub alignment: Option<String>,

    /// New backstory
    #[serde(default)]
    pub backstory: Option<String>,

    /// New max HP
    #[serde(default)]
    pub max_hp: Option<i32>,

    /// New current HP
    #[serde(default)]
    pub current_hp: Option<i32>,

    /// New speed
    #[serde(default)]
    pub speed: Option<i32>,

    /// Ability scores (all six must be provided together if updating)
    #[serde(default)]
    pub abilities: Option<AbilityScoresInput>,

    /// Personality traits
    #[serde(default)]
    pub personality_traits: Option<String>,

    /// Ideals
    #[serde(default)]
    pub ideals: Option<String>,

    /// Bonds
    #[serde(default)]
    pub bonds: Option<String>,

    /// Flaws
    #[serde(default)]
    pub flaws: Option<String>,

    /// NPC role (only for NPCs)
    #[serde(default)]
    pub npc_role: Option<String>,

    /// NPC location (only for NPCs)
    #[serde(default)]
    pub npc_location: Option<String>,

    /// NPC faction (only for NPCs)
    #[serde(default)]
    pub npc_faction: Option<String>,

    /// NPC notes (only for NPCs)
    #[serde(default)]
    pub npc_notes: Option<String>,

    /// Snapshot reason for version history
    #[serde(default)]
    pub snapshot_reason: Option<String>,
}

/// Ability scores input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityScoresInput {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

impl EditCharacterInput {
    /// Get the tool definition
    pub fn tool() -> Tool {
        Tool {
            name: "edit_character".to_string(),
            description: Some(
                "Edit a character's attributes. All parameters except character_id are optional - only provide fields you want to change. Creates a new version of the character, preserving history."
                    .to_string(),
            ),
            input_schema: ToolInputSchema::new(
                vec!["character_id".to_string()],
                create_properties(vec![
                    ("character_id", "integer", "The character ID to edit (required)"),
                    ("name", "string", "New character name"),
                    ("race", "string", "New race"),
                    ("alignment", "string", "New alignment (e.g., Lawful Good)"),
                    ("backstory", "string", "New backstory"),
                    ("max_hp", "integer", "New maximum HP"),
                    ("current_hp", "integer", "New current HP"),
                    ("speed", "integer", "New speed in feet"),
                    ("abilities", "object", "Ability scores object with strength, dexterity, constitution, intelligence, wisdom, charisma"),
                    ("personality_traits", "string", "Personality traits"),
                    ("ideals", "string", "Character ideals"),
                    ("bonds", "string", "Character bonds"),
                    ("flaws", "string", "Character flaws"),
                    ("npc_role", "string", "NPC role in story (NPCs only)"),
                    ("npc_location", "string", "NPC location (NPCs only)"),
                    ("npc_faction", "string", "NPC faction (NPCs only)"),
                    ("npc_notes", "string", "NPC notes (NPCs only)"),
                    ("snapshot_reason", "string", "Reason for this edit (for version history)"),
                ]),
                None,
            ),
            title: None,
            annotations: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            meta: None,
        }
    }

    /// Execute the edit_character tool
    pub async fn execute(
        &self,
        context: Arc<McpContext>,
    ) -> Result<EditCharacterResponse, McpError> {
        let _campaign = context.require_active_campaign().await?;
        let mut conn = context.get_connection()?;
        let mut service = CharacterService::new(&mut conn);

        // Get existing character data
        let (character, mut data) = service
            .get_character(self.character_id)
            .map_err(|e| McpError::Service(e.to_string()))?;

        // Track what changed for the response
        let mut changes = Vec::new();

        // Apply updates
        if let Some(ref name) = self.name {
            data.character_name = name.clone();
            changes.push("name".to_string());
        }
        if let Some(ref race) = self.race {
            data.race = race.clone();
            changes.push("race".to_string());
        }
        if let Some(ref alignment) = self.alignment {
            data.alignment = Some(alignment.clone());
            changes.push("alignment".to_string());
        }
        if let Some(ref backstory) = self.backstory {
            data.backstory = Some(backstory.clone());
            changes.push("backstory".to_string());
        }
        if let Some(max_hp) = self.max_hp {
            data.max_hp = max_hp;
            changes.push("max_hp".to_string());
        }
        if let Some(current_hp) = self.current_hp {
            data.current_hp = current_hp;
            changes.push("current_hp".to_string());
        }
        if let Some(speed) = self.speed {
            data.speed = speed;
            changes.push("speed".to_string());
        }
        if let Some(ref abilities) = self.abilities {
            data.abilities = AbilityScores {
                strength: abilities.strength,
                dexterity: abilities.dexterity,
                constitution: abilities.constitution,
                intelligence: abilities.intelligence,
                wisdom: abilities.wisdom,
                charisma: abilities.charisma,
            };
            changes.push("abilities".to_string());
        }
        if let Some(ref traits) = self.personality_traits {
            data.personality.traits = Some(traits.clone());
            changes.push("personality_traits".to_string());
        }
        if let Some(ref ideals) = self.ideals {
            data.personality.ideals = Some(ideals.clone());
            changes.push("ideals".to_string());
        }
        if let Some(ref bonds) = self.bonds {
            data.personality.bonds = Some(bonds.clone());
            changes.push("bonds".to_string());
        }
        if let Some(ref flaws) = self.flaws {
            data.personality.flaws = Some(flaws.clone());
            changes.push("flaws".to_string());
        }

        // NPC-specific fields (only update if character is NPC)
        if character.is_npc {
            if let Some(ref role) = self.npc_role {
                data.npc_role = Some(role.clone());
                changes.push("npc_role".to_string());
            }
            if let Some(ref location) = self.npc_location {
                data.npc_location = Some(location.clone());
                changes.push("npc_location".to_string());
            }
            if let Some(ref faction) = self.npc_faction {
                data.npc_faction = Some(faction.clone());
                changes.push("npc_faction".to_string());
            }
            if let Some(ref notes) = self.npc_notes {
                data.npc_notes = Some(notes.clone());
                changes.push("npc_notes".to_string());
            }
        }

        if changes.is_empty() {
            return Err(McpError::InvalidParameter(
                "No fields provided to update".to_string(),
            ));
        }

        // Build snapshot reason
        let snapshot_reason = self.snapshot_reason.clone().unwrap_or_else(|| {
            format!("Updated via MCP: {}", changes.join(", "))
        });

        // Create new version with updates
        let version = service
            .update_character(self.character_id, data, Some(snapshot_reason))
            .map_err(|e| McpError::Service(e.to_string()))?;

        Ok(EditCharacterResponse {
            success: true,
            character_id: self.character_id,
            new_version: version.version_number,
            fields_updated: changes,
        })
    }
}

/// Response from edit_character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditCharacterResponse {
    pub success: bool,
    pub character_id: i32,
    pub new_version: i32,
    pub fields_updated: Vec<String>,
}
