//! Feat Data Access Layer
//!
//! Database operations for character feats.

use crate::models::catalog::{Feat, NewFeat};
use crate::schema::feats;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new feat.
pub fn insert_feat(conn: &mut SqliteConnection, feat: &NewFeat) -> QueryResult<i32> {
    diesel::insert_into(feats::table)
        .values(feat)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple feats in a batch.
pub fn insert_feats(conn: &mut SqliteConnection, feats: &[NewFeat]) -> QueryResult<usize> {
    diesel::insert_into(feats::table)
        .values(feats)
        .execute(conn)
}

/// Get a feat by its ID.
pub fn get_feat(conn: &mut SqliteConnection, id: i32) -> QueryResult<Feat> {
    feats::table.filter(feats::id.eq(id)).first(conn)
}

/// Get a feat by name and source.
pub fn get_feat_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Feat>> {
    feats::table
        .filter(feats::name.eq(name))
        .filter(feats::source.eq(source))
        .first(conn)
        .optional()
}

/// List all feats, ordered by name.
pub fn list_feats(conn: &mut SqliteConnection) -> QueryResult<Vec<Feat>> {
    feats::table.order(feats::name.asc()).load(conn)
}

/// List feats from a specific source.
pub fn list_feats_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<Vec<Feat>> {
    feats::table
        .filter(feats::source.eq(source))
        .order(feats::name.asc())
        .load(conn)
}

/// Search feats by name pattern.
pub fn search_feats_by_name(conn: &mut SqliteConnection, pattern: &str) -> QueryResult<Vec<Feat>> {
    let pattern = format!("%{}%", pattern);
    feats::table
        .filter(feats::name.like(pattern))
        .order(feats::name.asc())
        .load(conn)
}

/// Delete a feat by its ID.
pub fn delete_feat(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(feats::table.filter(feats::id.eq(id))).execute(conn)
}

/// Delete all feats from a specific source.
pub fn delete_feats_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<usize> {
    diesel::delete(feats::table.filter(feats::source.eq(source))).execute(conn)
}

/// Count all feats.
pub fn count_feats(conn: &mut SqliteConnection) -> QueryResult<i64> {
    feats::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_feat_crud() {
        let mut conn = setup_test_db_with_sources();

        let feat = NewFeat::new("Alert", "PHB", r#"{"name":"Alert"}"#);
        let id = insert_feat(&mut conn, &feat).expect("Failed to insert");

        let retrieved = get_feat(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Alert");

        let by_name = get_feat_by_name(&mut conn, "Alert", "PHB")
            .expect("Failed to query")
            .expect("Feat not found");
        assert_eq!(by_name.name, "Alert");

        delete_feat(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_feats(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_search_feats() {
        let mut conn = setup_test_db_with_sources();

        let feats = vec![
            NewFeat::new("Alert", "PHB", r#"{}"#),
            NewFeat::new("Sharpshooter", "PHB", r#"{}"#),
            NewFeat::new("Great Weapon Master", "PHB", r#"{}"#),
        ];
        insert_feats(&mut conn, &feats).expect("Failed to insert");

        let results = search_feats_by_name(&mut conn, "sharp").expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Sharpshooter");
    }
}
