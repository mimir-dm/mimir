//! CharacterProficiency Data Access Layer
//!
//! Database operations for character proficiencies.

use crate::models::campaign::{
    CharacterProficiency, NewCharacterProficiency, UpdateCharacterProficiency,
};
use crate::schema::character_proficiencies;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new proficiency.
pub fn insert_character_proficiency(
    conn: &mut SqliteConnection,
    proficiency: &NewCharacterProficiency,
) -> QueryResult<String> {
    diesel::insert_into(character_proficiencies::table)
        .values(proficiency)
        .execute(conn)?;

    Ok(proficiency.id.to_string())
}

/// Get a proficiency by ID.
pub fn get_character_proficiency(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<CharacterProficiency> {
    character_proficiencies::table.find(id).first(conn)
}

/// Get a proficiency by ID, returning None if not found.
pub fn get_character_proficiency_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<CharacterProficiency>> {
    character_proficiencies::table.find(id).first(conn).optional()
}

/// List all proficiencies for a character.
pub fn list_character_proficiencies(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterProficiency>> {
    character_proficiencies::table
        .filter(character_proficiencies::character_id.eq(character_id))
        .order(character_proficiencies::proficiency_type.asc())
        .then_order_by(character_proficiencies::name.asc())
        .load(conn)
}

/// List proficiencies by type.
pub fn list_proficiencies_by_type(
    conn: &mut SqliteConnection,
    character_id: &str,
    proficiency_type: &str,
) -> QueryResult<Vec<CharacterProficiency>> {
    character_proficiencies::table
        .filter(character_proficiencies::character_id.eq(character_id))
        .filter(character_proficiencies::proficiency_type.eq(proficiency_type))
        .order(character_proficiencies::name.asc())
        .load(conn)
}

/// List skill proficiencies.
pub fn list_skill_proficiencies(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterProficiency>> {
    list_proficiencies_by_type(conn, character_id, "skill")
}

/// List saving throw proficiencies.
pub fn list_save_proficiencies(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterProficiency>> {
    list_proficiencies_by_type(conn, character_id, "save")
}

/// List languages.
pub fn list_languages(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterProficiency>> {
    list_proficiencies_by_type(conn, character_id, "language")
}

/// List expertise proficiencies.
pub fn list_expertise(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterProficiency>> {
    character_proficiencies::table
        .filter(character_proficiencies::character_id.eq(character_id))
        .filter(character_proficiencies::expertise.eq(1))
        .order(character_proficiencies::name.asc())
        .load(conn)
}

/// Check if character has a specific proficiency.
pub fn character_has_proficiency(
    conn: &mut SqliteConnection,
    character_id: &str,
    proficiency_type: &str,
    name: &str,
) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(
        character_proficiencies::table
            .filter(character_proficiencies::character_id.eq(character_id))
            .filter(character_proficiencies::proficiency_type.eq(proficiency_type))
            .filter(character_proficiencies::name.eq(name)),
    ))
    .get_result(conn)
}

/// Update a proficiency.
pub fn update_character_proficiency(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateCharacterProficiency,
) -> QueryResult<usize> {
    diesel::update(character_proficiencies::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a proficiency by ID.
pub fn delete_character_proficiency(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(character_proficiencies::table.find(id)).execute(conn)
}

/// Delete all proficiencies for a character.
pub fn delete_all_character_proficiencies(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        character_proficiencies::table
            .filter(character_proficiencies::character_id.eq(character_id)),
    )
    .execute(conn)
}

/// Check if a proficiency exists.
pub fn character_proficiency_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(character_proficiencies::table.find(id))).get_result(conn)
}

/// Count proficiencies for a character.
pub fn count_character_proficiencies(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<i64> {
    character_proficiencies::table
        .filter(character_proficiencies::character_id.eq(character_id))
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
            CREATE TABLE character_proficiencies (
                id TEXT PRIMARY KEY NOT NULL,
                character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
                proficiency_type TEXT NOT NULL,
                name TEXT NOT NULL,
                expertise INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO campaigns (id, name) VALUES ('camp-1', 'Test Campaign');
            INSERT INTO characters (id, campaign_id, name) VALUES ('char-1', 'camp-1', 'Rogue');
            PRAGMA foreign_keys = ON;
            "#,
        )
        .expect("Failed to create tables");

        conn
    }

    #[test]
    fn test_insert_and_get_proficiency() {
        let mut conn = setup_test_db();

        let prof = NewCharacterProficiency::skill("prof-1", "char-1", "Stealth");
        let id = insert_character_proficiency(&mut conn, &prof).expect("Failed to insert");
        assert_eq!(id, "prof-1");

        let retrieved = get_character_proficiency(&mut conn, "prof-1").expect("Failed to get");
        assert_eq!(retrieved.name, "Stealth");
        assert_eq!(retrieved.proficiency_type, "skill");
    }

    #[test]
    fn test_list_character_proficiencies() {
        let mut conn = setup_test_db();

        let stealth = NewCharacterProficiency::skill("prof-1", "char-1", "Stealth");
        let perception = NewCharacterProficiency::skill("prof-2", "char-1", "Perception");
        let thieves = NewCharacterProficiency::tool("prof-3", "char-1", "Thieves' Tools");
        insert_character_proficiency(&mut conn, &stealth).expect("Failed to insert");
        insert_character_proficiency(&mut conn, &perception).expect("Failed to insert");
        insert_character_proficiency(&mut conn, &thieves).expect("Failed to insert");

        let all = list_character_proficiencies(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_list_by_type() {
        let mut conn = setup_test_db();

        let stealth = NewCharacterProficiency::skill("prof-1", "char-1", "Stealth");
        let perception = NewCharacterProficiency::skill("prof-2", "char-1", "Perception");
        let dex_save = NewCharacterProficiency::save("prof-3", "char-1", "Dexterity");
        insert_character_proficiency(&mut conn, &stealth).expect("Failed to insert");
        insert_character_proficiency(&mut conn, &perception).expect("Failed to insert");
        insert_character_proficiency(&mut conn, &dex_save).expect("Failed to insert");

        let skills = list_skill_proficiencies(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(skills.len(), 2);

        let saves = list_save_proficiencies(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(saves.len(), 1);
    }

    #[test]
    fn test_expertise() {
        let mut conn = setup_test_db();

        let stealth = NewCharacterProficiency::skill("prof-1", "char-1", "Stealth")
            .with_expertise();
        let perception = NewCharacterProficiency::skill("prof-2", "char-1", "Perception");
        insert_character_proficiency(&mut conn, &stealth).expect("Failed to insert");
        insert_character_proficiency(&mut conn, &perception).expect("Failed to insert");

        let expertise = list_expertise(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(expertise.len(), 1);
        assert_eq!(expertise[0].name, "Stealth");
    }

    #[test]
    fn test_character_has_proficiency() {
        let mut conn = setup_test_db();

        assert!(!character_has_proficiency(&mut conn, "char-1", "skill", "Stealth")
            .expect("Failed to check"));

        let prof = NewCharacterProficiency::skill("prof-1", "char-1", "Stealth");
        insert_character_proficiency(&mut conn, &prof).expect("Failed to insert");

        assert!(character_has_proficiency(&mut conn, "char-1", "skill", "Stealth")
            .expect("Failed to check"));
    }

    #[test]
    fn test_update_expertise() {
        let mut conn = setup_test_db();

        let prof = NewCharacterProficiency::skill("prof-1", "char-1", "Stealth");
        insert_character_proficiency(&mut conn, &prof).expect("Failed to insert");

        let update = UpdateCharacterProficiency::set_expertise(true);
        update_character_proficiency(&mut conn, "prof-1", &update).expect("Failed to update");

        let retrieved = get_character_proficiency(&mut conn, "prof-1").expect("Failed to get");
        assert_eq!(retrieved.expertise, 1);
    }

    #[test]
    fn test_languages() {
        let mut conn = setup_test_db();

        let common = NewCharacterProficiency::language("prof-1", "char-1", "Common");
        let elvish = NewCharacterProficiency::language("prof-2", "char-1", "Elvish");
        let thieves_cant = NewCharacterProficiency::language("prof-3", "char-1", "Thieves' Cant");
        insert_character_proficiency(&mut conn, &common).expect("Failed to insert");
        insert_character_proficiency(&mut conn, &elvish).expect("Failed to insert");
        insert_character_proficiency(&mut conn, &thieves_cant).expect("Failed to insert");

        let languages = list_languages(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(languages.len(), 3);
    }

    #[test]
    fn test_delete_proficiency() {
        let mut conn = setup_test_db();

        let prof = NewCharacterProficiency::skill("prof-1", "char-1", "Acrobatics");
        insert_character_proficiency(&mut conn, &prof).expect("Failed to insert");

        assert!(character_proficiency_exists(&mut conn, "prof-1").expect("Failed to check"));

        delete_character_proficiency(&mut conn, "prof-1").expect("Failed to delete");

        assert!(!character_proficiency_exists(&mut conn, "prof-1").expect("Failed to check"));
    }
}
