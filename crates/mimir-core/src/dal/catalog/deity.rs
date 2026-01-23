//! Deity Data Access Layer
//!
//! Database operations for deities.

use crate::models::catalog::{Deity, DeityFilter, NewDeity};
use crate::schema::deities;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new deity, ignoring duplicates.
///
/// If a deity with the same (name, source) already exists,
/// returns the existing ID without error.
pub fn insert_deity(conn: &mut SqliteConnection, deity: &NewDeity) -> QueryResult<i32> {
    // Try to insert, ignoring conflicts
    diesel::insert_or_ignore_into(deities::table)
        .values(deity)
        .execute(conn)?;

    // Look up the ID (either newly inserted or existing)
    deities::table
        .filter(deities::name.eq(&deity.name))
        .filter(deities::source.eq(&deity.source))
        .select(deities::id)
        .first::<Option<i32>>(conn)?
        .ok_or(diesel::result::Error::NotFound)
}

/// Insert multiple deities in a batch.
pub fn insert_deities(
    conn: &mut SqliteConnection,
    deities: &[NewDeity],
) -> QueryResult<usize> {
    diesel::insert_into(deities::table)
        .values(deities)
        .execute(conn)
}

/// Get a deity by its ID.
pub fn get_deity(conn: &mut SqliteConnection, id: i32) -> QueryResult<Deity> {
    deities::table
        .filter(deities::id.eq(id))
        .first(conn)
}

/// Get a deity by name and source.
pub fn get_deity_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Deity>> {
    deities::table
        .filter(deities::name.eq(name))
        .filter(deities::source.eq(source))
        .first(conn)
        .optional()
}

/// List all deities, ordered by name.
pub fn list_deities(conn: &mut SqliteConnection) -> QueryResult<Vec<Deity>> {
    deities::table.order(deities::name.asc()).load(conn)
}

/// List deities from a specific source.
pub fn list_deities_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Deity>> {
    deities::table
        .filter(deities::source.eq(source))
        .order(deities::name.asc())
        .load(conn)
}

/// List deities by pantheon.
pub fn list_deities_by_pantheon(
    conn: &mut SqliteConnection,
    pantheon: &str,
) -> QueryResult<Vec<Deity>> {
    deities::table
        .filter(deities::pantheon.eq(pantheon))
        .order(deities::name.asc())
        .load(conn)
}

/// Delete a deity by its ID.
pub fn delete_deity(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(deities::table.filter(deities::id.eq(id))).execute(conn)
}

/// Delete all deities from a specific source.
pub fn delete_deities_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(deities::table.filter(deities::source.eq(source))).execute(conn)
}

/// Count all deities.
pub fn count_deities(conn: &mut SqliteConnection) -> QueryResult<i64> {
    deities::table.count().get_result(conn)
}

/// Count deities from a specific source.
pub fn count_deities_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    deities::table
        .filter(deities::source.eq(source))
        .count()
        .get_result(conn)
}

/// Get a deity by its ID, returning None if not found.
pub fn get_deity_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Deity>> {
    deities::table
        .filter(deities::id.eq(id))
        .first(conn)
        .optional()
}

/// List all distinct sources that have deities.
pub fn list_deity_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    deities::table
        .select(deities::source)
        .distinct()
        .order(deities::source.asc())
        .load(conn)
}

/// Search deities with filters.
pub fn search_deities(conn: &mut SqliteConnection, filter: &DeityFilter) -> QueryResult<Vec<Deity>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = deities::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(deities::name.like(pattern));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(deities::source.eq_any(sources));
    }

    if let Some(ref pantheon) = filter.pantheon {
        query = query.filter(deities::pantheon.eq(pantheon));
    }

    query.order(deities::name.asc()).load(conn)
}

/// Search deities with pagination.
pub fn search_deities_paginated(
    conn: &mut SqliteConnection,
    filter: &DeityFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Deity>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = deities::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(deities::name.like(pattern));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(deities::source.eq_any(sources));
    }

    if let Some(ref pantheon) = filter.pantheon {
        query = query.filter(deities::pantheon.eq(pantheon));
    }

    query
        .order(deities::name.asc())
        .limit(limit)
        .offset(offset)
        .load(conn)
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
            CREATE TABLE deities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                source TEXT NOT NULL REFERENCES catalog_sources(code),
                pantheon TEXT,
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
    fn test_deity_crud() {
        let mut conn = setup_test_db();

        let deity = NewDeity::new("Tyr", "PHB", r#"{"name":"Tyr"}"#)
            .with_pantheon("Forgotten Realms");
        let id = insert_deity(&mut conn, &deity).expect("Failed to insert");

        let retrieved = get_deity(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Tyr");
        assert_eq!(retrieved.pantheon, Some("Forgotten Realms".to_string()));

        let by_name = get_deity_by_name(&mut conn, "Tyr", "PHB")
            .expect("Failed to query")
            .expect("Deity not found");
        assert_eq!(by_name.name, "Tyr");

        delete_deity(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_deities(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_deities() {
        let mut conn = setup_test_db();

        let deities = vec![
            NewDeity::new("Bahamut", "PHB", r#"{}"#).with_pantheon("Forgotten Realms"),
            NewDeity::new("Moradin", "PHB", r#"{}"#).with_pantheon("Dwarven"),
            NewDeity::new("Tyr", "PHB", r#"{}"#).with_pantheon("Forgotten Realms"),
        ];
        insert_deities(&mut conn, &deities).expect("Failed to insert");

        let list = list_deities(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Bahamut"); // Alphabetical

        let fr = list_deities_by_pantheon(&mut conn, "Forgotten Realms").expect("Failed to list");
        assert_eq!(fr.len(), 2);
    }
}
