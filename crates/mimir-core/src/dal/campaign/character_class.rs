//! CharacterClass Data Access Layer
//!
//! Database operations for character class levels.

use crate::models::campaign::{CharacterClass, NewCharacterClass, UpdateCharacterClass};
use crate::schema::character_classes;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new character class.
pub fn insert_character_class(
    conn: &mut SqliteConnection,
    class: &NewCharacterClass,
) -> QueryResult<String> {
    diesel::insert_into(character_classes::table)
        .values(class)
        .execute(conn)?;

    Ok(class.id.to_string())
}

/// Get a character class by ID.
pub fn get_character_class(conn: &mut SqliteConnection, id: &str) -> QueryResult<CharacterClass> {
    character_classes::table.find(id).first(conn)
}

/// Get a character class by ID, returning None if not found.
pub fn get_character_class_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<CharacterClass>> {
    character_classes::table.find(id).first(conn).optional()
}

/// List all classes for a character.
pub fn list_character_classes(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterClass>> {
    character_classes::table
        .filter(character_classes::character_id.eq(character_id))
        .order(character_classes::starting_class.desc()) // Starting class first
        .then_order_by(character_classes::level.desc())
        .load(conn)
}

/// Get the starting class for a character.
pub fn get_starting_class(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Option<CharacterClass>> {
    character_classes::table
        .filter(character_classes::character_id.eq(character_id))
        .filter(character_classes::starting_class.eq(1))
        .first(conn)
        .optional()
}

/// Calculate total character level (sum of all class levels).
pub fn get_total_level(conn: &mut SqliteConnection, character_id: &str) -> QueryResult<i64> {
    use diesel::dsl::sum;

    character_classes::table
        .filter(character_classes::character_id.eq(character_id))
        .select(sum(character_classes::level))
        .first::<Option<i64>>(conn)
        .map(|opt| opt.unwrap_or(0))
}

/// Update a character class.
pub fn update_character_class(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateCharacterClass,
) -> QueryResult<usize> {
    diesel::update(character_classes::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a character class by ID.
pub fn delete_character_class(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(character_classes::table.find(id)).execute(conn)
}

/// Delete all classes for a character.
pub fn delete_character_classes(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        character_classes::table.filter(character_classes::character_id.eq(character_id)),
    )
    .execute(conn)
}

/// Check if a character class exists.
pub fn character_class_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(character_classes::table.find(id))).get_result(conn)
}

/// Count classes for a character.
pub fn count_character_classes(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<i64> {
    character_classes::table
        .filter(character_classes::character_id.eq(character_id))
        .count()
        .get_result(conn)
}

/// Find a character's class entry by class name and source.
/// Used for level up to determine if this is a new multiclass or existing class.
pub fn find_character_class_by_name(
    conn: &mut SqliteConnection,
    character_id: &str,
    class_name: &str,
    class_source: &str,
) -> QueryResult<Option<CharacterClass>> {
    character_classes::table
        .filter(character_classes::character_id.eq(character_id))
        .filter(character_classes::class_name.eq(class_name))
        .filter(character_classes::class_source.eq(class_source))
        .first(conn)
        .optional()
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

        let character = NewCharacter::new_pc("char-1", Some("camp-1"), "Test Hero", "Player");
        insert_character(conn, &character).expect("Failed to create character");
    }

    #[test]
    fn test_insert_and_get_character_class() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let class = NewCharacterClass::starting("class-1", "char-1", "Fighter", "PHB");
        let id = insert_character_class(&mut conn, &class).expect("Failed to insert");
        assert_eq!(id, "class-1");

        let retrieved = get_character_class(&mut conn, "class-1").expect("Failed to get");
        assert_eq!(retrieved.class_name, "Fighter");
        assert_eq!(retrieved.class_source, "PHB");
        assert_eq!(retrieved.starting_class, 1);
    }

    #[test]
    fn test_multiclass_character() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let fighter = NewCharacterClass::starting("class-1", "char-1", "Fighter", "PHB")
            .with_level(5);
        let rogue = NewCharacterClass::multiclass("class-2", "char-1", "Rogue", "PHB")
            .with_level(3);
        insert_character_class(&mut conn, &fighter).expect("Failed to insert");
        insert_character_class(&mut conn, &rogue).expect("Failed to insert");

        let classes = list_character_classes(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(classes.len(), 2);
        // Starting class should be first
        assert_eq!(classes[0].class_name, "Fighter");
        assert_eq!(classes[0].starting_class, 1);

        let total = get_total_level(&mut conn, "char-1").expect("Failed to get total");
        assert_eq!(total, 8);
    }

    #[test]
    fn test_get_starting_class() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let fighter = NewCharacterClass::starting("class-1", "char-1", "Fighter", "PHB");
        let rogue = NewCharacterClass::multiclass("class-2", "char-1", "Rogue", "PHB");
        insert_character_class(&mut conn, &fighter).expect("Failed to insert");
        insert_character_class(&mut conn, &rogue).expect("Failed to insert");

        let starting = get_starting_class(&mut conn, "char-1")
            .expect("Failed to query")
            .expect("Should have starting class");
        assert_eq!(starting.class_name, "Fighter");
    }

    #[test]
    fn test_update_class_level() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let class = NewCharacterClass::starting("class-1", "char-1", "Wizard", "PHB");
        insert_character_class(&mut conn, &class).expect("Failed to insert");

        let update = UpdateCharacterClass::set_level(5);
        update_character_class(&mut conn, "class-1", &update).expect("Failed to update");

        let retrieved = get_character_class(&mut conn, "class-1").expect("Failed to get");
        assert_eq!(retrieved.level, 5);
    }

    #[test]
    fn test_update_subclass() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let class = NewCharacterClass::starting("class-1", "char-1", "Fighter", "PHB")
            .with_level(3);
        insert_character_class(&mut conn, &class).expect("Failed to insert");

        let update = UpdateCharacterClass::set_subclass("Champion", "PHB");
        update_character_class(&mut conn, "class-1", &update).expect("Failed to update");

        let retrieved = get_character_class(&mut conn, "class-1").expect("Failed to get");
        assert_eq!(retrieved.subclass_name, Some("Champion".to_string()));
        assert_eq!(retrieved.subclass_source, Some("PHB".to_string()));
    }

    #[test]
    fn test_delete_character_class() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let class = NewCharacterClass::starting("class-1", "char-1", "Cleric", "PHB");
        insert_character_class(&mut conn, &class).expect("Failed to insert");

        assert!(character_class_exists(&mut conn, "class-1").expect("Failed to check"));

        delete_character_class(&mut conn, "class-1").expect("Failed to delete");

        assert!(!character_class_exists(&mut conn, "class-1").expect("Failed to check"));
    }

    #[test]
    fn test_count_character_classes() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        assert_eq!(
            count_character_classes(&mut conn, "char-1").expect("Failed to count"),
            0
        );

        let class1 = NewCharacterClass::starting("class-1", "char-1", "Fighter", "PHB");
        let class2 = NewCharacterClass::multiclass("class-2", "char-1", "Wizard", "PHB");
        insert_character_class(&mut conn, &class1).expect("Failed to insert");
        insert_character_class(&mut conn, &class2).expect("Failed to insert");

        assert_eq!(
            count_character_classes(&mut conn, "char-1").expect("Failed to count"),
            2
        );
    }
}
