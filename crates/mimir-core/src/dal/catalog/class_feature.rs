//! Class Feature Data Access Layer
//!
//! Database operations for class features.

use crate::models::catalog::{ClassFeature, ClassFeatureFilter, NewClassFeature};
use crate::schema::class_features;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new class feature, ignoring duplicates.
pub fn insert_class_feature(conn: &mut SqliteConnection, feature: &NewClassFeature) -> QueryResult<i32> {
    diesel::insert_or_ignore_into(class_features::table)
        .values(feature)
        .execute(conn)?;

    class_features::table
        .filter(class_features::name.eq(&feature.name))
        .filter(class_features::source.eq(&feature.source))
        .filter(class_features::class_name.eq(&feature.class_name))
        .filter(class_features::class_source.eq(&feature.class_source))
        .select(class_features::id)
        .first::<Option<i32>>(conn)?
        .ok_or(diesel::result::Error::NotFound)
}

/// Insert multiple class features in a batch.
pub fn insert_class_features(conn: &mut SqliteConnection, features: &[NewClassFeature]) -> QueryResult<usize> {
    diesel::insert_or_ignore_into(class_features::table)
        .values(features)
        .execute(conn)
}

/// Get a class feature by its ID.
pub fn get_class_feature(conn: &mut SqliteConnection, id: i32) -> QueryResult<ClassFeature> {
    class_features::table.filter(class_features::id.eq(id)).first(conn)
}

// Define the LOWER SQL function for case-insensitive matching
diesel::define_sql_function!(fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text);

/// Get a class feature by name, source, and class (case-insensitive name matching).
pub fn get_class_feature_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
    class_name: &str,
    class_source: &str,
) -> QueryResult<Option<ClassFeature>> {
    let name_lower = name.to_lowercase();
    let class_name_lower = class_name.to_lowercase();
    class_features::table
        .filter(lower(class_features::name).eq(&name_lower))
        .filter(class_features::source.eq(source))
        .filter(lower(class_features::class_name).eq(&class_name_lower))
        .filter(class_features::class_source.eq(class_source))
        .first(conn)
        .optional()
}

/// Get a class feature by name and class (any source, case-insensitive name matching).
pub fn get_class_feature_by_name_and_class(
    conn: &mut SqliteConnection,
    name: &str,
    class_name: &str,
) -> QueryResult<Option<ClassFeature>> {
    let name_lower = name.to_lowercase();
    let class_name_lower = class_name.to_lowercase();
    class_features::table
        .filter(lower(class_features::name).eq(&name_lower))
        .filter(lower(class_features::class_name).eq(&class_name_lower))
        .first(conn)
        .optional()
}

/// List all class features for a specific class.
pub fn list_class_features_by_class(
    conn: &mut SqliteConnection,
    class_name: &str,
    class_source: &str,
) -> QueryResult<Vec<ClassFeature>> {
    class_features::table
        .filter(class_features::class_name.eq(class_name))
        .filter(class_features::class_source.eq(class_source))
        .order((class_features::level.asc(), class_features::name.asc()))
        .load(conn)
}

/// Delete all class features from a specific source.
pub fn delete_class_features_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<usize> {
    diesel::delete(class_features::table.filter(class_features::source.eq(source))).execute(conn)
}

/// Count all class features.
pub fn count_class_features(conn: &mut SqliteConnection) -> QueryResult<i64> {
    class_features::table.count().get_result(conn)
}

/// Count class features from a specific source.
pub fn count_class_features_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    class_features::table
        .filter(class_features::source.eq(source))
        .count()
        .get_result(conn)
}

/// List all distinct sources that have class features.
pub fn list_class_feature_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    class_features::table
        .select(class_features::source)
        .distinct()
        .order(class_features::source.asc())
        .load(conn)
}

/// Search class features with filters.
pub fn search_class_features(
    conn: &mut SqliteConnection,
    filter: &ClassFeatureFilter,
) -> QueryResult<Vec<ClassFeature>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = class_features::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(class_features::name.like(pattern));
    }

    if let Some(ref class_name) = filter.class_name {
        query = query.filter(class_features::class_name.eq(class_name));
    }

    if let Some(ref class_source) = filter.class_source {
        query = query.filter(class_features::class_source.eq(class_source));
    }

    if let Some(level) = filter.level {
        query = query.filter(class_features::level.eq(level));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(class_features::source.eq_any(sources));
    }

    query
        .order((class_features::class_name.asc(), class_features::level.asc(), class_features::name.asc()))
        .load(conn)
}
