//! Hazard Data Access Layer
//!
//! Database operations for environmental hazards.

use crate::models::catalog::{Hazard, HazardFilter, NewHazard};
use crate::schema::hazards;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new hazard.
pub fn insert_hazard(conn: &mut SqliteConnection, hazard: &NewHazard) -> QueryResult<i32> {
    diesel::insert_into(hazards::table)
        .values(hazard)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple hazards in a batch.
pub fn insert_hazards(
    conn: &mut SqliteConnection,
    hazards: &[NewHazard],
) -> QueryResult<usize> {
    diesel::insert_into(hazards::table)
        .values(hazards)
        .execute(conn)
}

/// Get a hazard by its ID.
pub fn get_hazard(conn: &mut SqliteConnection, id: i32) -> QueryResult<Hazard> {
    hazards::table
        .filter(hazards::id.eq(id))
        .first(conn)
}

/// Get a hazard by its ID, returning None if not found.
pub fn get_hazard_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Hazard>> {
    hazards::table
        .filter(hazards::id.eq(id))
        .first(conn)
        .optional()
}

/// Get a hazard by name and source.
pub fn get_hazard_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Hazard>> {
    hazards::table
        .filter(hazards::name.eq(name))
        .filter(hazards::source.eq(source))
        .first(conn)
        .optional()
}

/// List all hazards, ordered by name.
pub fn list_hazards(conn: &mut SqliteConnection) -> QueryResult<Vec<Hazard>> {
    hazards::table.order(hazards::name.asc()).load(conn)
}

/// List hazards from a specific source.
pub fn list_hazards_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Hazard>> {
    hazards::table
        .filter(hazards::source.eq(source))
        .order(hazards::name.asc())
        .load(conn)
}

/// Delete a hazard by its ID.
pub fn delete_hazard(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(hazards::table.filter(hazards::id.eq(id))).execute(conn)
}

/// Delete all hazards from a specific source.
pub fn delete_hazards_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(hazards::table.filter(hazards::source.eq(source))).execute(conn)
}

/// Count all hazards.
pub fn count_hazards(conn: &mut SqliteConnection) -> QueryResult<i64> {
    hazards::table.count().get_result(conn)
}

/// Count hazards from a specific source.
pub fn count_hazards_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    hazards::table
        .filter(hazards::source.eq(source))
        .count()
        .get_result(conn)
}

/// List all distinct sources that have hazards.
pub fn list_hazard_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    hazards::table
        .select(hazards::source)
        .distinct()
        .order(hazards::source.asc())
        .load(conn)
}

/// Search hazards with filters.
pub fn search_hazards(conn: &mut SqliteConnection, filter: &HazardFilter) -> QueryResult<Vec<Hazard>> {
    let mut query = hazards::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(hazards::name.like(pattern));
    }

    if let Some(ref source) = filter.source {
        query = query.filter(hazards::source.eq(source));
    }

    query.order(hazards::name.asc()).load(conn)
}

/// Search hazards with pagination.
pub fn search_hazards_paginated(
    conn: &mut SqliteConnection,
    filter: &HazardFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Hazard>> {
    let mut query = hazards::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(hazards::name.like(pattern));
    }

    if let Some(ref source) = filter.source {
        query = query.filter(hazards::source.eq(source));
    }

    query
        .order(hazards::name.asc())
        .limit(limit)
        .offset(offset)
        .load(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_hazard_crud() {
        let mut conn = setup_test_db_with_sources();

        let hazard = NewHazard::new("Brown Mold", "DMG", r#"{"name":"Brown Mold"}"#);
        let id = insert_hazard(&mut conn, &hazard).expect("Failed to insert");

        let retrieved = get_hazard(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Brown Mold");

        let by_name = get_hazard_by_name(&mut conn, "Brown Mold", "DMG")
            .expect("Failed to query")
            .expect("Hazard not found");
        assert_eq!(by_name.name, "Brown Mold");

        delete_hazard(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_hazards(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_hazards() {
        let mut conn = setup_test_db_with_sources();

        let hazards = vec![
            NewHazard::new("Brown Mold", "DMG", r#"{}"#),
            NewHazard::new("Green Slime", "DMG", r#"{}"#),
            NewHazard::new("Yellow Mold", "DMG", r#"{}"#),
        ];
        insert_hazards(&mut conn, &hazards).expect("Failed to insert");

        let list = list_hazards(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Brown Mold"); // Alphabetical
    }
}
