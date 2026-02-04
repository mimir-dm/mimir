//! CatalogTable Data Access Layer
//!
//! Database operations for catalog tables.

use crate::models::catalog::{CatalogTable, CatalogTableFilter, NewCatalogTable};
use crate::schema::catalog_tables;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new catalog table.
pub fn insert_catalog_table(
    conn: &mut SqliteConnection,
    table: &NewCatalogTable,
) -> QueryResult<i32> {
    diesel::insert_into(catalog_tables::table)
        .values(table)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple catalog tables in a batch.
pub fn insert_catalog_tables(
    conn: &mut SqliteConnection,
    tables: &[NewCatalogTable],
) -> QueryResult<usize> {
    diesel::insert_into(catalog_tables::table)
        .values(tables)
        .execute(conn)
}

/// Get a catalog table by its ID.
pub fn get_catalog_table(conn: &mut SqliteConnection, id: i32) -> QueryResult<CatalogTable> {
    catalog_tables::table
        .filter(catalog_tables::id.eq(id))
        .first(conn)
}

/// Get a catalog table by name and source.
pub fn get_catalog_table_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<CatalogTable>> {
    catalog_tables::table
        .filter(catalog_tables::name.eq(name))
        .filter(catalog_tables::source.eq(source))
        .first(conn)
        .optional()
}

/// List all catalog tables, ordered by name.
pub fn list_catalog_tables(conn: &mut SqliteConnection) -> QueryResult<Vec<CatalogTable>> {
    catalog_tables::table
        .order(catalog_tables::name.asc())
        .load(conn)
}

/// List catalog tables from a specific source.
pub fn list_catalog_tables_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<CatalogTable>> {
    catalog_tables::table
        .filter(catalog_tables::source.eq(source))
        .order(catalog_tables::name.asc())
        .load(conn)
}

/// Delete a catalog table by its ID.
pub fn delete_catalog_table(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(catalog_tables::table.filter(catalog_tables::id.eq(id))).execute(conn)
}

/// Delete all catalog tables from a specific source.
pub fn delete_catalog_tables_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(catalog_tables::table.filter(catalog_tables::source.eq(source))).execute(conn)
}

/// Count all catalog tables.
pub fn count_catalog_tables(conn: &mut SqliteConnection) -> QueryResult<i64> {
    catalog_tables::table.count().get_result(conn)
}

/// Count catalog tables from a specific source.
pub fn count_catalog_tables_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    catalog_tables::table
        .filter(catalog_tables::source.eq(source))
        .count()
        .get_result(conn)
}

/// Get a catalog table by its ID, returning None if not found.
pub fn get_catalog_table_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<CatalogTable>> {
    catalog_tables::table
        .filter(catalog_tables::id.eq(id))
        .first(conn)
        .optional()
}

/// List all distinct sources that have catalog tables.
pub fn list_catalog_table_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    catalog_tables::table
        .select(catalog_tables::source)
        .distinct()
        .order(catalog_tables::source.asc())
        .load(conn)
}

/// Search catalog tables with filters.
pub fn search_catalog_tables(
    conn: &mut SqliteConnection,
    filter: &CatalogTableFilter,
) -> QueryResult<Vec<CatalogTable>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = catalog_tables::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(catalog_tables::name.like(pattern));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(catalog_tables::source.eq_any(sources));
    }

    query.order(catalog_tables::name.asc()).load(conn)
}

/// Search catalog tables with pagination.
pub fn search_catalog_tables_paginated(
    conn: &mut SqliteConnection,
    filter: &CatalogTableFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<CatalogTable>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = catalog_tables::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(catalog_tables::name.like(pattern));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(catalog_tables::source.eq_any(sources));
    }

    query
        .order(catalog_tables::name.asc())
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
        let source = NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z");
        insert_source(conn, &source).expect("Failed to insert source");
    }

    #[test]
    fn test_catalog_table_crud() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let table = NewCatalogTable::new("Wild Magic Surge", "PHB", r#"{"name":"Wild Magic Surge"}"#);
        let id = insert_catalog_table(&mut conn, &table).expect("Failed to insert");

        let retrieved = get_catalog_table(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Wild Magic Surge");

        delete_catalog_table(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_catalog_tables(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_catalog_tables() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let tables = vec![
            NewCatalogTable::new("Wild Magic Surge", "PHB", r#"{}"#),
            NewCatalogTable::new("Trinkets", "PHB", r#"{}"#),
        ];
        insert_catalog_tables(&mut conn, &tables).expect("Failed to insert");

        let list = list_catalog_tables(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Trinkets"); // Alphabetical
    }
}
