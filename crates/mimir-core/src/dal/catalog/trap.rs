//! Trap Data Access Layer
//!
//! Database operations for traps.

use crate::models::catalog::{NewTrap, Trap};
use crate::schema::traps;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new trap.
pub fn insert_trap(conn: &mut SqliteConnection, trap: &NewTrap) -> QueryResult<i32> {
    diesel::insert_into(traps::table)
        .values(trap)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple traps in a batch.
pub fn insert_traps(
    conn: &mut SqliteConnection,
    traps: &[NewTrap],
) -> QueryResult<usize> {
    diesel::insert_into(traps::table)
        .values(traps)
        .execute(conn)
}

/// Get a trap by its ID.
pub fn get_trap(conn: &mut SqliteConnection, id: i32) -> QueryResult<Trap> {
    traps::table
        .filter(traps::id.eq(id))
        .first(conn)
}

/// Get a trap by name and source.
pub fn get_trap_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Trap>> {
    traps::table
        .filter(traps::name.eq(name))
        .filter(traps::source.eq(source))
        .first(conn)
        .optional()
}

/// List all traps, ordered by name.
pub fn list_traps(conn: &mut SqliteConnection) -> QueryResult<Vec<Trap>> {
    traps::table.order(traps::name.asc()).load(conn)
}

/// List traps from a specific source.
pub fn list_traps_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Trap>> {
    traps::table
        .filter(traps::source.eq(source))
        .order(traps::name.asc())
        .load(conn)
}

/// List traps by tier (e.g., "simple", "complex").
pub fn list_traps_by_tier(
    conn: &mut SqliteConnection,
    tier: &str,
) -> QueryResult<Vec<Trap>> {
    traps::table
        .filter(traps::trap_tier.eq(tier))
        .order(traps::name.asc())
        .load(conn)
}

/// List simple traps.
pub fn list_simple_traps(conn: &mut SqliteConnection) -> QueryResult<Vec<Trap>> {
    list_traps_by_tier(conn, "simple")
}

/// List complex traps.
pub fn list_complex_traps(conn: &mut SqliteConnection) -> QueryResult<Vec<Trap>> {
    list_traps_by_tier(conn, "complex")
}

/// Delete a trap by its ID.
pub fn delete_trap(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(traps::table.filter(traps::id.eq(id))).execute(conn)
}

/// Delete all traps from a specific source.
pub fn delete_traps_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(traps::table.filter(traps::source.eq(source))).execute(conn)
}

/// Count all traps.
pub fn count_traps(conn: &mut SqliteConnection) -> QueryResult<i64> {
    traps::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_trap_crud() {
        let mut conn = setup_test_db_with_sources();

        let trap = NewTrap::new("Pit Trap", "DMG", r#"{"name":"Pit Trap"}"#)
            .with_tier("simple");
        let id = insert_trap(&mut conn, &trap).expect("Failed to insert");

        let retrieved = get_trap(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Pit Trap");
        assert_eq!(retrieved.trap_tier, Some("simple".to_string()));

        let by_name = get_trap_by_name(&mut conn, "Pit Trap", "DMG")
            .expect("Failed to query")
            .expect("Trap not found");
        assert_eq!(by_name.name, "Pit Trap");

        delete_trap(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_traps(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_traps() {
        let mut conn = setup_test_db_with_sources();

        let traps = vec![
            NewTrap::new("Collapsing Roof", "DMG", r#"{}"#).with_tier("simple"),
            NewTrap::new("Pit Trap", "DMG", r#"{}"#).with_tier("simple"),
            NewTrap::new("Sphere of Annihilation", "DMG", r#"{}"#).with_tier("complex"),
        ];
        insert_traps(&mut conn, &traps).expect("Failed to insert");

        let list = list_traps(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Collapsing Roof"); // Alphabetical

        let simple = list_simple_traps(&mut conn).expect("Failed to list");
        assert_eq!(simple.len(), 2);

        let complex = list_complex_traps(&mut conn).expect("Failed to list");
        assert_eq!(complex.len(), 1);
    }
}
