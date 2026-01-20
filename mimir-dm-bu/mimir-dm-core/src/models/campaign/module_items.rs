//! Module item association models
//!
//! Links items from the catalog to specific modules, with optional
//! location grouping for treasure distribution.

use crate::schema::module_items;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Database model for module item associations
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Associations)]
#[diesel(table_name = module_items)]
#[diesel(belongs_to(crate::models::campaign::modules::Module))]
pub struct ModuleItem {
    pub id: i32,
    pub module_id: i32,
    pub location: Option<String>,
    pub name: String,
    pub source: String,
    pub quantity: i32,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// New module item for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = module_items)]
pub struct NewModuleItem {
    pub module_id: i32,
    pub location: Option<String>,
    pub name: String,
    pub source: String,
    pub quantity: i32,
    pub notes: Option<String>,
}

/// Module item update structure
#[derive(Debug, Clone, Default, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = module_items)]
pub struct UpdateModuleItem {
    pub location: Option<Option<String>>,
    pub quantity: Option<i32>,
    pub notes: Option<Option<String>>,
}

/// Module item with full item data for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleItemWithData {
    pub id: i32,
    pub module_id: i32,
    pub location: Option<String>,
    pub name: String,
    pub source: String,
    pub quantity: i32,
    pub notes: Option<String>,
    /// Full item data from catalog (optional, loaded on demand)
    pub item_data: Option<serde_json::Value>,
}

impl From<ModuleItem> for ModuleItemWithData {
    fn from(mi: ModuleItem) -> Self {
        Self {
            id: mi.id,
            module_id: mi.module_id,
            location: mi.location,
            name: mi.name,
            source: mi.source,
            quantity: mi.quantity,
            notes: mi.notes,
            item_data: None,
        }
    }
}

/// Grouped items by location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationGroup {
    pub location: Option<String>,
    pub items: Vec<ModuleItemWithData>,
}
