//! Cult Data Access Layer
//!
//! Database operations for cults and supernatural gifts.

use crate::models::catalog::{Cult, CultFilter, NewCult};
use crate::schema::cults;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new cult.
pub fn insert_cult(conn: &mut SqliteConnection, cult: &NewCult) -> QueryResult<i32> {
    diesel::insert_into(cults::table)
        .values(cult)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple cults in a batch.
pub fn insert_cults(
    conn: &mut SqliteConnection,
    cults: &[NewCult],
) -> QueryResult<usize> {
    diesel::insert_into(cults::table)
        .values(cults)
        .execute(conn)
}

/// Get a cult by its ID.
pub fn get_cult(conn: &mut SqliteConnection, id: i32) -> QueryResult<Cult> {
    cults::table
        .filter(cults::id.eq(id))
        .first(conn)
}

/// Get a cult by name and source.
pub fn get_cult_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Cult>> {
    cults::table
        .filter(cults::name.eq(name))
        .filter(cults::source.eq(source))
        .first(conn)
        .optional()
}

/// List all cults, ordered by name.
pub fn list_cults(conn: &mut SqliteConnection) -> QueryResult<Vec<Cult>> {
    cults::table.order(cults::name.asc()).load(conn)
}

/// List cults from a specific source.
pub fn list_cults_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Cult>> {
    cults::table
        .filter(cults::source.eq(source))
        .order(cults::name.asc())
        .load(conn)
}

/// Delete a cult by its ID.
pub fn delete_cult(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(cults::table.filter(cults::id.eq(id))).execute(conn)
}

/// Delete all cults from a specific source.
pub fn delete_cults_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(cults::table.filter(cults::source.eq(source))).execute(conn)
}

/// Count all cults.
pub fn count_cults(conn: &mut SqliteConnection) -> QueryResult<i64> {
    cults::table.count().get_result(conn)
}

/// Count cults from a specific source.
pub fn count_cults_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    cults::table
        .filter(cults::source.eq(source))
        .count()
        .get_result(conn)
}

/// Get a cult by its ID, returning None if not found.
pub fn get_cult_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Cult>> {
    cults::table
        .filter(cults::id.eq(id))
        .first(conn)
        .optional()
}

/// List all distinct sources that have cults.
pub fn list_cult_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    cults::table
        .select(cults::source)
        .distinct()
        .order(cults::source.asc())
        .load(conn)
}

/// Search cults with filters.
pub fn search_cults(
    conn: &mut SqliteConnection,
    filter: &CultFilter,
) -> QueryResult<Vec<Cult>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = cults::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(cults::name.like(pattern));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(cults::source.eq_any(sources));
    }

    query.order(cults::name.asc()).load(conn)
}

/// Search cults with pagination.
pub fn search_cults_paginated(
    conn: &mut SqliteConnection,
    filter: &CultFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Cult>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = cults::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(cults::name.like(pattern));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(cults::source.eq_any(sources));
    }

    query
        .order(cults::name.asc())
        .limit(limit)
        .offset(offset)
        .load(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::catalog::insert_source;
    use crate::models::catalog::NewCatalogSource;

    fn setup_test_data(conn: &mut SqliteConnection) {
        let source = NewCatalogSource::new("MM", "Monster Manual", true, "2024-01-20T12:00:00Z");
        insert_source(conn, &source).expect("Failed to insert source");
    }

    #[test]
    fn test_cult_crud() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let cult = NewCult::new("Cult of the Dragon", "MM", r#"{"name":"Cult of the Dragon"}"#);
        let id = insert_cult(&mut conn, &cult).expect("Failed to insert");

        let retrieved = get_cult(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Cult of the Dragon");

        let by_name = get_cult_by_name(&mut conn, "Cult of the Dragon", "MM")
            .expect("Failed to query")
            .expect("Cult not found");
        assert_eq!(by_name.name, "Cult of the Dragon");

        delete_cult(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_cults(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_cults() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let cults = vec![
            NewCult::new("Cult of the Dragon", "MM", r#"{}"#),
            NewCult::new("Cult of the Eternal Flame", "MM", r#"{}"#),
            NewCult::new("Cult of the Howling Hatred", "MM", r#"{}"#),
        ];
        insert_cults(&mut conn, &cults).expect("Failed to insert");

        let list = list_cults(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Cult of the Dragon"); // Alphabetical
    }
}
