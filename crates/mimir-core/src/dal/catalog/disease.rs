//! Disease Data Access Layer
//!
//! Database operations for diseases.

use crate::models::catalog::{Disease, NewDisease};
use crate::schema::diseases;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new disease.
pub fn insert_disease(conn: &mut SqliteConnection, disease: &NewDisease) -> QueryResult<i32> {
    diesel::insert_into(diseases::table)
        .values(disease)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple diseases in a batch.
pub fn insert_diseases(
    conn: &mut SqliteConnection,
    diseases: &[NewDisease],
) -> QueryResult<usize> {
    diesel::insert_into(diseases::table)
        .values(diseases)
        .execute(conn)
}

/// Get a disease by its ID.
pub fn get_disease(conn: &mut SqliteConnection, id: i32) -> QueryResult<Disease> {
    diseases::table
        .filter(diseases::id.eq(id))
        .first(conn)
}

/// Get a disease by name and source.
pub fn get_disease_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Disease>> {
    diseases::table
        .filter(diseases::name.eq(name))
        .filter(diseases::source.eq(source))
        .first(conn)
        .optional()
}

/// List all diseases, ordered by name.
pub fn list_diseases(conn: &mut SqliteConnection) -> QueryResult<Vec<Disease>> {
    diseases::table.order(diseases::name.asc()).load(conn)
}

/// List diseases from a specific source.
pub fn list_diseases_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Disease>> {
    diseases::table
        .filter(diseases::source.eq(source))
        .order(diseases::name.asc())
        .load(conn)
}

/// Delete a disease by its ID.
pub fn delete_disease(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(diseases::table.filter(diseases::id.eq(id))).execute(conn)
}

/// Delete all diseases from a specific source.
pub fn delete_diseases_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(diseases::table.filter(diseases::source.eq(source))).execute(conn)
}

/// Count all diseases.
pub fn count_diseases(conn: &mut SqliteConnection) -> QueryResult<i64> {
    diseases::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_disease_crud() {
        let mut conn = setup_test_db_with_sources();

        let disease = NewDisease::new("Cackle Fever", "DMG", r#"{"name":"Cackle Fever"}"#);
        let id = insert_disease(&mut conn, &disease).expect("Failed to insert");

        let retrieved = get_disease(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Cackle Fever");

        let by_name = get_disease_by_name(&mut conn, "Cackle Fever", "DMG")
            .expect("Failed to query")
            .expect("Disease not found");
        assert_eq!(by_name.name, "Cackle Fever");

        delete_disease(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_diseases(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_diseases() {
        let mut conn = setup_test_db_with_sources();

        let diseases = vec![
            NewDisease::new("Cackle Fever", "DMG", r#"{}"#),
            NewDisease::new("Sewer Plague", "DMG", r#"{}"#),
            NewDisease::new("Sight Rot", "DMG", r#"{}"#),
        ];
        insert_diseases(&mut conn, &diseases).expect("Failed to insert");

        let list = list_diseases(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Cackle Fever"); // Alphabetical
    }
}
