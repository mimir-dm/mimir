//! Class Data Access Layer
//!
//! Database operations for character classes.

use crate::models::catalog::{Class, ClassFilter, NewClass};
use crate::schema::classes;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new class.
pub fn insert_class(conn: &mut SqliteConnection, class: &NewClass) -> QueryResult<i32> {
    diesel::insert_into(classes::table)
        .values(class)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple classes in a batch.
pub fn insert_classes(conn: &mut SqliteConnection, classes: &[NewClass]) -> QueryResult<usize> {
    diesel::insert_into(classes::table)
        .values(classes)
        .execute(conn)
}

/// Get a class by its ID.
pub fn get_class(conn: &mut SqliteConnection, id: i32) -> QueryResult<Class> {
    classes::table.filter(classes::id.eq(id)).first(conn)
}

/// Get a class by its ID, returning None if not found.
pub fn get_class_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Class>> {
    classes::table.filter(classes::id.eq(id)).first(conn).optional()
}

/// Get a class by name and source.
pub fn get_class_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Class>> {
    classes::table
        .filter(classes::name.eq(name))
        .filter(classes::source.eq(source))
        .first(conn)
        .optional()
}

/// List all classes, ordered by name.
pub fn list_classes(conn: &mut SqliteConnection) -> QueryResult<Vec<Class>> {
    classes::table.order(classes::name.asc()).load(conn)
}

/// List classes from a specific source.
pub fn list_classes_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<Vec<Class>> {
    classes::table
        .filter(classes::source.eq(source))
        .order(classes::name.asc())
        .load(conn)
}

/// Delete a class by its ID.
pub fn delete_class(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(classes::table.filter(classes::id.eq(id))).execute(conn)
}

/// Delete all classes from a specific source.
pub fn delete_classes_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<usize> {
    diesel::delete(classes::table.filter(classes::source.eq(source))).execute(conn)
}

/// Count all classes.
pub fn count_classes(conn: &mut SqliteConnection) -> QueryResult<i64> {
    classes::table.count().get_result(conn)
}

/// Count classes from a specific source.
pub fn count_classes_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    classes::table
        .filter(classes::source.eq(source))
        .count()
        .get_result(conn)
}

/// List all distinct sources that have classes.
pub fn list_class_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    classes::table
        .select(classes::source)
        .distinct()
        .order(classes::source.asc())
        .load(conn)
}

/// Search classes with filters.
pub fn search_classes(
    conn: &mut SqliteConnection,
    filter: &ClassFilter,
) -> QueryResult<Vec<Class>> {
    let mut query = classes::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(classes::name.like(pattern));
    }

    if let Some(ref source) = filter.source {
        query = query.filter(classes::source.eq(source));
    }

    query.order(classes::name.asc()).load(conn)
}

/// Search classes with pagination.
pub fn search_classes_paginated(
    conn: &mut SqliteConnection,
    filter: &ClassFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Class>> {
    let mut query = classes::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(classes::name.like(pattern));
    }

    if let Some(ref source) = filter.source {
        query = query.filter(classes::source.eq(source));
    }

    query
        .order(classes::name.asc())
        .limit(limit)
        .offset(offset)
        .load(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_class_crud() {
        let mut conn = setup_test_db_with_sources();

        let class = NewClass::new("Wizard", "PHB", r#"{"name":"Wizard"}"#);
        let id = insert_class(&mut conn, &class).expect("Failed to insert");

        let retrieved = get_class(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Wizard");

        let by_name = get_class_by_name(&mut conn, "Wizard", "PHB")
            .expect("Failed to query")
            .expect("Class not found");
        assert_eq!(by_name.name, "Wizard");

        assert_eq!(count_classes(&mut conn).expect("Failed to count"), 1);

        delete_class(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_classes(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_classes() {
        let mut conn = setup_test_db_with_sources();

        let classes = vec![
            NewClass::new("Fighter", "PHB", r#"{"name":"Fighter"}"#),
            NewClass::new("Wizard", "PHB", r#"{"name":"Wizard"}"#),
        ];
        insert_classes(&mut conn, &classes).expect("Failed to insert");

        let list = list_classes(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Fighter"); // Alphabetical
        assert_eq!(list[1].name, "Wizard");
    }
}
