//! Catalog Source Data Access Layer
//!
//! Database operations for catalog sources.

use crate::models::catalog::{CatalogSource, NewCatalogSource, UpdateCatalogSource};
use crate::schema::catalog_sources;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new catalog source.
///
/// Returns the inserted source code on success.
pub fn insert_source(conn: &mut SqliteConnection, source: &NewCatalogSource) -> QueryResult<String> {
    diesel::insert_into(catalog_sources::table)
        .values(source)
        .execute(conn)?;

    Ok(source.code.to_string())
}

/// Insert multiple catalog sources in a batch.
pub fn insert_sources(
    conn: &mut SqliteConnection,
    sources: &[NewCatalogSource],
) -> QueryResult<usize> {
    diesel::insert_into(catalog_sources::table)
        .values(sources)
        .execute(conn)
}

/// Get a catalog source by its code.
pub fn get_source(conn: &mut SqliteConnection, code: &str) -> QueryResult<CatalogSource> {
    catalog_sources::table.find(code).first(conn)
}

/// Get a catalog source by its code, returning None if not found.
pub fn get_source_optional(
    conn: &mut SqliteConnection,
    code: &str,
) -> QueryResult<Option<CatalogSource>> {
    catalog_sources::table.find(code).first(conn).optional()
}

/// List all catalog sources.
pub fn list_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<CatalogSource>> {
    catalog_sources::table
        .order(catalog_sources::name.asc())
        .load(conn)
}

/// List only enabled catalog sources.
pub fn list_enabled_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<CatalogSource>> {
    catalog_sources::table
        .filter(catalog_sources::enabled.eq(1))
        .order(catalog_sources::name.asc())
        .load(conn)
}

/// Set the enabled status for a catalog source.
pub fn set_enabled(conn: &mut SqliteConnection, code: &str, enabled: bool) -> QueryResult<usize> {
    let update = UpdateCatalogSource::set_enabled(enabled);
    diesel::update(catalog_sources::table.find(code))
        .set(&update)
        .execute(conn)
}

/// Delete a catalog source by its code.
///
/// Note: This will fail if there are entities referencing this source.
pub fn delete_source(conn: &mut SqliteConnection, code: &str) -> QueryResult<usize> {
    diesel::delete(catalog_sources::table.find(code)).execute(conn)
}

/// Check if a source exists.
pub fn source_exists(conn: &mut SqliteConnection, code: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(catalog_sources::table.find(code))).get_result(conn)
}

/// Count all catalog sources.
pub fn count_sources(conn: &mut SqliteConnection) -> QueryResult<i64> {
    catalog_sources::table.count().get_result(conn)
}

/// Count enabled catalog sources.
pub fn count_enabled_sources(conn: &mut SqliteConnection) -> QueryResult<i64> {
    catalog_sources::table
        .filter(catalog_sources::enabled.eq(1))
        .count()
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;

    fn setup_test_db() -> SqliteConnection {
        let mut conn = SqliteConnection::establish(":memory:")
            .expect("Failed to create in-memory database");

        // Create the table
        conn.batch_execute(
            "CREATE TABLE catalog_sources (
                code TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                imported_at TEXT NOT NULL
            );
            CREATE INDEX idx_catalog_sources_enabled ON catalog_sources(enabled);"
        ).expect("Failed to create table");

        conn
    }

    #[test]
    fn test_insert_and_get_source() {
        let mut conn = setup_test_db();

        let source = NewCatalogSource::new(
            "PHB",
            "Player's Handbook",
            true,
            "2024-01-20T12:00:00Z",
        );

        let code = insert_source(&mut conn, &source).expect("Failed to insert");
        assert_eq!(code, "PHB");

        let retrieved = get_source(&mut conn, "PHB").expect("Failed to get");
        assert_eq!(retrieved.code, "PHB");
        assert_eq!(retrieved.name, "Player's Handbook");
        assert_eq!(retrieved.enabled, 1);
    }

    #[test]
    fn test_insert_multiple_sources() {
        let mut conn = setup_test_db();

        let sources = vec![
            NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z"),
            NewCatalogSource::new("MM", "Monster Manual", true, "2024-01-20T12:00:00Z"),
            NewCatalogSource::new("DMG", "Dungeon Master's Guide", false, "2024-01-20T12:00:00Z"),
        ];

        let count = insert_sources(&mut conn, &sources).expect("Failed to insert");
        assert_eq!(count, 3);

        let all = list_sources(&mut conn).expect("Failed to list");
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_list_enabled_sources() {
        let mut conn = setup_test_db();

        let sources = vec![
            NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z"),
            NewCatalogSource::new("MM", "Monster Manual", true, "2024-01-20T12:00:00Z"),
            NewCatalogSource::new("DMG", "Dungeon Master's Guide", false, "2024-01-20T12:00:00Z"),
        ];
        insert_sources(&mut conn, &sources).expect("Failed to insert");

        let enabled = list_enabled_sources(&mut conn).expect("Failed to list enabled");
        assert_eq!(enabled.len(), 2);
        assert!(enabled.iter().all(|s| s.enabled == 1));
    }

    #[test]
    fn test_set_enabled() {
        let mut conn = setup_test_db();

        let source = NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z");
        insert_source(&mut conn, &source).expect("Failed to insert");

        // Disable
        set_enabled(&mut conn, "PHB", false).expect("Failed to set enabled");
        let retrieved = get_source(&mut conn, "PHB").expect("Failed to get");
        assert_eq!(retrieved.enabled, 0);

        // Re-enable
        set_enabled(&mut conn, "PHB", true).expect("Failed to set enabled");
        let retrieved = get_source(&mut conn, "PHB").expect("Failed to get");
        assert_eq!(retrieved.enabled, 1);
    }

    #[test]
    fn test_get_source_optional() {
        let mut conn = setup_test_db();

        let result = get_source_optional(&mut conn, "NONEXISTENT").expect("Failed to query");
        assert!(result.is_none());

        let source = NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z");
        insert_source(&mut conn, &source).expect("Failed to insert");

        let result = get_source_optional(&mut conn, "PHB").expect("Failed to query");
        assert!(result.is_some());
    }

    #[test]
    fn test_source_exists() {
        let mut conn = setup_test_db();

        assert!(!source_exists(&mut conn, "PHB").expect("Failed to check"));

        let source = NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z");
        insert_source(&mut conn, &source).expect("Failed to insert");

        assert!(source_exists(&mut conn, "PHB").expect("Failed to check"));
    }

    #[test]
    fn test_delete_source() {
        let mut conn = setup_test_db();

        let source = NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z");
        insert_source(&mut conn, &source).expect("Failed to insert");

        assert!(source_exists(&mut conn, "PHB").expect("Failed to check"));

        delete_source(&mut conn, "PHB").expect("Failed to delete");

        assert!(!source_exists(&mut conn, "PHB").expect("Failed to check"));
    }

    #[test]
    fn test_count_sources() {
        let mut conn = setup_test_db();

        assert_eq!(count_sources(&mut conn).expect("Failed to count"), 0);

        let sources = vec![
            NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z"),
            NewCatalogSource::new("MM", "Monster Manual", true, "2024-01-20T12:00:00Z"),
            NewCatalogSource::new("DMG", "Dungeon Master's Guide", false, "2024-01-20T12:00:00Z"),
        ];
        insert_sources(&mut conn, &sources).expect("Failed to insert");

        assert_eq!(count_sources(&mut conn).expect("Failed to count"), 3);
        assert_eq!(count_enabled_sources(&mut conn).expect("Failed to count"), 2);
    }
}
