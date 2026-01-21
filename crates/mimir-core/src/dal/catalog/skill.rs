//! Skill Data Access Layer
//!
//! Database operations for skills.

use crate::models::catalog::{NewSkill, Skill};
use crate::schema::skills;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new skill.
pub fn insert_skill(conn: &mut SqliteConnection, skill: &NewSkill) -> QueryResult<i32> {
    diesel::insert_into(skills::table)
        .values(skill)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple skills in a batch.
pub fn insert_skills(
    conn: &mut SqliteConnection,
    skills: &[NewSkill],
) -> QueryResult<usize> {
    diesel::insert_into(skills::table)
        .values(skills)
        .execute(conn)
}

/// Get a skill by its ID.
pub fn get_skill(conn: &mut SqliteConnection, id: i32) -> QueryResult<Skill> {
    skills::table
        .filter(skills::id.eq(id))
        .first(conn)
}

/// Get a skill by name and source.
pub fn get_skill_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Skill>> {
    skills::table
        .filter(skills::name.eq(name))
        .filter(skills::source.eq(source))
        .first(conn)
        .optional()
}

/// List all skills, ordered by name.
pub fn list_skills(conn: &mut SqliteConnection) -> QueryResult<Vec<Skill>> {
    skills::table.order(skills::name.asc()).load(conn)
}

/// List skills from a specific source.
pub fn list_skills_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Skill>> {
    skills::table
        .filter(skills::source.eq(source))
        .order(skills::name.asc())
        .load(conn)
}

/// List skills by ability score.
pub fn list_skills_by_ability(
    conn: &mut SqliteConnection,
    ability: &str,
) -> QueryResult<Vec<Skill>> {
    skills::table
        .filter(skills::ability.eq(ability))
        .order(skills::name.asc())
        .load(conn)
}

/// Delete a skill by its ID.
pub fn delete_skill(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(skills::table.filter(skills::id.eq(id))).execute(conn)
}

/// Delete all skills from a specific source.
pub fn delete_skills_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(skills::table.filter(skills::source.eq(source))).execute(conn)
}

/// Count all skills.
pub fn count_skills(conn: &mut SqliteConnection) -> QueryResult<i64> {
    skills::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_source;
    use crate::models::catalog::NewCatalogSource;
    use diesel::connection::SimpleConnection;

    fn setup_test_db() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

        conn.batch_execute(
            "CREATE TABLE catalog_sources (
                code TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                imported_at TEXT NOT NULL
            );
            CREATE TABLE skills (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                source TEXT NOT NULL REFERENCES catalog_sources(code),
                ability TEXT,
                data TEXT NOT NULL,
                UNIQUE(name, source)
            );",
        )
        .expect("Failed to create tables");

        let source = NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z");
        insert_source(&mut conn, &source).expect("Failed to insert source");

        conn
    }

    #[test]
    fn test_skill_crud() {
        let mut conn = setup_test_db();

        let skill = NewSkill::new("Stealth", "PHB", r#"{"name":"Stealth"}"#)
            .with_ability("DEX");
        let id = insert_skill(&mut conn, &skill).expect("Failed to insert");

        let retrieved = get_skill(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Stealth");
        assert_eq!(retrieved.ability, Some("DEX".to_string()));

        let by_name = get_skill_by_name(&mut conn, "Stealth", "PHB")
            .expect("Failed to query")
            .expect("Skill not found");
        assert_eq!(by_name.name, "Stealth");

        delete_skill(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_skills(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_skills() {
        let mut conn = setup_test_db();

        let skills = vec![
            NewSkill::new("Acrobatics", "PHB", r#"{}"#).with_ability("DEX"),
            NewSkill::new("Athletics", "PHB", r#"{}"#).with_ability("STR"),
            NewSkill::new("Stealth", "PHB", r#"{}"#).with_ability("DEX"),
        ];
        insert_skills(&mut conn, &skills).expect("Failed to insert");

        let list = list_skills(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Acrobatics"); // Alphabetical

        let dex_skills = list_skills_by_ability(&mut conn, "DEX").expect("Failed to list");
        assert_eq!(dex_skills.len(), 2);
    }
}
