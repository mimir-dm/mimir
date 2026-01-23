//! Subclass Data Access Layer
//!
//! Database operations for character subclasses.

use crate::models::catalog::{NewSubclass, Subclass};
use crate::schema::subclasses;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new subclass, ignoring duplicates.
///
/// If a subclass with the same (name, class_name, source) already exists,
/// returns the existing ID without error.
pub fn insert_subclass(conn: &mut SqliteConnection, subclass: &NewSubclass) -> QueryResult<i32> {
    // Try to insert, ignoring conflicts
    diesel::insert_or_ignore_into(subclasses::table)
        .values(subclass)
        .execute(conn)?;

    // Look up the ID (either newly inserted or existing)
    subclasses::table
        .filter(subclasses::name.eq(&subclass.name))
        .filter(subclasses::class_name.eq(&subclass.class_name))
        .filter(subclasses::source.eq(&subclass.source))
        .select(subclasses::id)
        .first::<Option<i32>>(conn)?
        .ok_or(diesel::result::Error::NotFound)
}

/// Insert multiple subclasses in a batch.
pub fn insert_subclasses(
    conn: &mut SqliteConnection,
    subclasses: &[NewSubclass],
) -> QueryResult<usize> {
    diesel::insert_into(subclasses::table)
        .values(subclasses)
        .execute(conn)
}

/// Get a subclass by its ID.
pub fn get_subclass(conn: &mut SqliteConnection, id: i32) -> QueryResult<Subclass> {
    subclasses::table.filter(subclasses::id.eq(id)).first(conn)
}

// Define the LOWER SQL function for case-insensitive matching
diesel::define_sql_function!(fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text);

/// Get a subclass by name, class, and source (case-insensitive name matching).
pub fn get_subclass_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    class_name: &str,
    source: &str,
) -> QueryResult<Option<Subclass>> {
    let name_lower = name.to_lowercase();
    let class_name_lower = class_name.to_lowercase();
    subclasses::table
        .filter(lower(subclasses::name).eq(&name_lower))
        .filter(lower(subclasses::class_name).eq(&class_name_lower))
        .filter(subclasses::source.eq(source))
        .first(conn)
        .optional()
}

/// List all subclasses, ordered by class then name.
pub fn list_subclasses(conn: &mut SqliteConnection) -> QueryResult<Vec<Subclass>> {
    subclasses::table
        .order((subclasses::class_name.asc(), subclasses::name.asc()))
        .load(conn)
}

/// List subclasses for a specific class.
pub fn list_subclasses_by_class(
    conn: &mut SqliteConnection,
    class_name: &str,
) -> QueryResult<Vec<Subclass>> {
    subclasses::table
        .filter(subclasses::class_name.eq(class_name))
        .order(subclasses::name.asc())
        .load(conn)
}

/// List subclasses from a specific source.
pub fn list_subclasses_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Subclass>> {
    subclasses::table
        .filter(subclasses::source.eq(source))
        .order((subclasses::class_name.asc(), subclasses::name.asc()))
        .load(conn)
}

/// Delete a subclass by its ID.
pub fn delete_subclass(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(subclasses::table.filter(subclasses::id.eq(id))).execute(conn)
}

/// Delete all subclasses from a specific source.
pub fn delete_subclasses_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(subclasses::table.filter(subclasses::source.eq(source))).execute(conn)
}

/// Count all subclasses.
pub fn count_subclasses(conn: &mut SqliteConnection) -> QueryResult<i64> {
    subclasses::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_subclass_crud() {
        let mut conn = setup_test_db_with_sources();

        let subclass =
            NewSubclass::new("School of Evocation", "Wizard", "PHB", r#"{"name":"School of Evocation"}"#);
        let id = insert_subclass(&mut conn, &subclass).expect("Failed to insert");

        let retrieved = get_subclass(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "School of Evocation");
        assert_eq!(retrieved.class_name, "Wizard");

        let by_name = get_subclass_by_name(&mut conn, "School of Evocation", "Wizard", "PHB")
            .expect("Failed to query")
            .expect("Subclass not found");
        assert_eq!(by_name.name, "School of Evocation");

        delete_subclass(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_subclasses(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_subclasses_by_class() {
        let mut conn = setup_test_db_with_sources();

        let subclasses = vec![
            NewSubclass::new("School of Evocation", "Wizard", "PHB", r#"{}"#),
            NewSubclass::new("School of Abjuration", "Wizard", "PHB", r#"{}"#),
            NewSubclass::new("Champion", "Fighter", "PHB", r#"{}"#),
        ];
        insert_subclasses(&mut conn, &subclasses).expect("Failed to insert");

        let wizard_subclasses = list_subclasses_by_class(&mut conn, "Wizard").expect("Failed to list");
        assert_eq!(wizard_subclasses.len(), 2);

        let fighter_subclasses = list_subclasses_by_class(&mut conn, "Fighter").expect("Failed to list");
        assert_eq!(fighter_subclasses.len(), 1);
    }
}
