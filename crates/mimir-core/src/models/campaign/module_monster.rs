//! ModuleMonster Model
//!
//! Catalog monster instances with optional customizations for use in modules.

use crate::schema::module_monsters;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A monster instance in a module, referencing the catalog with optional customizations.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = module_monsters)]
pub struct ModuleMonster {
    /// Unique ID (UUID)
    pub id: String,
    /// Module this monster belongs to
    pub module_id: String,
    /// Catalog monster name
    pub monster_name: String,
    /// Catalog monster source
    pub monster_source: String,
    /// Display name override (e.g., "Goblin Chief" instead of "Goblin")
    pub display_name: Option<String>,
    /// DM notes for this instance
    pub notes: Option<String>,
    /// Quantity for encounters
    pub quantity: i32,
    /// ISO8601 timestamp of creation
    pub created_at: String,
    /// ISO8601 timestamp of last update
    pub updated_at: String,
}

impl ModuleMonster {
    /// Get the effective display name (custom or catalog name).
    pub fn effective_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.monster_name)
    }

    /// Check if this monster has a custom display name.
    pub fn has_custom_name(&self) -> bool {
        self.display_name.is_some()
    }
}

/// Data for inserting a new module monster.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = module_monsters)]
pub struct NewModuleMonster<'a> {
    pub id: &'a str,
    pub module_id: &'a str,
    pub monster_name: &'a str,
    pub monster_source: &'a str,
    pub display_name: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub quantity: i32,
}

impl<'a> NewModuleMonster<'a> {
    /// Create a new module monster from catalog reference.
    pub fn new(
        id: &'a str,
        module_id: &'a str,
        monster_name: &'a str,
        monster_source: &'a str,
    ) -> Self {
        Self {
            id,
            module_id,
            monster_name,
            monster_source,
            display_name: None,
            notes: None,
            quantity: 1,
        }
    }

    /// Set a custom display name.
    pub fn with_display_name(mut self, name: &'a str) -> Self {
        self.display_name = Some(name);
        self
    }

    /// Set DM notes.
    pub fn with_notes(mut self, notes: &'a str) -> Self {
        self.notes = Some(notes);
        self
    }

    /// Set quantity for encounters.
    pub fn with_quantity(mut self, quantity: i32) -> Self {
        self.quantity = quantity;
        self
    }
}

/// Data for updating a module monster.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = module_monsters)]
pub struct UpdateModuleMonster<'a> {
    pub display_name: Option<Option<&'a str>>,
    pub notes: Option<Option<&'a str>>,
    pub quantity: Option<i32>,
    pub updated_at: Option<&'a str>,
}

impl<'a> UpdateModuleMonster<'a> {
    /// Update the display name.
    pub fn set_display_name(name: Option<&'a str>, updated_at: &'a str) -> Self {
        Self {
            display_name: Some(name),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update the notes.
    pub fn set_notes(notes: Option<&'a str>, updated_at: &'a str) -> Self {
        Self {
            notes: Some(notes),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update the quantity.
    pub fn set_quantity(quantity: i32, updated_at: &'a str) -> Self {
        Self {
            quantity: Some(quantity),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_module_monster() {
        let monster = NewModuleMonster::new("mm-1", "mod-1", "Goblin", "MM");
        assert_eq!(monster.monster_name, "Goblin");
        assert_eq!(monster.monster_source, "MM");
        assert!(monster.display_name.is_none());
        assert_eq!(monster.quantity, 1);
    }

    #[test]
    fn test_with_display_name() {
        let monster = NewModuleMonster::new("mm-1", "mod-1", "Goblin", "MM")
            .with_display_name("Goblin Chief");
        assert_eq!(monster.display_name, Some("Goblin Chief"));
    }

    #[test]
    fn test_with_quantity() {
        let monster = NewModuleMonster::new("mm-1", "mod-1", "Goblin", "MM")
            .with_quantity(6);
        assert_eq!(monster.quantity, 6);
    }

    #[test]
    fn test_with_notes() {
        let monster = NewModuleMonster::new("mm-1", "mod-1", "Goblin", "MM")
            .with_notes("Guards the entrance");
        assert_eq!(monster.notes, Some("Guards the entrance"));
    }

    #[test]
    fn test_update_quantity() {
        let update = UpdateModuleMonster::set_quantity(4, "2024-01-20T12:00:00Z");
        assert_eq!(update.quantity, Some(4));
    }
}
