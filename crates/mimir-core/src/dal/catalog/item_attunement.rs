//! Item Attunement Data Access Layer
//!
//! Database operations for class-specific item attunement requirements.

use crate::models::catalog::{ItemAttunementClass, NewItemAttunementClass};
use crate::schema::item_attunement_classes;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert an item attunement class requirement.
pub fn insert_item_attunement_class(
    conn: &mut SqliteConnection,
    attunement: &NewItemAttunementClass,
) -> QueryResult<i32> {
    diesel::insert_into(item_attunement_classes::table)
        .values(attunement)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple item attunement class requirements in a batch.
pub fn insert_item_attunement_classes(
    conn: &mut SqliteConnection,
    attunements: &[NewItemAttunementClass],
) -> QueryResult<usize> {
    diesel::insert_into(item_attunement_classes::table)
        .values(attunements)
        .execute(conn)
}

/// Get all class requirements for an item.
pub fn get_item_attunement_classes(
    conn: &mut SqliteConnection,
    item_id: i32,
) -> QueryResult<Vec<ItemAttunementClass>> {
    item_attunement_classes::table
        .filter(item_attunement_classes::item_id.eq(item_id))
        .order(item_attunement_classes::class_name.asc())
        .load(conn)
}

/// Get all items that can be attuned by a specific class.
pub fn get_items_attuneable_by_class(
    conn: &mut SqliteConnection,
    class_name: &str,
) -> QueryResult<Vec<ItemAttunementClass>> {
    item_attunement_classes::table
        .filter(item_attunement_classes::class_name.eq(class_name))
        .load(conn)
}

/// Get the class names that can attune to a specific item.
pub fn get_attunement_class_names_for_item(
    conn: &mut SqliteConnection,
    item_id: i32,
) -> QueryResult<Vec<String>> {
    item_attunement_classes::table
        .filter(item_attunement_classes::item_id.eq(item_id))
        .select(item_attunement_classes::class_name)
        .order(item_attunement_classes::class_name.asc())
        .load(conn)
}

/// Check if an item has class-specific attunement requirements.
pub fn item_has_class_attunement(conn: &mut SqliteConnection, item_id: i32) -> QueryResult<bool> {
    let count: i64 = item_attunement_classes::table
        .filter(item_attunement_classes::item_id.eq(item_id))
        .count()
        .get_result(conn)?;
    Ok(count > 0)
}

/// Delete all class requirements for an item.
pub fn delete_item_attunement_classes(
    conn: &mut SqliteConnection,
    item_id: i32,
) -> QueryResult<usize> {
    diesel::delete(
        item_attunement_classes::table.filter(item_attunement_classes::item_id.eq(item_id)),
    )
    .execute(conn)
}

/// Count item attunement class associations.
pub fn count_item_attunement_classes(conn: &mut SqliteConnection) -> QueryResult<i64> {
    item_attunement_classes::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::catalog::{insert_item, insert_source};
    use crate::models::catalog::{NewCatalogSource, NewItem};

    fn setup_test_data(conn: &mut SqliteConnection) {
        let source = NewCatalogSource::new("DMG", "Dungeon Master's Guide", true, "2024-01-20T12:00:00Z");
        insert_source(conn, &source).expect("Failed to insert source");
    }

    fn insert_test_item(conn: &mut SqliteConnection, name: &str) -> i32 {
        let data = format!(r#"{{"name":"{}"}}"#, name);
        let item = NewItem::new(name, "DMG", &data).with_rarity("rare");
        insert_item(conn, &item).expect("Failed to insert item")
    }

    #[test]
    fn test_item_attunement_crud() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);
        let item_id = insert_test_item(&mut conn, "Holy Avenger");

        // Insert class requirements (Paladin only)
        let attunements = vec![NewItemAttunementClass::new(item_id, "Paladin")];
        insert_item_attunement_classes(&mut conn, &attunements).expect("Failed to insert");

        // Check has class attunement
        assert!(item_has_class_attunement(&mut conn, item_id).expect("Failed to check"));

        // Get classes for item
        let found = get_item_attunement_classes(&mut conn, item_id).expect("Failed to query");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].class_name, "Paladin");

        // Get class names
        let names = get_attunement_class_names_for_item(&mut conn, item_id).expect("Failed to query");
        assert_eq!(names, vec!["Paladin"]);

        // Delete
        delete_item_attunement_classes(&mut conn, item_id).expect("Failed to delete");
        assert_eq!(
            count_item_attunement_classes(&mut conn).expect("Failed to count"),
            0
        );

        // Verify no more class attunement
        assert!(!item_has_class_attunement(&mut conn, item_id).expect("Failed to check"));
    }

    #[test]
    fn test_multiple_class_attunement() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);
        let item_id = insert_test_item(&mut conn, "Staff of Healing");

        // This item requires attunement by Cleric, Druid, or Bard
        let attunements = vec![
            NewItemAttunementClass::new(item_id, "Bard"),
            NewItemAttunementClass::new(item_id, "Cleric"),
            NewItemAttunementClass::new(item_id, "Druid"),
        ];
        insert_item_attunement_classes(&mut conn, &attunements).expect("Failed to insert");

        let names = get_attunement_class_names_for_item(&mut conn, item_id).expect("Failed to query");
        assert_eq!(names, vec!["Bard", "Cleric", "Druid"]);
    }

    #[test]
    fn test_get_items_by_class() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);
        let holy_avenger_id = insert_test_item(&mut conn, "Holy Avenger");
        let staff_healing_id = insert_test_item(&mut conn, "Staff of Healing");

        // Holy Avenger: Paladin only
        insert_item_attunement_classes(
            &mut conn,
            &[NewItemAttunementClass::new(holy_avenger_id, "Paladin")],
        )
        .expect("Failed to insert");

        // Staff of Healing: Cleric, Druid
        insert_item_attunement_classes(
            &mut conn,
            &[
                NewItemAttunementClass::new(staff_healing_id, "Cleric"),
                NewItemAttunementClass::new(staff_healing_id, "Druid"),
            ],
        )
        .expect("Failed to insert");

        // Paladin can use Holy Avenger
        let paladin_items = get_items_attuneable_by_class(&mut conn, "Paladin").expect("Failed to query");
        assert_eq!(paladin_items.len(), 1);
        assert_eq!(paladin_items[0].item_id, holy_avenger_id);

        // Cleric can use Staff of Healing
        let cleric_items = get_items_attuneable_by_class(&mut conn, "Cleric").expect("Failed to query");
        assert_eq!(cleric_items.len(), 1);
        assert_eq!(cleric_items[0].item_id, staff_healing_id);
    }
}
