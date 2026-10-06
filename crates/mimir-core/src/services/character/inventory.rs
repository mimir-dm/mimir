//! Character inventory.

use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{
    CharacterInventory, NewCharacterInventory, UpdateCharacterInventory,
};
use crate::services::{ServiceError, ServiceResult};

use super::CharacterService;

/// Input for adding an item to inventory.
#[derive(Debug, Clone)]
pub struct AddInventoryInput {
    /// Item name from catalog
    pub item_name: String,
    /// Item source (e.g., "PHB")
    pub item_source: String,
    /// Quantity (default 1)
    pub quantity: Option<i32>,
    /// Whether equipped
    pub equipped: bool,
    /// Whether attuned
    pub attuned: bool,
    /// Notes about the item
    pub notes: Option<String>,
}

impl AddInventoryInput {
    /// Create input for adding an item.
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            item_name: name.into(),
            item_source: source.into(),
            quantity: None,
            equipped: false,
            attuned: false,
            notes: None,
        }
    }

    /// Set quantity.
    pub fn with_quantity(mut self, quantity: i32) -> Self {
        self.quantity = Some(quantity);
        self
    }

    /// Mark as equipped.
    pub fn equipped(mut self) -> Self {
        self.equipped = true;
        self
    }

    /// Mark as attuned.
    pub fn attuned(mut self) -> Self {
        self.attuned = true;
        self
    }

    /// Add notes.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

impl<'a> CharacterService<'a> {
    // --- Inventory Management ---

    /// Add an item to a character's inventory.
    pub fn add_to_inventory(
        &mut self,
        character_id: &str,
        input: AddInventoryInput,
    ) -> ServiceResult<CharacterInventory> {
        // Verify character exists
        if !dal::character_exists(self.conn, character_id)? {
            return Err(ServiceError::not_found("Character", character_id));
        }

        let inv_id = Uuid::new_v4().to_string();
        let notes_ref = input.notes.as_deref();

        let mut new_item =
            NewCharacterInventory::new(&inv_id, character_id, &input.item_name, &input.item_source);

        if let Some(qty) = input.quantity {
            new_item = new_item.with_quantity(qty);
        }
        if input.equipped {
            new_item = new_item.equipped();
        }
        if input.attuned {
            new_item = new_item.attuned();
        }
        if let Some(notes) = notes_ref {
            new_item = new_item.with_notes(notes);
        }

        dal::insert_character_inventory(self.conn, &new_item)?;
        dal::get_character_inventory(self.conn, &inv_id).map_err(ServiceError::from)
    }

    /// Remove an item from a character's inventory.
    pub fn remove_from_inventory(&mut self, inventory_id: &str) -> ServiceResult<()> {
        let rows = dal::delete_character_inventory(self.conn, inventory_id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("InventoryItem", inventory_id));
        }
        Ok(())
    }

    /// Get a character's inventory.
    pub fn get_inventory(&mut self, character_id: &str) -> ServiceResult<Vec<CharacterInventory>> {
        dal::list_character_inventory(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Get equipped items for a character.
    pub fn get_equipped_items(
        &mut self,
        character_id: &str,
    ) -> ServiceResult<Vec<CharacterInventory>> {
        dal::list_equipped_items(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Get attuned items for a character.
    pub fn get_attuned_items(
        &mut self,
        character_id: &str,
    ) -> ServiceResult<Vec<CharacterInventory>> {
        dal::list_attuned_items(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Update an inventory item (quantity, equipped, attuned, notes).
    pub fn update_inventory_item(
        &mut self,
        inventory_id: &str,
        quantity: Option<i32>,
        equipped: Option<bool>,
        attuned: Option<bool>,
    ) -> ServiceResult<CharacterInventory> {
        let update = UpdateCharacterInventory {
            quantity,
            equipped: equipped.map(|e| if e { 1 } else { 0 }),
            attuned: attuned.map(|a| if a { 1 } else { 0 }),
            notes: None,
        };

        let rows = dal::update_character_inventory(self.conn, inventory_id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("InventoryItem", inventory_id));
        }

        dal::get_character_inventory(self.conn, inventory_id).map_err(ServiceError::from)
    }

    /// Count attuned items for a character (D&D 5e max is 3).
    pub fn count_attuned_items(&mut self, character_id: &str) -> ServiceResult<i64> {
        dal::count_attuned_items(self.conn, character_id).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::create_test_campaign;
    use super::super::CreateCharacterInput;
    use super::*;
    use crate::test_utils::setup_test_db;

    #[test]
    fn test_add_to_inventory() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(Some(&campaign_id), "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        let item_input = AddInventoryInput::new("Longsword", "PHB");
        let item = service
            .add_to_inventory(&character.id, item_input)
            .expect("Failed to add item");

        assert_eq!(item.item_name, "Longsword");
        assert_eq!(item.item_source, "PHB");
        assert_eq!(item.quantity, 1);
        assert!(!item.is_equipped());
        assert!(!item.is_attuned());
    }

    #[test]
    fn test_add_to_inventory_with_options() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(Some(&campaign_id), "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        let item_input = AddInventoryInput::new("Cloak of Protection", "DMG")
            .equipped()
            .attuned()
            .with_notes("Found in dungeon");
        let item = service
            .add_to_inventory(&character.id, item_input)
            .expect("Failed to add item");

        assert!(item.is_equipped());
        assert!(item.is_attuned());
        assert_eq!(item.notes, Some("Found in dungeon".to_string()));
    }

    #[test]
    fn test_get_inventory() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(Some(&campaign_id), "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        service
            .add_to_inventory(&character.id, AddInventoryInput::new("Sword", "PHB"))
            .expect("Failed to add item");
        service
            .add_to_inventory(&character.id, AddInventoryInput::new("Shield", "PHB"))
            .expect("Failed to add item");

        let inventory = service
            .get_inventory(&character.id)
            .expect("Failed to get inventory");
        assert_eq!(inventory.len(), 2);
    }

    #[test]
    fn test_remove_from_inventory() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(Some(&campaign_id), "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        let item = service
            .add_to_inventory(&character.id, AddInventoryInput::new("Sword", "PHB"))
            .expect("Failed to add item");

        service
            .remove_from_inventory(&item.id)
            .expect("Failed to remove item");

        let inventory = service
            .get_inventory(&character.id)
            .expect("Failed to get inventory");
        assert_eq!(inventory.len(), 0);
    }

    #[test]
    fn test_update_inventory_item() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(Some(&campaign_id), "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        let item = service
            .add_to_inventory(
                &character.id,
                AddInventoryInput::new("Arrow", "PHB").with_quantity(20),
            )
            .expect("Failed to add item");

        let updated = service
            .update_inventory_item(&item.id, Some(15), Some(true), None)
            .expect("Failed to update item");

        assert_eq!(updated.quantity, 15);
        assert!(updated.is_equipped());
        assert!(!updated.is_attuned());
    }
}
