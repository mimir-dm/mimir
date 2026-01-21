//! Module Data Access Layer
//!
//! Database operations for modules (adventure chapters).

use crate::models::campaign::{Module, NewModule, UpdateModule};
use crate::schema::modules;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new module.
pub fn insert_module(conn: &mut SqliteConnection, module: &NewModule) -> QueryResult<String> {
    diesel::insert_into(modules::table)
        .values(module)
        .execute(conn)?;

    Ok(module.id.to_string())
}

/// Get a module by ID.
pub fn get_module(conn: &mut SqliteConnection, id: &str) -> QueryResult<Module> {
    modules::table.find(id).first(conn)
}

/// Get a module by ID, returning None if not found.
pub fn get_module_optional(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Module>> {
    modules::table.find(id).first(conn).optional()
}

/// Get a module by campaign ID and module number.
pub fn get_module_by_number(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    module_number: i32,
) -> QueryResult<Option<Module>> {
    modules::table
        .filter(modules::campaign_id.eq(campaign_id))
        .filter(modules::module_number.eq(module_number))
        .first(conn)
        .optional()
}

/// List all modules for a campaign, ordered by module_number.
pub fn list_modules(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<Vec<Module>> {
    modules::table
        .filter(modules::campaign_id.eq(campaign_id))
        .order(modules::module_number.asc())
        .load(conn)
}

/// Update a module.
pub fn update_module(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateModule,
) -> QueryResult<usize> {
    diesel::update(modules::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a module by ID.
///
/// Note: This will cascade delete all related data (documents, maps, etc.).
pub fn delete_module(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(modules::table.find(id)).execute(conn)
}

/// Check if a module exists.
pub fn module_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(modules::table.find(id))).get_result(conn)
}

/// Count modules for a campaign.
pub fn count_modules(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<i64> {
    modules::table
        .filter(modules::campaign_id.eq(campaign_id))
        .count()
        .get_result(conn)
}

/// Get the next available module number for a campaign.
pub fn next_module_number(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<i32> {
    let max: Option<i32> = modules::table
        .filter(modules::campaign_id.eq(campaign_id))
        .select(diesel::dsl::max(modules::module_number))
        .first(conn)?;

    Ok(max.unwrap_or(0) + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;

    fn setup_test_db() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

        conn.batch_execute(
            "CREATE TABLE campaigns (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                archived_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE modules (
                id TEXT PRIMARY KEY NOT NULL,
                campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT,
                module_number INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(campaign_id, module_number)
            );
            CREATE INDEX idx_modules_campaign ON modules(campaign_id);

            -- Insert test campaign
            INSERT INTO campaigns (id, name) VALUES ('camp-1', 'Test Campaign');
            PRAGMA foreign_keys = ON;",
        )
        .expect("Failed to create tables");

        conn
    }

    #[test]
    fn test_insert_and_get_module() {
        let mut conn = setup_test_db();

        let module = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        let id = insert_module(&mut conn, &module).expect("Failed to insert");
        assert_eq!(id, "mod-1");

        let retrieved = get_module(&mut conn, "mod-1").expect("Failed to get");
        assert_eq!(retrieved.id, "mod-1");
        assert_eq!(retrieved.campaign_id, "camp-1");
        assert_eq!(retrieved.name, "Chapter 1");
        assert_eq!(retrieved.module_number, 1);
        assert!(retrieved.description.is_none());
    }

    #[test]
    fn test_insert_module_with_description() {
        let mut conn = setup_test_db();

        let module = NewModule::new("mod-1", "camp-1", "Chapter 1", 1)
            .with_description("The beginning of our story");
        insert_module(&mut conn, &module).expect("Failed to insert");

        let retrieved = get_module(&mut conn, "mod-1").expect("Failed to get");
        assert_eq!(
            retrieved.description,
            Some("The beginning of our story".to_string())
        );
    }

    #[test]
    fn test_list_modules_ordered_by_number() {
        let mut conn = setup_test_db();

        // Insert out of order
        let module3 = NewModule::new("mod-3", "camp-1", "Chapter 3", 3);
        let module1 = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        let module2 = NewModule::new("mod-2", "camp-1", "Chapter 2", 2);
        insert_module(&mut conn, &module3).expect("Failed to insert");
        insert_module(&mut conn, &module1).expect("Failed to insert");
        insert_module(&mut conn, &module2).expect("Failed to insert");

        let modules = list_modules(&mut conn, "camp-1").expect("Failed to list");
        assert_eq!(modules.len(), 3);
        assert_eq!(modules[0].module_number, 1);
        assert_eq!(modules[1].module_number, 2);
        assert_eq!(modules[2].module_number, 3);
    }

    #[test]
    fn test_get_module_by_number() {
        let mut conn = setup_test_db();

        let module = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        insert_module(&mut conn, &module).expect("Failed to insert");

        let result = get_module_by_number(&mut conn, "camp-1", 1).expect("Failed to get");
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, "mod-1");

        let result = get_module_by_number(&mut conn, "camp-1", 2).expect("Failed to get");
        assert!(result.is_none());
    }

    #[test]
    fn test_update_module_name() {
        let mut conn = setup_test_db();

        let module = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        insert_module(&mut conn, &module).expect("Failed to insert");

        let update = UpdateModule::set_name("Chapter One: The Beginning", "2024-01-20T12:00:00Z");
        update_module(&mut conn, "mod-1", &update).expect("Failed to update");

        let retrieved = get_module(&mut conn, "mod-1").expect("Failed to get");
        assert_eq!(retrieved.name, "Chapter One: The Beginning");
    }

    #[test]
    fn test_update_module_number() {
        let mut conn = setup_test_db();

        let module = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        insert_module(&mut conn, &module).expect("Failed to insert");

        let update = UpdateModule::set_module_number(5, "2024-01-20T12:00:00Z");
        update_module(&mut conn, "mod-1", &update).expect("Failed to update");

        let retrieved = get_module(&mut conn, "mod-1").expect("Failed to get");
        assert_eq!(retrieved.module_number, 5);
    }

    #[test]
    fn test_delete_module() {
        let mut conn = setup_test_db();

        let module = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        insert_module(&mut conn, &module).expect("Failed to insert");

        assert!(module_exists(&mut conn, "mod-1").expect("Failed to check"));

        delete_module(&mut conn, "mod-1").expect("Failed to delete");

        assert!(!module_exists(&mut conn, "mod-1").expect("Failed to check"));
    }

    #[test]
    fn test_get_module_optional() {
        let mut conn = setup_test_db();

        let result = get_module_optional(&mut conn, "nonexistent").expect("Failed to query");
        assert!(result.is_none());

        let module = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        insert_module(&mut conn, &module).expect("Failed to insert");

        let result = get_module_optional(&mut conn, "mod-1").expect("Failed to query");
        assert!(result.is_some());
    }

    #[test]
    fn test_count_modules() {
        let mut conn = setup_test_db();

        assert_eq!(count_modules(&mut conn, "camp-1").expect("Failed to count"), 0);

        let module1 = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        let module2 = NewModule::new("mod-2", "camp-1", "Chapter 2", 2);
        insert_module(&mut conn, &module1).expect("Failed to insert");
        insert_module(&mut conn, &module2).expect("Failed to insert");

        assert_eq!(count_modules(&mut conn, "camp-1").expect("Failed to count"), 2);
    }

    #[test]
    fn test_next_module_number() {
        let mut conn = setup_test_db();

        // Empty campaign should return 1
        assert_eq!(
            next_module_number(&mut conn, "camp-1").expect("Failed to get next"),
            1
        );

        let module1 = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        insert_module(&mut conn, &module1).expect("Failed to insert");

        assert_eq!(
            next_module_number(&mut conn, "camp-1").expect("Failed to get next"),
            2
        );

        let module5 = NewModule::new("mod-5", "camp-1", "Chapter 5", 5);
        insert_module(&mut conn, &module5).expect("Failed to insert");

        assert_eq!(
            next_module_number(&mut conn, "camp-1").expect("Failed to get next"),
            6
        );
    }
}
