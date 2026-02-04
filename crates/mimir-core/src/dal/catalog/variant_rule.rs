//! VariantRule Data Access Layer
//!
//! Database operations for variant rules.

use crate::models::catalog::{NewVariantRule, VariantRule, VariantRuleFilter};
use crate::schema::variant_rules;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new variant rule.
pub fn insert_variant_rule(
    conn: &mut SqliteConnection,
    rule: &NewVariantRule,
) -> QueryResult<i32> {
    diesel::insert_into(variant_rules::table)
        .values(rule)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple variant rules in a batch.
pub fn insert_variant_rules(
    conn: &mut SqliteConnection,
    rules: &[NewVariantRule],
) -> QueryResult<usize> {
    diesel::insert_into(variant_rules::table)
        .values(rules)
        .execute(conn)
}

/// Get a variant rule by its ID.
pub fn get_variant_rule(conn: &mut SqliteConnection, id: i32) -> QueryResult<VariantRule> {
    variant_rules::table
        .filter(variant_rules::id.eq(id))
        .first(conn)
}

/// Get a variant rule by name and source.
pub fn get_variant_rule_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<VariantRule>> {
    variant_rules::table
        .filter(variant_rules::name.eq(name))
        .filter(variant_rules::source.eq(source))
        .first(conn)
        .optional()
}

/// List all variant rules, ordered by name.
pub fn list_variant_rules(conn: &mut SqliteConnection) -> QueryResult<Vec<VariantRule>> {
    variant_rules::table
        .order(variant_rules::name.asc())
        .load(conn)
}

/// List variant rules from a specific source.
pub fn list_variant_rules_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<VariantRule>> {
    variant_rules::table
        .filter(variant_rules::source.eq(source))
        .order(variant_rules::name.asc())
        .load(conn)
}

/// Delete a variant rule by its ID.
pub fn delete_variant_rule(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(variant_rules::table.filter(variant_rules::id.eq(id))).execute(conn)
}

/// Delete all variant rules from a specific source.
pub fn delete_variant_rules_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(variant_rules::table.filter(variant_rules::source.eq(source))).execute(conn)
}

/// Count all variant rules.
pub fn count_variant_rules(conn: &mut SqliteConnection) -> QueryResult<i64> {
    variant_rules::table.count().get_result(conn)
}

/// Count variant rules from a specific source.
pub fn count_variant_rules_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    variant_rules::table
        .filter(variant_rules::source.eq(source))
        .count()
        .get_result(conn)
}

/// Get a variant rule by its ID, returning None if not found.
pub fn get_variant_rule_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<VariantRule>> {
    variant_rules::table
        .filter(variant_rules::id.eq(id))
        .first(conn)
        .optional()
}

/// List all distinct sources that have variant rules.
pub fn list_variant_rule_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    variant_rules::table
        .select(variant_rules::source)
        .distinct()
        .order(variant_rules::source.asc())
        .load(conn)
}

/// Search variant rules with filters.
pub fn search_variant_rules(
    conn: &mut SqliteConnection,
    filter: &VariantRuleFilter,
) -> QueryResult<Vec<VariantRule>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = variant_rules::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(variant_rules::name.like(pattern));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(variant_rules::source.eq_any(sources));
    }

    if let Some(ref rule_type) = filter.rule_type {
        query = query.filter(variant_rules::rule_type.eq(rule_type));
    }

    query.order(variant_rules::name.asc()).load(conn)
}

/// Search variant rules with pagination.
pub fn search_variant_rules_paginated(
    conn: &mut SqliteConnection,
    filter: &VariantRuleFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<VariantRule>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = variant_rules::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(variant_rules::name.like(pattern));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(variant_rules::source.eq_any(sources));
    }

    if let Some(ref rule_type) = filter.rule_type {
        query = query.filter(variant_rules::rule_type.eq(rule_type));
    }

    query
        .order(variant_rules::name.asc())
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
        let source = NewCatalogSource::new("DMG", "Dungeon Master's Guide", true, "2024-01-20T12:00:00Z");
        insert_source(conn, &source).expect("Failed to insert source");
    }

    #[test]
    fn test_variant_rule_crud() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let rule = NewVariantRule::new("Flanking", "DMG", r#"{"name":"Flanking"}"#)
            .with_type("O");
        let id = insert_variant_rule(&mut conn, &rule).expect("Failed to insert");

        let retrieved = get_variant_rule(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Flanking");
        assert_eq!(retrieved.rule_type, Some("O".to_string()));

        delete_variant_rule(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_variant_rules(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_variant_rules() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let rules = vec![
            NewVariantRule::new("Flanking", "DMG", r#"{}"#),
            NewVariantRule::new("Injuries", "DMG", r#"{}"#),
            NewVariantRule::new("Massive Damage", "DMG", r#"{}"#),
        ];
        insert_variant_rules(&mut conn, &rules).expect("Failed to insert");

        let list = list_variant_rules(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Flanking"); // Alphabetical
    }
}
