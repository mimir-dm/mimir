//! OptionalFeature Data Access Layer
//!
//! Database operations for optional features.

use crate::models::catalog::{NewOptionalFeature, OptionalFeature};
use crate::schema::optional_features;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new optional feature.
pub fn insert_optional_feature(
    conn: &mut SqliteConnection,
    feature: &NewOptionalFeature,
) -> QueryResult<i32> {
    diesel::insert_into(optional_features::table)
        .values(feature)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple optional features in a batch.
pub fn insert_optional_features(
    conn: &mut SqliteConnection,
    features: &[NewOptionalFeature],
) -> QueryResult<usize> {
    diesel::insert_into(optional_features::table)
        .values(features)
        .execute(conn)
}

/// Get an optional feature by its ID.
pub fn get_optional_feature(conn: &mut SqliteConnection, id: i32) -> QueryResult<OptionalFeature> {
    optional_features::table
        .filter(optional_features::id.eq(id))
        .first(conn)
}

/// Get an optional feature by name and source.
pub fn get_optional_feature_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<OptionalFeature>> {
    optional_features::table
        .filter(optional_features::name.eq(name))
        .filter(optional_features::source.eq(source))
        .first(conn)
        .optional()
}

/// List all optional features, ordered by name.
pub fn list_optional_features(conn: &mut SqliteConnection) -> QueryResult<Vec<OptionalFeature>> {
    optional_features::table
        .order(optional_features::name.asc())
        .load(conn)
}

/// List optional features from a specific source.
pub fn list_optional_features_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<OptionalFeature>> {
    optional_features::table
        .filter(optional_features::source.eq(source))
        .order(optional_features::name.asc())
        .load(conn)
}

/// List optional features by type (e.g., "EI" for Eldritch Invocations).
pub fn list_optional_features_by_type(
    conn: &mut SqliteConnection,
    feature_type: &str,
) -> QueryResult<Vec<OptionalFeature>> {
    optional_features::table
        .filter(optional_features::feature_type.eq(feature_type))
        .order(optional_features::name.asc())
        .load(conn)
}

/// Delete an optional feature by its ID.
pub fn delete_optional_feature(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(optional_features::table.filter(optional_features::id.eq(id))).execute(conn)
}

/// Delete all optional features from a specific source.
pub fn delete_optional_features_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(optional_features::table.filter(optional_features::source.eq(source)))
        .execute(conn)
}

/// Count all optional features.
pub fn count_optional_features(conn: &mut SqliteConnection) -> QueryResult<i64> {
    optional_features::table.count().get_result(conn)
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
            CREATE TABLE optional_features (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                source TEXT NOT NULL REFERENCES catalog_sources(code),
                feature_type TEXT,
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
    fn test_optional_feature_crud() {
        let mut conn = setup_test_db();

        let feature = NewOptionalFeature::new("Agonizing Blast", "PHB", r#"{"name":"Agonizing Blast"}"#)
            .with_feature_type("EI");
        let id = insert_optional_feature(&mut conn, &feature).expect("Failed to insert");

        let retrieved = get_optional_feature(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Agonizing Blast");
        assert_eq!(retrieved.feature_type, Some("EI".to_string()));

        delete_optional_feature(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_optional_features(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_by_type() {
        let mut conn = setup_test_db();

        let features = vec![
            NewOptionalFeature::new("Agonizing Blast", "PHB", r#"{}"#).with_feature_type("EI"),
            NewOptionalFeature::new("Eldritch Spear", "PHB", r#"{}"#).with_feature_type("EI"),
            NewOptionalFeature::new("Quickened Spell", "PHB", r#"{}"#).with_feature_type("MM"),
        ];
        insert_optional_features(&mut conn, &features).expect("Failed to insert");

        let invocations = list_optional_features_by_type(&mut conn, "EI").expect("Failed to list");
        assert_eq!(invocations.len(), 2);

        let metamagic = list_optional_features_by_type(&mut conn, "MM").expect("Failed to list");
        assert_eq!(metamagic.len(), 1);
    }
}
