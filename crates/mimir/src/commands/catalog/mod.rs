//! Catalog Commands
//!
//! Tauri commands for searching and retrieving D&D 5e catalog content.
//! Split into sub-modules by entity type for better organization.

pub mod characters;
mod helpers;
pub mod items;
pub mod level_up;
pub mod monsters;
pub mod other;
pub mod spells;
pub mod world;

// Re-export all public functions for backwards compatibility
pub use characters::*;
pub use items::*;
pub use level_up::*;
pub use monsters::*;
pub use other::*;
pub use spells::*;
pub use world::*;

use mimir_core::models::catalog::{
    Action, Background, CatalogTable, Class, ClassFeature, Condition, Cult, Deity, Disease, Feat,
    Hazard, Item, Language, Monster, Object, OptionalFeature, Psionic, Race, Reward, Spell,
    Subclass, SubclassFeature, Trap, VariantRule, Vehicle,
};

use super::CatalogEntity;

// =============================================================================
// CatalogEntity trait implementations
// =============================================================================

impl CatalogEntity for Monster {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        self.fluff.as_deref()
    }
}

impl CatalogEntity for Spell {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        self.fluff.as_deref()
    }
}

impl CatalogEntity for Item {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        self.fluff.as_deref()
    }
}

impl CatalogEntity for Race {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        self.fluff.as_deref()
    }
}

impl CatalogEntity for Background {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        self.fluff.as_deref()
    }
}

impl CatalogEntity for Class {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        self.fluff.as_deref()
    }
}

impl CatalogEntity for ClassFeature {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        None
    }
}

impl CatalogEntity for Subclass {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        self.fluff.as_deref()
    }
}

impl CatalogEntity for SubclassFeature {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        None
    }
}

impl CatalogEntity for Feat {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        self.fluff.as_deref()
    }
}

impl CatalogEntity for Condition {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        self.fluff.as_deref()
    }
}

impl CatalogEntity for Disease {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        self.fluff.as_deref()
    }
}

impl CatalogEntity for Language {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
}

impl CatalogEntity for Trap {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
}

impl CatalogEntity for Hazard {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
}

impl CatalogEntity for Action {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
}

impl CatalogEntity for Deity {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
}

impl CatalogEntity for OptionalFeature {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
}

impl CatalogEntity for CatalogTable {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
}

impl CatalogEntity for VariantRule {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
}

impl CatalogEntity for Vehicle {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        self.fluff.as_deref()
    }
}

impl CatalogEntity for Cult {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
}

impl CatalogEntity for Psionic {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
}

impl CatalogEntity for Reward {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
}

impl CatalogEntity for Object {
    fn id(&self) -> Option<i32> {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn fluff(&self) -> Option<&str> {
        self.fluff.as_deref()
    }
}
