//! Object Data Access Layer
//!
//! Database operations for objects.

use crate::models::catalog::{NewObject, Object, ObjectFilter};
use crate::schema::objects;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new object.
pub fn insert_object(conn: &mut SqliteConnection, object: &NewObject) -> QueryResult<i32> {
    diesel::insert_into(objects::table)
        .values(object)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple objects in a batch.
pub fn insert_objects(
    conn: &mut SqliteConnection,
    objects: &[NewObject],
) -> QueryResult<usize> {
    diesel::insert_into(objects::table)
        .values(objects)
        .execute(conn)
}

/// Get an object by its ID.
pub fn get_object(conn: &mut SqliteConnection, id: i32) -> QueryResult<Object> {
    objects::table
        .filter(objects::id.eq(id))
        .first(conn)
}

/// Get an object by name and source.
pub fn get_object_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Object>> {
    objects::table
        .filter(objects::name.eq(name))
        .filter(objects::source.eq(source))
        .first(conn)
        .optional()
}

/// List all objects, ordered by name.
pub fn list_objects(conn: &mut SqliteConnection) -> QueryResult<Vec<Object>> {
    objects::table.order(objects::name.asc()).load(conn)
}

/// List objects from a specific source.
pub fn list_objects_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Object>> {
    objects::table
        .filter(objects::source.eq(source))
        .order(objects::name.asc())
        .load(conn)
}

/// List objects by type.
pub fn list_objects_by_type(
    conn: &mut SqliteConnection,
    object_type: &str,
) -> QueryResult<Vec<Object>> {
    objects::table
        .filter(objects::object_type.eq(object_type))
        .order(objects::name.asc())
        .load(conn)
}

/// Delete an object by its ID.
pub fn delete_object(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(objects::table.filter(objects::id.eq(id))).execute(conn)
}

/// Delete all objects from a specific source.
pub fn delete_objects_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(objects::table.filter(objects::source.eq(source))).execute(conn)
}

/// Count all objects.
pub fn count_objects(conn: &mut SqliteConnection) -> QueryResult<i64> {
    objects::table.count().get_result(conn)
}

/// Count objects from a specific source.
pub fn count_objects_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    objects::table
        .filter(objects::source.eq(source))
        .count()
        .get_result(conn)
}

/// Get an object by its ID, returning None if not found.
pub fn get_object_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Object>> {
    objects::table
        .filter(objects::id.eq(id))
        .first(conn)
        .optional()
}

/// List all distinct sources that have objects.
pub fn list_object_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    objects::table
        .select(objects::source)
        .distinct()
        .order(objects::source.asc())
        .load(conn)
}

/// Search objects with filters.
pub fn search_objects(
    conn: &mut SqliteConnection,
    filter: &ObjectFilter,
) -> QueryResult<Vec<Object>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = objects::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(objects::name.like(pattern));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(objects::source.eq_any(sources));
    }

    if let Some(ref object_type) = filter.object_type {
        query = query.filter(objects::object_type.eq(object_type));
    }

    query.order(objects::name.asc()).load(conn)
}

/// Search objects with pagination.
pub fn search_objects_paginated(
    conn: &mut SqliteConnection,
    filter: &ObjectFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Object>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = objects::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(objects::name.like(pattern));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(objects::source.eq_any(sources));
    }

    if let Some(ref object_type) = filter.object_type {
        query = query.filter(objects::object_type.eq(object_type));
    }

    query
        .order(objects::name.asc())
        .limit(limit)
        .offset(offset)
        .load(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_object_crud() {
        let mut conn = setup_test_db_with_sources();

        let object = NewObject::new("Ballista", "DMG", r#"{"name":"Ballista"}"#)
            .with_type("siege weapon");
        let id = insert_object(&mut conn, &object).expect("Failed to insert");

        let retrieved = get_object(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Ballista");
        assert_eq!(retrieved.object_type, Some("siege weapon".to_string()));

        let by_name = get_object_by_name(&mut conn, "Ballista", "DMG")
            .expect("Failed to query")
            .expect("Object not found");
        assert_eq!(by_name.name, "Ballista");

        delete_object(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_objects(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_objects() {
        let mut conn = setup_test_db_with_sources();

        let objects = vec![
            NewObject::new("Ballista", "DMG", r#"{}"#).with_type("siege weapon"),
            NewObject::new("Cannon", "DMG", r#"{}"#).with_type("siege weapon"),
            NewObject::new("Trebuchet", "DMG", r#"{}"#).with_type("siege weapon"),
        ];
        insert_objects(&mut conn, &objects).expect("Failed to insert");

        let list = list_objects(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Ballista"); // Alphabetical

        let siege = list_objects_by_type(&mut conn, "siege weapon").expect("Failed to list");
        assert_eq!(siege.len(), 3);
    }
}
