//! Condition Data Access Layer
//!
//! Database operations for conditions.

use crate::models::catalog::{Condition, ConditionFilter, NewCondition};
use crate::schema::conditions;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new condition.
pub fn insert_condition(conn: &mut SqliteConnection, condition: &NewCondition) -> QueryResult<i32> {
    diesel::insert_into(conditions::table)
        .values(condition)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple conditions in a batch.
pub fn insert_conditions(
    conn: &mut SqliteConnection,
    conditions: &[NewCondition],
) -> QueryResult<usize> {
    diesel::insert_into(conditions::table)
        .values(conditions)
        .execute(conn)
}

/// Get a condition by its ID.
pub fn get_condition(conn: &mut SqliteConnection, id: i32) -> QueryResult<Condition> {
    conditions::table
        .filter(conditions::id.eq(id))
        .first(conn)
}

/// Get a condition by its ID, returning None if not found.
pub fn get_condition_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Condition>> {
    conditions::table
        .filter(conditions::id.eq(id))
        .first(conn)
        .optional()
}

/// Get a condition by name and source.
pub fn get_condition_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Condition>> {
    conditions::table
        .filter(conditions::name.eq(name))
        .filter(conditions::source.eq(source))
        .first(conn)
        .optional()
}

/// List all conditions, ordered by name.
pub fn list_conditions(conn: &mut SqliteConnection) -> QueryResult<Vec<Condition>> {
    conditions::table.order(conditions::name.asc()).load(conn)
}

/// List conditions from a specific source.
pub fn list_conditions_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Condition>> {
    conditions::table
        .filter(conditions::source.eq(source))
        .order(conditions::name.asc())
        .load(conn)
}

/// Delete a condition by its ID.
pub fn delete_condition(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(conditions::table.filter(conditions::id.eq(id))).execute(conn)
}

/// Delete all conditions from a specific source.
pub fn delete_conditions_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(conditions::table.filter(conditions::source.eq(source))).execute(conn)
}

/// Count all conditions.
pub fn count_conditions(conn: &mut SqliteConnection) -> QueryResult<i64> {
    conditions::table.count().get_result(conn)
}

/// Count conditions from a specific source.
pub fn count_conditions_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    conditions::table
        .filter(conditions::source.eq(source))
        .count()
        .get_result(conn)
}

/// List all distinct sources that have conditions.
pub fn list_condition_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    conditions::table
        .select(conditions::source)
        .distinct()
        .order(conditions::source.asc())
        .load(conn)
}

/// Search conditions with filters.
pub fn search_conditions(conn: &mut SqliteConnection, filter: &ConditionFilter) -> QueryResult<Vec<Condition>> {
    // If sources filter is explicitly empty, return no results
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = conditions::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(conditions::name.like(pattern));
    }

    // Use effective_sources() to support both single source and sources array
    if let Some(sources) = filter.effective_sources() {
        query = query.filter(conditions::source.eq_any(sources));
    }

    query.order(conditions::name.asc()).load(conn)
}

/// Search conditions with pagination.
pub fn search_conditions_paginated(
    conn: &mut SqliteConnection,
    filter: &ConditionFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Condition>> {
    // If sources filter is explicitly empty, return no results
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = conditions::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(conditions::name.like(pattern));
    }

    // Use effective_sources() to support both single source and sources array
    if let Some(sources) = filter.effective_sources() {
        query = query.filter(conditions::source.eq_any(sources));
    }

    query
        .order(conditions::name.asc())
        .limit(limit)
        .offset(offset)
        .load(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_condition_crud() {
        let mut conn = setup_test_db_with_sources();

        let condition = NewCondition::new("Blinded", "PHB", r#"{"name":"Blinded"}"#);
        let id = insert_condition(&mut conn, &condition).expect("Failed to insert");

        let retrieved = get_condition(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Blinded");

        let by_name = get_condition_by_name(&mut conn, "Blinded", "PHB")
            .expect("Failed to query")
            .expect("Condition not found");
        assert_eq!(by_name.name, "Blinded");

        delete_condition(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_conditions(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_conditions() {
        let mut conn = setup_test_db_with_sources();

        let conditions = vec![
            NewCondition::new("Blinded", "PHB", r#"{}"#),
            NewCondition::new("Charmed", "PHB", r#"{}"#),
            NewCondition::new("Deafened", "PHB", r#"{}"#),
        ];
        insert_conditions(&mut conn, &conditions).expect("Failed to insert");

        let list = list_conditions(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Blinded"); // Alphabetical
    }
}
