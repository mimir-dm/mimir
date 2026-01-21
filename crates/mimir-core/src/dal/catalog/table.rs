//! CatalogTable Data Access Layer
//!
//! Database operations for catalog tables.

use crate::models::catalog::{CatalogTable, NewCatalogTable};
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
            CREATE TABLE catalog_tables (
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
    fn test_catalog_table_crud() {
        let mut conn = setup_test_db();

        let table = NewCatalogTable::new("Wild Magic Surge", "PHB", r#"{"name":"Wild Magic Surge"}"#);
        let id = insert_catalog_table(&mut conn, &table).expect("Failed to insert");

        let retrieved = get_catalog_table(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Wild Magic Surge");

        delete_catalog_table(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_catalog_tables(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_catalog_tables() {
        let mut conn = setup_test_db();

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
