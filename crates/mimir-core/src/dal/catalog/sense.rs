//! Sense Data Access Layer
//!
//! Database operations for senses.

use crate::models::catalog::{NewSense, Sense};
use crate::schema::senses;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new sense.
pub fn insert_sense(conn: &mut SqliteConnection, sense: &NewSense) -> QueryResult<i32> {
    diesel::insert_into(senses::table)
        .values(sense)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple senses in a batch.
pub fn insert_senses(
    conn: &mut SqliteConnection,
    senses: &[NewSense],
) -> QueryResult<usize> {
    diesel::insert_into(senses::table)
        .values(senses)
        .execute(conn)
}

/// Get a sense by its ID.
pub fn get_sense(conn: &mut SqliteConnection, id: i32) -> QueryResult<Sense> {
    senses::table
        .filter(senses::id.eq(id))
        .first(conn)
}

/// Get a sense by name and source.
pub fn get_sense_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Sense>> {
    senses::table
        .filter(senses::name.eq(name))
        .filter(senses::source.eq(source))
        .first(conn)
        .optional()
}

/// List all senses, ordered by name.
pub fn list_senses(conn: &mut SqliteConnection) -> QueryResult<Vec<Sense>> {
    senses::table.order(senses::name.asc()).load(conn)
}

/// List senses from a specific source.
pub fn list_senses_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Sense>> {
    senses::table
        .filter(senses::source.eq(source))
        .order(senses::name.asc())
        .load(conn)
}

/// Delete a sense by its ID.
pub fn delete_sense(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(senses::table.filter(senses::id.eq(id))).execute(conn)
}

/// Delete all senses from a specific source.
pub fn delete_senses_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(senses::table.filter(senses::source.eq(source))).execute(conn)
}

/// Count all senses.
pub fn count_senses(conn: &mut SqliteConnection) -> QueryResult<i64> {
    senses::table.count().get_result(conn)
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
            CREATE TABLE senses (
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
    fn test_sense_crud() {
        let mut conn = setup_test_db();

        let sense = NewSense::new("Darkvision", "PHB", r#"{"name":"Darkvision"}"#);
        let id = insert_sense(&mut conn, &sense).expect("Failed to insert");

        let retrieved = get_sense(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Darkvision");

        let by_name = get_sense_by_name(&mut conn, "Darkvision", "PHB")
            .expect("Failed to query")
            .expect("Sense not found");
        assert_eq!(by_name.name, "Darkvision");

        delete_sense(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_senses(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_senses() {
        let mut conn = setup_test_db();

        let senses = vec![
            NewSense::new("Blindsight", "PHB", r#"{}"#),
            NewSense::new("Darkvision", "PHB", r#"{}"#),
            NewSense::new("Tremorsense", "PHB", r#"{}"#),
        ];
        insert_senses(&mut conn, &senses).expect("Failed to insert");

        let list = list_senses(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Blindsight"); // Alphabetical
    }
}
