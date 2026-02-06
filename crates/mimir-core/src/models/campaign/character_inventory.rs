//! CharacterInventory Model
//!
//! Tracks items held by characters.

use crate::schema::character_inventory;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use ts_rs::TS;

/// An item in a character's inventory.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "bindings/"))]
#[diesel(table_name = character_inventory)]
pub struct CharacterInventory {
    /// Unique ID (UUID)
    pub id: String,
    /// Character who has this item
    pub character_id: String,
    /// Item name
    pub item_name: String,
    /// Item source (e.g., "PHB", "DMG")
    pub item_source: String,
    /// Quantity of this item
    pub quantity: i32,
    /// Whether the item is equipped
    pub equipped: i32,
    /// Whether the item is attuned
    pub attuned: i32,
    /// Additional notes about the item
    pub notes: Option<String>,
}

impl CharacterInventory {
    /// Check if this item is equipped.
    pub fn is_equipped(&self) -> bool {
        self.equipped != 0
    }

    /// Check if this item is attuned.
    pub fn is_attuned(&self) -> bool {
        self.attuned != 0
    }
}

/// Data for inserting a new inventory item.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = character_inventory)]
pub struct NewCharacterInventory<'a> {
    pub id: &'a str,
    pub character_id: &'a str,
    pub item_name: &'a str,
    pub item_source: &'a str,
    pub quantity: i32,
    pub equipped: i32,
    pub attuned: i32,
    pub notes: Option<&'a str>,
}

impl<'a> NewCharacterInventory<'a> {
    /// Create a new inventory item.
    pub fn new(
        id: &'a str,
        character_id: &'a str,
        item_name: &'a str,
        item_source: &'a str,
    ) -> Self {
        Self {
            id,
            character_id,
            item_name,
            item_source,
            quantity: 1,
            equipped: 0,
            attuned: 0,
            notes: None,
        }
    }

    /// Set the quantity.
    pub fn with_quantity(mut self, quantity: i32) -> Self {
        self.quantity = quantity;
        self
    }

    /// Mark as equipped.
    pub fn equipped(mut self) -> Self {
        self.equipped = 1;
        self
    }

    /// Mark as attuned.
    pub fn attuned(mut self) -> Self {
        self.attuned = 1;
        self
    }

    /// Add notes.
    pub fn with_notes(mut self, notes: &'a str) -> Self {
        self.notes = Some(notes);
        self
    }
}

/// Data for updating an inventory item.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = character_inventory)]
pub struct UpdateCharacterInventory<'a> {
    pub quantity: Option<i32>,
    pub equipped: Option<i32>,
    pub attuned: Option<i32>,
    pub notes: Option<Option<&'a str>>,
}

impl<'a> UpdateCharacterInventory<'a> {
    /// Update quantity.
    pub fn set_quantity(quantity: i32) -> Self {
        Self {
            quantity: Some(quantity),
            ..Default::default()
        }
    }

    /// Set equipped status.
    pub fn set_equipped(equipped: bool) -> Self {
        Self {
            equipped: Some(if equipped { 1 } else { 0 }),
            ..Default::default()
        }
    }

    /// Set attuned status.
    pub fn set_attuned(attuned: bool) -> Self {
        Self {
            attuned: Some(if attuned { 1 } else { 0 }),
            ..Default::default()
        }
    }

    /// Set notes.
    pub fn set_notes(notes: Option<&'a str>) -> Self {
        Self {
            notes: Some(notes),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_inventory_item() {
        let item = NewCharacterInventory::new("inv-1", "char-1", "Longsword", "PHB");
        assert_eq!(item.item_name, "Longsword");
        assert_eq!(item.quantity, 1);
        assert_eq!(item.equipped, 0);
        assert_eq!(item.attuned, 0);
    }

    #[test]
    fn test_with_quantity() {
        let item = NewCharacterInventory::new("inv-1", "char-1", "Arrow", "PHB")
            .with_quantity(20);
        assert_eq!(item.quantity, 20);
    }

    #[test]
    fn test_equipped_and_attuned() {
        let item = NewCharacterInventory::new("inv-1", "char-1", "Cloak of Protection", "DMG")
            .equipped()
            .attuned();
        assert_eq!(item.equipped, 1);
        assert_eq!(item.attuned, 1);
    }

    #[test]
    fn test_with_notes() {
        let item = NewCharacterInventory::new("inv-1", "char-1", "Mystery Potion", "PHB")
            .with_notes("Found in dungeon, unidentified");
        assert_eq!(item.notes, Some("Found in dungeon, unidentified"));
    }

    #[test]
    fn test_update_quantity() {
        let update = UpdateCharacterInventory::set_quantity(5);
        assert_eq!(update.quantity, Some(5));
    }

    #[test]
    fn test_update_equipped() {
        let equip = UpdateCharacterInventory::set_equipped(true);
        assert_eq!(equip.equipped, Some(1));

        let unequip = UpdateCharacterInventory::set_equipped(false);
        assert_eq!(unequip.equipped, Some(0));
    }
}
