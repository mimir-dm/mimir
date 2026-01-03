//! Character models
//!
//! Models for managing characters and their version history.

pub mod data;

use crate::schema::{character_versions, characters};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// Re-export commonly used data types
pub use data::{
    AbilityScores, Appearance, CharacterData, ClassLevel, Currency, EquippedItems,
    FeatureReference, InventoryItem, LegendaryAction, Personality, Proficiencies, RoleplayNotes,
    SpellData, SpellReference, SpellSlots,
};

/// Database model for characters (metadata only)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Associations)]
#[diesel(table_name = characters)]
#[diesel(belongs_to(crate::models::campaign::Campaign, foreign_key = campaign_id))]
#[diesel(belongs_to(crate::models::player::Player, foreign_key = player_id))]
pub struct Character {
    pub id: i32,
    pub campaign_id: Option<i32>,
    pub player_id: Option<i32>,
    pub character_name: String,
    pub is_npc: i32,
    pub current_level: i32,
    pub current_version: i32,
    pub directory_path: String,
    pub created_at: String,
    pub last_updated_at: String,
    pub class: Option<String>,
    pub race: Option<String>,
}

impl Character {
    /// Returns true if this character is an NPC (non-player character)
    pub fn is_npc(&self) -> bool {
        self.is_npc != 0
    }
}

/// New character for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = characters)]
pub struct NewCharacter {
    pub campaign_id: Option<i32>,
    pub player_id: Option<i32>,
    pub character_name: String,
    pub is_npc: Option<i32>,
    pub directory_path: String,
    pub class: Option<String>,
    pub race: Option<String>,
}

/// Character update structure
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = characters)]
pub struct UpdateCharacter {
    pub character_name: Option<String>,
    pub is_npc: Option<i32>,
    pub current_level: Option<i32>,
    pub current_version: Option<i32>,
    pub last_updated_at: Option<String>,
    pub campaign_id: Option<Option<i32>>,
    pub directory_path: Option<String>,
}

/// Database model for character versions (full character data)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Associations)]
#[diesel(table_name = character_versions)]
#[diesel(belongs_to(Character, foreign_key = character_id))]
pub struct CharacterVersion {
    pub id: i32,
    pub character_id: i32,
    pub version_number: i32,
    pub file_path: String,
    pub character_data: String, // YAML/JSON blob
    pub snapshot_reason: Option<String>,
    pub level: i32,
    pub created_at: String,
}

/// New character version for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = character_versions)]
pub struct NewCharacterVersion {
    pub character_id: i32,
    pub version_number: i32,
    pub file_path: String,
    pub character_data: String,
    pub snapshot_reason: Option<String>,
    pub level: i32,
}
