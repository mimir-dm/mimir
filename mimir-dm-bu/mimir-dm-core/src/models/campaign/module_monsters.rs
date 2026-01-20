//! Module monster association models
//!
//! Links monsters from the catalog to specific modules, with optional
//! encounter grouping for organization during play.

use crate::schema::module_monsters;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Database model for module monster associations
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Associations)]
#[diesel(table_name = module_monsters)]
#[diesel(belongs_to(crate::models::campaign::modules::Module))]
pub struct ModuleMonster {
    pub id: i32,
    pub module_id: i32,
    pub monster_name: String,
    pub monster_source: String,
    pub quantity: i32,
    pub encounter_tag: Option<String>,
    /// Optional custom display name (e.g., "Frost Wight" when using goblin stats)
    pub display_name: Option<String>,
    /// DM notes about customizations or thematic changes
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// New module monster for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = module_monsters)]
pub struct NewModuleMonster {
    pub module_id: i32,
    pub monster_name: String,
    pub monster_source: String,
    pub quantity: i32,
    pub encounter_tag: Option<String>,
    /// Optional custom display name (e.g., "Frost Wight" when using goblin stats)
    pub display_name: Option<String>,
    /// DM notes about customizations or thematic changes
    pub notes: Option<String>,
}

/// Module monster update structure
#[derive(Debug, Clone, Default, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = module_monsters)]
pub struct UpdateModuleMonster {
    pub quantity: Option<i32>,
    pub encounter_tag: Option<Option<String>>,
    /// Optional custom display name (e.g., "Frost Wight" when using goblin stats)
    pub display_name: Option<Option<String>>,
    /// DM notes about customizations or thematic changes
    pub notes: Option<Option<String>>,
}

/// Module monster with full monster data for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMonsterWithData {
    pub id: i32,
    pub module_id: i32,
    pub monster_name: String,
    pub monster_source: String,
    pub quantity: i32,
    pub encounter_tag: Option<String>,
    /// Optional custom display name (e.g., "Frost Wight" when using goblin stats)
    pub display_name: Option<String>,
    /// DM notes about customizations or thematic changes
    pub notes: Option<String>,
    /// Full monster data from catalog (optional, loaded on demand)
    pub monster_data: Option<serde_json::Value>,
}

impl From<ModuleMonster> for ModuleMonsterWithData {
    fn from(mm: ModuleMonster) -> Self {
        Self {
            id: mm.id,
            module_id: mm.module_id,
            monster_name: mm.monster_name,
            monster_source: mm.monster_source,
            quantity: mm.quantity,
            encounter_tag: mm.encounter_tag,
            display_name: mm.display_name,
            notes: mm.notes,
            monster_data: None,
        }
    }
}

/// Grouped monsters by encounter tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterGroup {
    pub encounter_tag: Option<String>,
    pub monsters: Vec<ModuleMonsterWithData>,
}
