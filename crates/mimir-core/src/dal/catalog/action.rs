//! Action Data Access Layer
//!
//! Database operations for actions.

use crate::models::catalog::{Action, ActionFilter, NewAction};
use crate::schema::actions;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new action.
pub fn insert_action(conn: &mut SqliteConnection, action: &NewAction) -> QueryResult<i32> {
    diesel::insert_into(actions::table)
        .values(action)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple actions in a batch.
pub fn insert_actions(
    conn: &mut SqliteConnection,
    actions: &[NewAction],
) -> QueryResult<usize> {
    diesel::insert_into(actions::table)
        .values(actions)
        .execute(conn)
}

/// Get an action by its ID.
pub fn get_action(conn: &mut SqliteConnection, id: i32) -> QueryResult<Action> {
    actions::table
        .filter(actions::id.eq(id))
        .first(conn)
}

/// Get an action by its ID, returning None if not found.
pub fn get_action_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Action>> {
    actions::table
        .filter(actions::id.eq(id))
        .first(conn)
        .optional()
}

/// Get an action by name and source.
pub fn get_action_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Action>> {
    actions::table
        .filter(actions::name.eq(name))
        .filter(actions::source.eq(source))
        .first(conn)
        .optional()
}

/// List all actions, ordered by name.
pub fn list_actions(conn: &mut SqliteConnection) -> QueryResult<Vec<Action>> {
    actions::table.order(actions::name.asc()).load(conn)
}

/// List actions from a specific source.
pub fn list_actions_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Action>> {
    actions::table
        .filter(actions::source.eq(source))
        .order(actions::name.asc())
        .load(conn)
}

/// Delete an action by its ID.
pub fn delete_action(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(actions::table.filter(actions::id.eq(id))).execute(conn)
}

/// Delete all actions from a specific source.
pub fn delete_actions_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(actions::table.filter(actions::source.eq(source))).execute(conn)
}

/// Count all actions.
pub fn count_actions(conn: &mut SqliteConnection) -> QueryResult<i64> {
    actions::table.count().get_result(conn)
}

/// Count actions from a specific source.
pub fn count_actions_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    actions::table
        .filter(actions::source.eq(source))
        .count()
        .get_result(conn)
}

/// List all distinct sources that have actions.
pub fn list_action_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    actions::table
        .select(actions::source)
        .distinct()
        .order(actions::source.asc())
        .load(conn)
}

/// Search actions with filters.
pub fn search_actions(conn: &mut SqliteConnection, filter: &ActionFilter) -> QueryResult<Vec<Action>> {
    // If sources filter is explicitly empty, return no results
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = actions::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(actions::name.like(pattern));
    }

    // Use effective_sources() to support both single source and sources array
    if let Some(sources) = filter.effective_sources() {
        query = query.filter(actions::source.eq_any(sources));
    }

    query.order(actions::name.asc()).load(conn)
}

/// Search actions with pagination.
pub fn search_actions_paginated(
    conn: &mut SqliteConnection,
    filter: &ActionFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Action>> {
    // If sources filter is explicitly empty, return no results
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = actions::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(actions::name.like(pattern));
    }

    // Use effective_sources() to support both single source and sources array
    if let Some(sources) = filter.effective_sources() {
        query = query.filter(actions::source.eq_any(sources));
    }

    query
        .order(actions::name.asc())
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
            CREATE TABLE actions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                source TEXT NOT NULL REFERENCES catalog_sources(code),
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
    fn test_action_crud() {
        let mut conn = setup_test_db();

        let action = NewAction::new("Attack", "PHB", r#"{"name":"Attack"}"#);
        let id = insert_action(&mut conn, &action).expect("Failed to insert");

        let retrieved = get_action(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Attack");

        let by_name = get_action_by_name(&mut conn, "Attack", "PHB")
            .expect("Failed to query")
            .expect("Action not found");
        assert_eq!(by_name.name, "Attack");

        delete_action(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_actions(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_actions() {
        let mut conn = setup_test_db();

        let actions = vec![
            NewAction::new("Attack", "PHB", r#"{}"#),
            NewAction::new("Dash", "PHB", r#"{}"#),
            NewAction::new("Dodge", "PHB", r#"{}"#),
        ];
        insert_actions(&mut conn, &actions).expect("Failed to insert");

        let list = list_actions(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Attack"); // Alphabetical
    }
}
