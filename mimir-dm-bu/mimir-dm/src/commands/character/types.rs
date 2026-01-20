//! Character command request/response types.
//!
//! Frontend-friendly types for JSON serialization in character management commands.

use mimir_dm_core::models::character::SpellReference;
use serde::{Deserialize, Serialize};

/// Request type for creating a new character with full options.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCharacterRequest {
    pub character_name: String,
    pub player_id: Option<i32>, // Optional for NPCs
    pub race: String,
    pub race_source: String,
    pub subrace: Option<String>,
    pub class: String,
    pub class_source: String,
    pub subclass: Option<String>,
    pub background: String,
    pub background_source: String,
    pub ability_score_method: String, // "standard_array", "point_buy", or "manual"
    pub ability_scores: Option<AbilityScoresInput>,
    pub alignment: Option<String>,
    pub personality: Option<PersonalityInput>,
    pub skill_proficiencies: Option<Vec<String>>,
    pub equipment: Option<Vec<InventoryItemInput>>,
    pub cantrips: Option<Vec<SpellReferenceInput>>,
    pub known_spells: Option<Vec<SpellReferenceInput>>,
    // NPC fields
    pub is_npc: Option<bool>,
    pub npc_role: Option<String>,
    pub npc_location: Option<String>,
    pub npc_faction: Option<String>,
    pub npc_notes: Option<String>,
    // Boss NPC abilities
    pub legendary_actions: Option<Vec<LegendaryActionInput>>,
    pub legendary_action_count: Option<i32>,
}

/// Ability scores input for character creation.
#[derive(Debug, Serialize, Deserialize)]
pub struct AbilityScoresInput {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

/// Personality traits input for character creation.
#[derive(Debug, Serialize, Deserialize)]
pub struct PersonalityInput {
    pub traits: Option<String>,
    pub ideals: Option<String>,
    pub bonds: Option<String>,
    pub flaws: Option<String>,
}

/// Inventory item input for character creation/updates.
#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryItemInput {
    pub name: String,
    pub source: Option<String>,
    pub quantity: i32,
    pub weight: f64,
    pub value: f64,
    pub notes: Option<String>,
}

/// Spell reference input for character spell management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellReferenceInput {
    pub name: String,
    pub source: String,
}

impl From<SpellReferenceInput> for SpellReference {
    fn from(input: SpellReferenceInput) -> Self {
        SpellReference::new(input.name, input.source)
    }
}

/// Legendary action input for boss NPCs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendaryActionInput {
    pub name: String,
    pub cost: i32,
    pub description: String,
}

/// Request type for leveling up a character.
#[derive(Debug, Serialize, Deserialize)]
pub struct LevelUpRequest {
    pub class_name: String,
    pub class_source: String,
    pub hit_points_roll: Option<i32>,
    pub take_average_hp: bool,
    pub subclass: Option<String>,
    pub ability_score_improvement: Option<String>, // JSON string with ASI data
    pub feat: Option<String>,
    pub new_spell_slots: Option<String>, // JSON string with spell slot updates
    pub new_known_spells: Option<Vec<SpellReferenceInput>>, // Updated known spells list
    pub new_cantrips: Option<Vec<SpellReferenceInput>>, // Updated cantrips list
}

/// Currency update request for character inventory.
#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencyUpdate {
    pub copper: Option<i32>,
    pub silver: Option<i32>,
    pub electrum: Option<i32>,
    pub gold: Option<i32>,
    pub platinum: Option<i32>,
}

/// Input type for feature reference lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureReferenceInput {
    pub name: String,
    pub class_name: String,
    pub subclass_name: Option<String>,
    pub source: String,
    pub level: i32,
}

/// Feature details returned to frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDetail {
    pub name: String,
    pub class_name: String,
    pub subclass_name: Option<String>,
    pub source: String,
    pub level: i32,
    pub description: String,
}
