//! Subclass Feature Data Access Layer
//!
//! Database operations for subclass features.

use crate::models::catalog::{SubclassFeature, SubclassFeatureFilter, NewSubclassFeature};
use crate::schema::subclass_features;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new subclass feature, ignoring duplicates.
pub fn insert_subclass_feature(conn: &mut SqliteConnection, feature: &NewSubclassFeature) -> QueryResult<i32> {
    diesel::insert_or_ignore_into(subclass_features::table)
        .values(feature)
        .execute(conn)?;

    subclass_features::table
        .filter(subclass_features::name.eq(&feature.name))
        .filter(subclass_features::source.eq(&feature.source))
        .filter(subclass_features::class_name.eq(&feature.class_name))
        .filter(subclass_features::class_source.eq(&feature.class_source))
        .filter(subclass_features::subclass_name.eq(&feature.subclass_name))
        .filter(subclass_features::subclass_source.eq(&feature.subclass_source))
        .select(subclass_features::id)
        .first::<Option<i32>>(conn)?
        .ok_or(diesel::result::Error::NotFound)
}

/// Insert multiple subclass features in a batch.
pub fn insert_subclass_features(conn: &mut SqliteConnection, features: &[NewSubclassFeature]) -> QueryResult<usize> {
    diesel::insert_or_ignore_into(subclass_features::table)
        .values(features)
        .execute(conn)
}

/// Get a subclass feature by its ID.
pub fn get_subclass_feature(conn: &mut SqliteConnection, id: i32) -> QueryResult<SubclassFeature> {
    subclass_features::table.filter(subclass_features::id.eq(id)).first(conn)
}

// Define the LOWER SQL function for case-insensitive matching
diesel::define_sql_function!(fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text);

/// Get a subclass feature by name and subclass (case-insensitive name matching).
pub fn get_subclass_feature_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    subclass_name: &str,
    subclass_source: &str,
) -> QueryResult<Option<SubclassFeature>> {
    let name_lower = name.to_lowercase();
    let subclass_name_lower = subclass_name.to_lowercase();
    subclass_features::table
        .filter(lower(subclass_features::name).eq(&name_lower))
        .filter(lower(subclass_features::subclass_name).eq(&subclass_name_lower))
        .filter(subclass_features::subclass_source.eq(subclass_source))
        .first(conn)
        .optional()
}

/// List all subclass features for a specific subclass.
pub fn list_subclass_features_by_subclass(
    conn: &mut SqliteConnection,
    subclass_name: &str,
    subclass_source: &str,
) -> QueryResult<Vec<SubclassFeature>> {
    subclass_features::table
        .filter(subclass_features::subclass_name.eq(subclass_name))
        .filter(subclass_features::subclass_source.eq(subclass_source))
        .order((subclass_features::level.asc(), subclass_features::name.asc()))
        .load(conn)
}

/// Delete all subclass features from a specific source.
pub fn delete_subclass_features_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<usize> {
    diesel::delete(subclass_features::table.filter(subclass_features::source.eq(source))).execute(conn)
}

/// Count all subclass features.
pub fn count_subclass_features(conn: &mut SqliteConnection) -> QueryResult<i64> {
    subclass_features::table.count().get_result(conn)
}

/// Count subclass features from a specific source.
pub fn count_subclass_features_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    subclass_features::table
        .filter(subclass_features::source.eq(source))
        .count()
        .get_result(conn)
}

/// List all distinct sources that have subclass features.
pub fn list_subclass_feature_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    subclass_features::table
        .select(subclass_features::source)
        .distinct()
        .order(subclass_features::source.asc())
        .load(conn)
}

/// Search subclass features with filters.
pub fn search_subclass_features(
    conn: &mut SqliteConnection,
    filter: &SubclassFeatureFilter,
) -> QueryResult<Vec<SubclassFeature>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = subclass_features::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(subclass_features::name.like(pattern));
    }

    if let Some(ref class_name) = filter.class_name {
        query = query.filter(subclass_features::class_name.eq(class_name));
    }

    if let Some(ref class_source) = filter.class_source {
        query = query.filter(subclass_features::class_source.eq(class_source));
    }

    if let Some(ref subclass_name) = filter.subclass_name {
        query = query.filter(subclass_features::subclass_name.eq(subclass_name));
    }

    if let Some(ref subclass_source) = filter.subclass_source {
        query = query.filter(subclass_features::subclass_source.eq(subclass_source));
    }

    if let Some(level) = filter.level {
        query = query.filter(subclass_features::level.eq(level));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(subclass_features::source.eq_any(sources));
    }

    query
        .order((subclass_features::subclass_name.asc(), subclass_features::level.asc(), subclass_features::name.asc()))
        .load(conn)
}
