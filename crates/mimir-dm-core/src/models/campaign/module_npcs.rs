//! Module NPC association models
//!
//! Links NPCs (characters with is_npc=true) to specific modules.
//! NPCs are campaign characters, not catalog items like monsters.

use crate::models::character::Character;
use crate::schema::module_npcs;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Database model for module NPC associations
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Associations)]
#[diesel(table_name = module_npcs)]
#[diesel(belongs_to(crate::models::campaign::modules::Module))]
#[diesel(belongs_to(Character))]
pub struct ModuleNpc {
    pub id: i32,
    pub module_id: i32,
    pub character_id: i32,
    pub role: Option<String>,
    pub encounter_tag: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// New module NPC for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = module_npcs)]
pub struct NewModuleNpc {
    pub module_id: i32,
    pub character_id: i32,
    pub role: Option<String>,
    pub encounter_tag: Option<String>,
    pub notes: Option<String>,
}

/// Module NPC update structure
#[derive(Debug, Clone, Default, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = module_npcs)]
pub struct UpdateModuleNpc {
    pub role: Option<Option<String>>,
    pub encounter_tag: Option<Option<String>>,
    pub notes: Option<Option<String>>,
}

/// Module NPC with character data for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleNpcWithCharacter {
    pub id: i32,
    pub module_id: i32,
    pub character_id: i32,
    pub role: Option<String>,
    pub encounter_tag: Option<String>,
    pub notes: Option<String>,
    /// Character name from the linked character
    pub character_name: String,
    /// Full character data (optional, loaded on demand)
    pub character_data: Option<serde_json::Value>,
}

impl ModuleNpcWithCharacter {
    /// Create from ModuleNpc and Character
    pub fn from_parts(npc: ModuleNpc, character: &Character) -> Self {
        Self {
            id: npc.id,
            module_id: npc.module_id,
            character_id: npc.character_id,
            role: npc.role,
            encounter_tag: npc.encounter_tag,
            notes: npc.notes,
            character_name: character.character_name.clone(),
            character_data: None,
        }
    }
}

/// Grouped NPCs by role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleGroup {
    pub role: Option<String>,
    pub npcs: Vec<ModuleNpcWithCharacter>,
}
