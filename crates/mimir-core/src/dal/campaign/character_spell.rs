//! CharacterSpell Data Access Layer
//!
//! Database operations for character spells.

use crate::models::campaign::{CharacterSpell, NewCharacterSpell, UpdateCharacterSpell};
use crate::schema::character_spells;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new character spell.
pub fn insert_character_spell(
    conn: &mut SqliteConnection,
    spell: &NewCharacterSpell,
) -> QueryResult<String> {
    diesel::insert_into(character_spells::table)
        .values(spell)
        .execute(conn)?;

    Ok(spell.id.to_string())
}

/// Get a character spell by ID.
pub fn get_character_spell(conn: &mut SqliteConnection, id: &str) -> QueryResult<CharacterSpell> {
    character_spells::table.find(id).first(conn)
}

/// Get a character spell by ID, returning None if not found.
pub fn get_character_spell_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<CharacterSpell>> {
    character_spells::table.find(id).first(conn).optional()
}

/// List all spells for a character.
pub fn list_character_spells(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterSpell>> {
    character_spells::table
        .filter(character_spells::character_id.eq(character_id))
        .order(character_spells::spell_name.asc())
        .load(conn)
}

/// List spells by source class.
pub fn list_spells_by_class(
    conn: &mut SqliteConnection,
    character_id: &str,
    source_class: &str,
) -> QueryResult<Vec<CharacterSpell>> {
    character_spells::table
        .filter(character_spells::character_id.eq(character_id))
        .filter(character_spells::source_class.eq(source_class))
        .order(character_spells::spell_name.asc())
        .load(conn)
}

/// List prepared spells for a character.
pub fn list_prepared_spells(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterSpell>> {
    character_spells::table
        .filter(character_spells::character_id.eq(character_id))
        .filter(character_spells::prepared.eq(1))
        .order(character_spells::spell_name.asc())
        .load(conn)
}

/// Count prepared spells for a character.
pub fn count_prepared_spells(conn: &mut SqliteConnection, character_id: &str) -> QueryResult<i64> {
    character_spells::table
        .filter(character_spells::character_id.eq(character_id))
        .filter(character_spells::prepared.eq(1))
        .count()
        .get_result(conn)
}

/// Check if character knows a specific spell.
pub fn character_knows_spell(
    conn: &mut SqliteConnection,
    character_id: &str,
    spell_name: &str,
) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(
        character_spells::table
            .filter(character_spells::character_id.eq(character_id))
            .filter(character_spells::spell_name.eq(spell_name)),
    ))
    .get_result(conn)
}

/// Update a character spell.
pub fn update_character_spell(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateCharacterSpell,
) -> QueryResult<usize> {
    diesel::update(character_spells::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a character spell by ID.
pub fn delete_character_spell(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(character_spells::table.find(id)).execute(conn)
}

/// Delete all spells for a character.
pub fn delete_all_character_spells(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        character_spells::table.filter(character_spells::character_id.eq(character_id)),
    )
    .execute(conn)
}

/// Check if a character spell exists.
pub fn character_spell_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(character_spells::table.find(id))).get_result(conn)
}

/// Count spells for a character.
pub fn count_character_spells(conn: &mut SqliteConnection, character_id: &str) -> QueryResult<i64> {
    character_spells::table
        .filter(character_spells::character_id.eq(character_id))
        .count()
        .get_result(conn)
}

/// Find a character spell by name and source class.
/// Used for spell swap operations during level up.
pub fn find_character_spell_by_name(
    conn: &mut SqliteConnection,
    character_id: &str,
    spell_name: &str,
    source_class: &str,
) -> QueryResult<Option<CharacterSpell>> {
    character_spells::table
        .filter(character_spells::character_id.eq(character_id))
        .filter(character_spells::spell_name.eq(spell_name))
        .filter(character_spells::source_class.eq(source_class))
        .first(conn)
        .optional()
}

/// Count spells for a character by source class.
pub fn count_spells_by_class(
    conn: &mut SqliteConnection,
    character_id: &str,
    source_class: &str,
) -> QueryResult<i64> {
    character_spells::table
        .filter(character_spells::character_id.eq(character_id))
        .filter(character_spells::source_class.eq(source_class))
        .count()
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;

    fn setup_test_db() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

        conn.batch_execute(
            r#"
            CREATE TABLE campaigns (id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL);
            CREATE TABLE characters (
                id TEXT PRIMARY KEY NOT NULL,
                campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
                name TEXT NOT NULL
            );
            CREATE TABLE character_spells (
                id TEXT PRIMARY KEY NOT NULL,
                character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
                spell_name TEXT NOT NULL,
                spell_source TEXT NOT NULL,
                source_class TEXT NOT NULL,
                prepared INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO campaigns (id, name) VALUES ('camp-1', 'Test Campaign');
            INSERT INTO characters (id, campaign_id, name) VALUES ('char-1', 'camp-1', 'Wizard');
            PRAGMA foreign_keys = ON;
            "#,
        )
        .expect("Failed to create tables");

        conn
    }

    #[test]
    fn test_insert_and_get_spell() {
        let mut conn = setup_test_db();

        let spell = NewCharacterSpell::new("spell-1", "char-1", "Fireball", "PHB", "Wizard");
        let id = insert_character_spell(&mut conn, &spell).expect("Failed to insert");
        assert_eq!(id, "spell-1");

        let retrieved = get_character_spell(&mut conn, "spell-1").expect("Failed to get");
        assert_eq!(retrieved.spell_name, "Fireball");
        assert_eq!(retrieved.source_class, "Wizard");
    }

    #[test]
    fn test_list_character_spells() {
        let mut conn = setup_test_db();

        let fireball = NewCharacterSpell::new("spell-1", "char-1", "Fireball", "PHB", "Wizard");
        let shield = NewCharacterSpell::new("spell-2", "char-1", "Shield", "PHB", "Wizard");
        let mm = NewCharacterSpell::new("spell-3", "char-1", "Magic Missile", "PHB", "Wizard");
        insert_character_spell(&mut conn, &fireball).expect("Failed to insert");
        insert_character_spell(&mut conn, &shield).expect("Failed to insert");
        insert_character_spell(&mut conn, &mm).expect("Failed to insert");

        let spells = list_character_spells(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(spells.len(), 3);
    }

    #[test]
    fn test_prepared_spells() {
        let mut conn = setup_test_db();

        let fireball = NewCharacterSpell::new("spell-1", "char-1", "Fireball", "PHB", "Wizard")
            .prepared();
        let shield = NewCharacterSpell::new("spell-2", "char-1", "Shield", "PHB", "Wizard")
            .prepared();
        let mm = NewCharacterSpell::new("spell-3", "char-1", "Magic Missile", "PHB", "Wizard");
        insert_character_spell(&mut conn, &fireball).expect("Failed to insert");
        insert_character_spell(&mut conn, &shield).expect("Failed to insert");
        insert_character_spell(&mut conn, &mm).expect("Failed to insert");

        let prepared = list_prepared_spells(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(prepared.len(), 2);

        let count = count_prepared_spells(&mut conn, "char-1").expect("Failed to count");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_spells_by_class() {
        let mut conn = setup_test_db();

        // Multiclass wizard/cleric
        let fireball = NewCharacterSpell::new("spell-1", "char-1", "Fireball", "PHB", "Wizard");
        let cure = NewCharacterSpell::new("spell-2", "char-1", "Cure Wounds", "PHB", "Cleric");
        let shield = NewCharacterSpell::new("spell-3", "char-1", "Shield", "PHB", "Wizard");
        insert_character_spell(&mut conn, &fireball).expect("Failed to insert");
        insert_character_spell(&mut conn, &cure).expect("Failed to insert");
        insert_character_spell(&mut conn, &shield).expect("Failed to insert");

        let wizard_spells =
            list_spells_by_class(&mut conn, "char-1", "Wizard").expect("Failed to list");
        assert_eq!(wizard_spells.len(), 2);

        let cleric_spells =
            list_spells_by_class(&mut conn, "char-1", "Cleric").expect("Failed to list");
        assert_eq!(cleric_spells.len(), 1);
    }

    #[test]
    fn test_character_knows_spell() {
        let mut conn = setup_test_db();

        assert!(!character_knows_spell(&mut conn, "char-1", "Fireball").expect("Failed to check"));

        let spell = NewCharacterSpell::new("spell-1", "char-1", "Fireball", "PHB", "Wizard");
        insert_character_spell(&mut conn, &spell).expect("Failed to insert");

        assert!(character_knows_spell(&mut conn, "char-1", "Fireball").expect("Failed to check"));
    }

    #[test]
    fn test_update_prepared_status() {
        let mut conn = setup_test_db();

        let spell = NewCharacterSpell::new("spell-1", "char-1", "Fireball", "PHB", "Wizard");
        insert_character_spell(&mut conn, &spell).expect("Failed to insert");

        let update = UpdateCharacterSpell::set_prepared(true);
        update_character_spell(&mut conn, "spell-1", &update).expect("Failed to update");

        let retrieved = get_character_spell(&mut conn, "spell-1").expect("Failed to get");
        assert_eq!(retrieved.prepared, 1);
    }

    #[test]
    fn test_delete_spell() {
        let mut conn = setup_test_db();

        let spell = NewCharacterSpell::new("spell-1", "char-1", "Fireball", "PHB", "Wizard");
        insert_character_spell(&mut conn, &spell).expect("Failed to insert");

        assert!(character_spell_exists(&mut conn, "spell-1").expect("Failed to check"));

        delete_character_spell(&mut conn, "spell-1").expect("Failed to delete");

        assert!(!character_spell_exists(&mut conn, "spell-1").expect("Failed to check"));
    }
}
