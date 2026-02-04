//! Psionic Data Access Layer
//!
//! Database operations for psionics.

use crate::models::catalog::{NewPsionic, Psionic, PsionicFilter};
use crate::schema::psionics;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new psionic.
pub fn insert_psionic(conn: &mut SqliteConnection, psionic: &NewPsionic) -> QueryResult<i32> {
    diesel::insert_into(psionics::table)
        .values(psionic)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple psionics in a batch.
pub fn insert_psionics(conn: &mut SqliteConnection, psionics: &[NewPsionic]) -> QueryResult<usize> {
    diesel::insert_into(psionics::table)
        .values(psionics)
        .execute(conn)
}

/// Get a psionic by its ID.
pub fn get_psionic(conn: &mut SqliteConnection, id: i32) -> QueryResult<Psionic> {
    psionics::table.filter(psionics::id.eq(id)).first(conn)
}

/// Get a psionic by name and source.
pub fn get_psionic_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Psionic>> {
    psionics::table
        .filter(psionics::name.eq(name))
        .filter(psionics::source.eq(source))
        .first(conn)
        .optional()
}

/// List all psionics, ordered by name.
pub fn list_psionics(conn: &mut SqliteConnection) -> QueryResult<Vec<Psionic>> {
    psionics::table.order(psionics::name.asc()).load(conn)
}

/// List psionics from a specific source.
pub fn list_psionics_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Psionic>> {
    psionics::table
        .filter(psionics::source.eq(source))
        .order(psionics::name.asc())
        .load(conn)
}

/// List psionics by type (e.g., "D" for Discipline, "T" for Talent).
pub fn list_psionics_by_type(
    conn: &mut SqliteConnection,
    psionic_type: &str,
) -> QueryResult<Vec<Psionic>> {
    psionics::table
        .filter(psionics::psionic_type.eq(psionic_type))
        .order(psionics::name.asc())
        .load(conn)
}

/// List psionics by order (e.g., "Avatar", "Wu Jen").
pub fn list_psionics_by_order(
    conn: &mut SqliteConnection,
    order: &str,
) -> QueryResult<Vec<Psionic>> {
    psionics::table
        .filter(psionics::psionic_order.eq(order))
        .order(psionics::name.asc())
        .load(conn)
}

/// Delete a psionic by its ID.
pub fn delete_psionic(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(psionics::table.filter(psionics::id.eq(id))).execute(conn)
}

/// Delete all psionics from a specific source.
pub fn delete_psionics_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(psionics::table.filter(psionics::source.eq(source))).execute(conn)
}

/// Count all psionics.
pub fn count_psionics(conn: &mut SqliteConnection) -> QueryResult<i64> {
    psionics::table.count().get_result(conn)
}

/// Count psionics from a specific source.
pub fn count_psionics_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    psionics::table
        .filter(psionics::source.eq(source))
        .count()
        .get_result(conn)
}

/// Get a psionic by its ID, returning None if not found.
pub fn get_psionic_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Psionic>> {
    psionics::table
        .filter(psionics::id.eq(id))
        .first(conn)
        .optional()
}

/// List all distinct sources that have psionics.
pub fn list_psionic_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    psionics::table
        .select(psionics::source)
        .distinct()
        .order(psionics::source.asc())
        .load(conn)
}

/// Search psionics with filters.
pub fn search_psionics(
    conn: &mut SqliteConnection,
    filter: &PsionicFilter,
) -> QueryResult<Vec<Psionic>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = psionics::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(psionics::name.like(pattern));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(psionics::source.eq_any(sources));
    }

    if let Some(ref psionic_type) = filter.psionic_type {
        query = query.filter(psionics::psionic_type.eq(psionic_type));
    }

    if let Some(ref psionic_order) = filter.psionic_order {
        query = query.filter(psionics::psionic_order.eq(psionic_order));
    }

    query.order(psionics::name.asc()).load(conn)
}

/// Search psionics with pagination.
pub fn search_psionics_paginated(
    conn: &mut SqliteConnection,
    filter: &PsionicFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Psionic>> {
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = psionics::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(psionics::name.like(pattern));
    }

    if let Some(sources) = filter.effective_sources() {
        query = query.filter(psionics::source.eq_any(sources));
    }

    if let Some(ref psionic_type) = filter.psionic_type {
        query = query.filter(psionics::psionic_type.eq(psionic_type));
    }

    if let Some(ref psionic_order) = filter.psionic_order {
        query = query.filter(psionics::psionic_order.eq(psionic_order));
    }

    query
        .order(psionics::name.asc())
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
        let source = NewCatalogSource::new("UAMystic", "UA: Mystic", true, "2024-01-20T12:00:00Z");
        insert_source(conn, &source).expect("Failed to insert source");
    }

    #[test]
    fn test_psionic_crud() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let psionic = NewPsionic::new("Mastery of Force", "UAMystic", r#"{"name":"Mastery of Force"}"#)
            .with_type("D")
            .with_order("Wu Jen");
        let id = insert_psionic(&mut conn, &psionic).expect("Failed to insert");

        let retrieved = get_psionic(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Mastery of Force");
        assert_eq!(retrieved.psionic_type, Some("D".to_string()));
        assert_eq!(retrieved.psionic_order, Some("Wu Jen".to_string()));

        delete_psionic(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_psionics(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_by_type_and_order() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let psionics_data = vec![
            NewPsionic::new("Psychic Focus", "UAMystic", r#"{}"#).with_type("T"),
            NewPsionic::new("Mastery of Force", "UAMystic", r#"{}"#)
                .with_type("D")
                .with_order("Wu Jen"),
            NewPsionic::new("Mastery of Ice", "UAMystic", r#"{}"#)
                .with_type("D")
                .with_order("Wu Jen"),
        ];
        insert_psionics(&mut conn, &psionics_data).expect("Failed to insert");

        let disciplines = list_psionics_by_type(&mut conn, "D").expect("Failed to list");
        assert_eq!(disciplines.len(), 2);

        let talents = list_psionics_by_type(&mut conn, "T").expect("Failed to list");
        assert_eq!(talents.len(), 1);

        let wu_jen = list_psionics_by_order(&mut conn, "Wu Jen").expect("Failed to list");
        assert_eq!(wu_jen.len(), 2);
    }
}
