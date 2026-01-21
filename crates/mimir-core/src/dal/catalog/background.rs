//! Background Data Access Layer
//!
//! Database operations for character backgrounds.

use crate::models::catalog::{Background, NewBackground};
use crate::schema::backgrounds;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new background.
pub fn insert_background(conn: &mut SqliteConnection, background: &NewBackground) -> QueryResult<i32> {
    diesel::insert_into(backgrounds::table)
        .values(background)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple backgrounds in a batch.
pub fn insert_backgrounds(
    conn: &mut SqliteConnection,
    backgrounds: &[NewBackground],
) -> QueryResult<usize> {
    diesel::insert_into(backgrounds::table)
        .values(backgrounds)
        .execute(conn)
}

/// Get a background by its ID.
pub fn get_background(conn: &mut SqliteConnection, id: i32) -> QueryResult<Background> {
    backgrounds::table
        .filter(backgrounds::id.eq(id))
        .first(conn)
}

/// Get a background by name and source.
pub fn get_background_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Background>> {
    backgrounds::table
        .filter(backgrounds::name.eq(name))
        .filter(backgrounds::source.eq(source))
        .first(conn)
        .optional()
}

/// List all backgrounds, ordered by name.
pub fn list_backgrounds(conn: &mut SqliteConnection) -> QueryResult<Vec<Background>> {
    backgrounds::table.order(backgrounds::name.asc()).load(conn)
}

/// List backgrounds from a specific source.
pub fn list_backgrounds_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Background>> {
    backgrounds::table
        .filter(backgrounds::source.eq(source))
        .order(backgrounds::name.asc())
        .load(conn)
}

/// Delete a background by its ID.
pub fn delete_background(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(backgrounds::table.filter(backgrounds::id.eq(id))).execute(conn)
}

/// Delete all backgrounds from a specific source.
pub fn delete_backgrounds_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(backgrounds::table.filter(backgrounds::source.eq(source))).execute(conn)
}

/// Count all backgrounds.
pub fn count_backgrounds(conn: &mut SqliteConnection) -> QueryResult<i64> {
    backgrounds::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_background_crud() {
        let mut conn = setup_test_db_with_sources();

        let background = NewBackground::new("Acolyte", "PHB", r#"{"name":"Acolyte"}"#);
        let id = insert_background(&mut conn, &background).expect("Failed to insert");

        let retrieved = get_background(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Acolyte");

        let by_name = get_background_by_name(&mut conn, "Acolyte", "PHB")
            .expect("Failed to query")
            .expect("Background not found");
        assert_eq!(by_name.name, "Acolyte");

        delete_background(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_backgrounds(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_backgrounds() {
        let mut conn = setup_test_db_with_sources();

        let backgrounds = vec![
            NewBackground::new("Acolyte", "PHB", r#"{}"#),
            NewBackground::new("Criminal", "PHB", r#"{}"#),
            NewBackground::new("Noble", "PHB", r#"{}"#),
        ];
        insert_backgrounds(&mut conn, &backgrounds).expect("Failed to insert");

        let list = list_backgrounds(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Acolyte"); // Alphabetical
    }
}
