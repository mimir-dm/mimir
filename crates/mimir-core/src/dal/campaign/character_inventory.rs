//! CharacterInventory Data Access Layer
//!
//! Database operations for character inventory items.

use crate::models::campaign::{CharacterInventory, NewCharacterInventory, UpdateCharacterInventory};
use crate::schema::character_inventory;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new inventory item.
pub fn insert_character_inventory(
    conn: &mut SqliteConnection,
    item: &NewCharacterInventory,
) -> QueryResult<String> {
    diesel::insert_into(character_inventory::table)
        .values(item)
        .execute(conn)?;

    Ok(item.id.to_string())
}

/// Get an inventory item by ID.
pub fn get_character_inventory(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<CharacterInventory> {
    character_inventory::table.find(id).first(conn)
}

/// Get an inventory item by ID, returning None if not found.
pub fn get_character_inventory_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<CharacterInventory>> {
    character_inventory::table.find(id).first(conn).optional()
}

/// List all inventory items for a character.
pub fn list_character_inventory(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterInventory>> {
    character_inventory::table
        .filter(character_inventory::character_id.eq(character_id))
        .order(character_inventory::item_name.asc())
        .load(conn)
}

/// List equipped items for a character.
pub fn list_equipped_items(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterInventory>> {
    character_inventory::table
        .filter(character_inventory::character_id.eq(character_id))
        .filter(character_inventory::equipped.eq(1))
        .order(character_inventory::item_name.asc())
        .load(conn)
}

/// List attuned items for a character.
pub fn list_attuned_items(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterInventory>> {
    character_inventory::table
        .filter(character_inventory::character_id.eq(character_id))
        .filter(character_inventory::attuned.eq(1))
        .order(character_inventory::item_name.asc())
        .load(conn)
}

/// Count attuned items for a character (max 3 in D&D 5e).
pub fn count_attuned_items(conn: &mut SqliteConnection, character_id: &str) -> QueryResult<i64> {
    character_inventory::table
        .filter(character_inventory::character_id.eq(character_id))
        .filter(character_inventory::attuned.eq(1))
        .count()
        .get_result(conn)
}

/// Update an inventory item.
pub fn update_character_inventory(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateCharacterInventory,
) -> QueryResult<usize> {
    diesel::update(character_inventory::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete an inventory item by ID.
pub fn delete_character_inventory(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(character_inventory::table.find(id)).execute(conn)
}

/// Delete all inventory for a character.
pub fn delete_all_character_inventory(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        character_inventory::table.filter(character_inventory::character_id.eq(character_id)),
    )
    .execute(conn)
}

/// Check if an inventory item exists.
pub fn character_inventory_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(character_inventory::table.find(id))).get_result(conn)
}

/// Count inventory items for a character.
pub fn count_character_inventory(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<i64> {
    character_inventory::table
        .filter(character_inventory::character_id.eq(character_id))
        .count()
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::campaign::{insert_campaign, insert_character};
    use crate::models::campaign::{NewCampaign, NewCharacter};

    fn setup_test_data(conn: &mut SqliteConnection) {
        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");

        let character = NewCharacter::new_pc("char-1", Some("camp-1"), "Hero", "Player");
        insert_character(conn, &character).expect("Failed to create character");
    }

    #[test]
    fn test_insert_and_get_inventory() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let item = NewCharacterInventory::new("inv-1", "char-1", "Longsword", "PHB");
        let id = insert_character_inventory(&mut conn, &item).expect("Failed to insert");
        assert_eq!(id, "inv-1");

        let retrieved = get_character_inventory(&mut conn, "inv-1").expect("Failed to get");
        assert_eq!(retrieved.item_name, "Longsword");
        assert_eq!(retrieved.quantity, 1);
    }

    #[test]
    fn test_list_character_inventory() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let sword = NewCharacterInventory::new("inv-1", "char-1", "Longsword", "PHB");
        let arrows = NewCharacterInventory::new("inv-2", "char-1", "Arrow", "PHB")
            .with_quantity(20);
        insert_character_inventory(&mut conn, &sword).expect("Failed to insert");
        insert_character_inventory(&mut conn, &arrows).expect("Failed to insert");

        let items = list_character_inventory(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_equipped_items() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let sword = NewCharacterInventory::new("inv-1", "char-1", "Longsword", "PHB")
            .equipped();
        let shield = NewCharacterInventory::new("inv-2", "char-1", "Shield", "PHB")
            .equipped();
        let potion = NewCharacterInventory::new("inv-3", "char-1", "Health Potion", "PHB");
        insert_character_inventory(&mut conn, &sword).expect("Failed to insert");
        insert_character_inventory(&mut conn, &shield).expect("Failed to insert");
        insert_character_inventory(&mut conn, &potion).expect("Failed to insert");

        let equipped = list_equipped_items(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(equipped.len(), 2);
    }

    #[test]
    fn test_attuned_items() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let cloak = NewCharacterInventory::new("inv-1", "char-1", "Cloak of Protection", "DMG")
            .equipped()
            .attuned();
        let ring = NewCharacterInventory::new("inv-2", "char-1", "Ring of Protection", "DMG")
            .equipped()
            .attuned();
        let sword = NewCharacterInventory::new("inv-3", "char-1", "Longsword +1", "DMG")
            .equipped();
        insert_character_inventory(&mut conn, &cloak).expect("Failed to insert");
        insert_character_inventory(&mut conn, &ring).expect("Failed to insert");
        insert_character_inventory(&mut conn, &sword).expect("Failed to insert");

        let attuned = list_attuned_items(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(attuned.len(), 2);

        let count = count_attuned_items(&mut conn, "char-1").expect("Failed to count");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_update_inventory() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let item = NewCharacterInventory::new("inv-1", "char-1", "Arrow", "PHB")
            .with_quantity(20);
        insert_character_inventory(&mut conn, &item).expect("Failed to insert");

        let update = UpdateCharacterInventory::set_quantity(15);
        update_character_inventory(&mut conn, "inv-1", &update).expect("Failed to update");

        let retrieved = get_character_inventory(&mut conn, "inv-1").expect("Failed to get");
        assert_eq!(retrieved.quantity, 15);
    }

    #[test]
    fn test_delete_inventory() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let item = NewCharacterInventory::new("inv-1", "char-1", "Sword", "PHB");
        insert_character_inventory(&mut conn, &item).expect("Failed to insert");

        assert!(character_inventory_exists(&mut conn, "inv-1").expect("Failed to check"));

        delete_character_inventory(&mut conn, "inv-1").expect("Failed to delete");

        assert!(!character_inventory_exists(&mut conn, "inv-1").expect("Failed to check"));
    }
}
