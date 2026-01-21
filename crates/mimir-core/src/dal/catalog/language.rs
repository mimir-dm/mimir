//! Language Data Access Layer
//!
//! Database operations for languages.

use crate::models::catalog::{Language, NewLanguage};
use crate::schema::languages;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new language.
pub fn insert_language(conn: &mut SqliteConnection, language: &NewLanguage) -> QueryResult<i32> {
    diesel::insert_into(languages::table)
        .values(language)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple languages in a batch.
pub fn insert_languages(
    conn: &mut SqliteConnection,
    languages: &[NewLanguage],
) -> QueryResult<usize> {
    diesel::insert_into(languages::table)
        .values(languages)
        .execute(conn)
}

/// Get a language by its ID.
pub fn get_language(conn: &mut SqliteConnection, id: i32) -> QueryResult<Language> {
    languages::table
        .filter(languages::id.eq(id))
        .first(conn)
}

/// Get a language by name and source.
pub fn get_language_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Language>> {
    languages::table
        .filter(languages::name.eq(name))
        .filter(languages::source.eq(source))
        .first(conn)
        .optional()
}

/// List all languages, ordered by name.
pub fn list_languages(conn: &mut SqliteConnection) -> QueryResult<Vec<Language>> {
    languages::table.order(languages::name.asc()).load(conn)
}

/// List languages from a specific source.
pub fn list_languages_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Language>> {
    languages::table
        .filter(languages::source.eq(source))
        .order(languages::name.asc())
        .load(conn)
}

/// List languages by type (e.g., "Standard", "Exotic").
pub fn list_languages_by_type(
    conn: &mut SqliteConnection,
    language_type: &str,
) -> QueryResult<Vec<Language>> {
    languages::table
        .filter(languages::language_type.eq(language_type))
        .order(languages::name.asc())
        .load(conn)
}

/// Delete a language by its ID.
pub fn delete_language(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(languages::table.filter(languages::id.eq(id))).execute(conn)
}

/// Delete all languages from a specific source.
pub fn delete_languages_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(languages::table.filter(languages::source.eq(source))).execute(conn)
}

/// Count all languages.
pub fn count_languages(conn: &mut SqliteConnection) -> QueryResult<i64> {
    languages::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_language_crud() {
        let mut conn = setup_test_db_with_sources();

        let language = NewLanguage::new("Common", "PHB", r#"{"name":"Common"}"#)
            .with_type("Standard");
        let id = insert_language(&mut conn, &language).expect("Failed to insert");

        let retrieved = get_language(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Common");
        assert_eq!(retrieved.language_type, Some("Standard".to_string()));

        let by_name = get_language_by_name(&mut conn, "Common", "PHB")
            .expect("Failed to query")
            .expect("Language not found");
        assert_eq!(by_name.name, "Common");

        delete_language(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_languages(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_languages() {
        let mut conn = setup_test_db_with_sources();

        let languages = vec![
            NewLanguage::new("Common", "PHB", r#"{}"#).with_type("Standard"),
            NewLanguage::new("Dwarvish", "PHB", r#"{}"#).with_type("Standard"),
            NewLanguage::new("Undercommon", "PHB", r#"{}"#).with_type("Exotic"),
        ];
        insert_languages(&mut conn, &languages).expect("Failed to insert");

        let list = list_languages(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Common"); // Alphabetical

        let standard = list_languages_by_type(&mut conn, "Standard").expect("Failed to list");
        assert_eq!(standard.len(), 2);
    }
}
