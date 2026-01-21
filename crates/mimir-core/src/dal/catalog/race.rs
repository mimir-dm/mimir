//! Race Data Access Layer
//!
//! Database operations for character races.

use crate::models::catalog::{NewRace, Race};
use crate::schema::races;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new race.
pub fn insert_race(conn: &mut SqliteConnection, race: &NewRace) -> QueryResult<i32> {
    diesel::insert_into(races::table)
        .values(race)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple races in a batch.
pub fn insert_races(conn: &mut SqliteConnection, races: &[NewRace]) -> QueryResult<usize> {
    diesel::insert_into(races::table)
        .values(races)
        .execute(conn)
}

/// Get a race by its ID.
pub fn get_race(conn: &mut SqliteConnection, id: i32) -> QueryResult<Race> {
    races::table.filter(races::id.eq(id)).first(conn)
}

/// Get a race by name and source.
pub fn get_race_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Race>> {
    races::table
        .filter(races::name.eq(name))
        .filter(races::source.eq(source))
        .first(conn)
        .optional()
}

/// List all races, ordered by name.
pub fn list_races(conn: &mut SqliteConnection) -> QueryResult<Vec<Race>> {
    races::table.order(races::name.asc()).load(conn)
}

/// List races from a specific source.
pub fn list_races_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<Vec<Race>> {
    races::table
        .filter(races::source.eq(source))
        .order(races::name.asc())
        .load(conn)
}

/// Search races by name pattern.
pub fn search_races_by_name(conn: &mut SqliteConnection, pattern: &str) -> QueryResult<Vec<Race>> {
    let pattern = format!("%{}%", pattern);
    races::table
        .filter(races::name.like(pattern))
        .order(races::name.asc())
        .load(conn)
}

/// Delete a race by its ID.
pub fn delete_race(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(races::table.filter(races::id.eq(id))).execute(conn)
}

/// Delete all races from a specific source.
pub fn delete_races_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<usize> {
    diesel::delete(races::table.filter(races::source.eq(source))).execute(conn)
}

/// Count all races.
pub fn count_races(conn: &mut SqliteConnection) -> QueryResult<i64> {
    races::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_race_crud() {
        let mut conn = setup_test_db_with_sources();

        let race = NewRace::new("Elf", "PHB", r#"{"name":"Elf"}"#);
        let id = insert_race(&mut conn, &race).expect("Failed to insert");

        let retrieved = get_race(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Elf");

        let by_name = get_race_by_name(&mut conn, "Elf", "PHB")
            .expect("Failed to query")
            .expect("Race not found");
        assert_eq!(by_name.name, "Elf");

        delete_race(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_races(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_search_races() {
        let mut conn = setup_test_db_with_sources();

        let races = vec![
            NewRace::new("Elf", "PHB", r#"{}"#),
            NewRace::new("Half-Elf", "PHB", r#"{}"#),
            NewRace::new("Dwarf", "PHB", r#"{}"#),
        ];
        insert_races(&mut conn, &races).expect("Failed to insert");

        let results = search_races_by_name(&mut conn, "elf").expect("Failed to search");
        assert_eq!(results.len(), 2);
    }
}
