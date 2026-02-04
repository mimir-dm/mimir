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

/// Reorder a module by moving it to a new position (1-indexed).
///
/// Uses a sentinel value (-1) to work around the UNIQUE(campaign_id, module_number) constraint,
/// then shifts affected modules and places the target module at the new position.
pub fn reorder_module(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    module_id: &str,
    new_position: i32,
) -> QueryResult<()> {
    use diesel::Connection;

    conn.transaction(|conn| {
        // Get the module's current position
        let module = get_module(conn, module_id)?;
        let current = module.module_number;

        if current == new_position {
            return Ok(());
        }

        // Validate new_position is in range
        let total = count_modules(conn, campaign_id)? as i32;
        if new_position < 1 || new_position > total {
            return Err(diesel::result::Error::RollbackTransaction);
        }

        let now = chrono::Utc::now().to_rfc3339();

        // Step 1: Move target module to sentinel (-1)
        diesel::update(modules::table.find(module_id))
            .set((
                modules::module_number.eq(-1),
                modules::updated_at.eq(&now),
            ))
            .execute(conn)?;

        // Step 2: Shift affected modules one at a time to avoid UNIQUE conflicts
        if current < new_position {
            // Moving down: shift [current+1, new_position] each by -1, in ascending order
            for pos in (current + 1)..=new_position {
                diesel::update(
                    modules::table
                        .filter(modules::campaign_id.eq(campaign_id))
                        .filter(modules::module_number.eq(pos)),
                )
                .set((
                    modules::module_number.eq(pos - 1),
                    modules::updated_at.eq(&now),
                ))
                .execute(conn)?;
            }
        } else {
            // Moving up: shift [new_position, current-1] each by +1, in descending order
            for pos in (new_position..current).rev() {
                diesel::update(
                    modules::table
                        .filter(modules::campaign_id.eq(campaign_id))
                        .filter(modules::module_number.eq(pos)),
                )
                .set((
                    modules::module_number.eq(pos + 1),
                    modules::updated_at.eq(&now),
                ))
                .execute(conn)?;
            }
        }

        // Step 3: Place module at new position
        diesel::update(modules::table.find(module_id))
            .set((
                modules::module_number.eq(new_position),
                modules::updated_at.eq(&now),
            ))
            .execute(conn)?;

        Ok(())
    })
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
    use crate::db::test_connection;
    use crate::dal::campaign::insert_campaign;
    use crate::models::campaign::NewCampaign;

    fn setup_test_data(conn: &mut SqliteConnection) {
        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");
    }

    #[test]
    fn test_insert_and_get_module() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

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
        let mut conn = test_connection();
        setup_test_data(&mut conn);

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
        let mut conn = test_connection();
        setup_test_data(&mut conn);

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
        let mut conn = test_connection();
        setup_test_data(&mut conn);

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
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let module = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        insert_module(&mut conn, &module).expect("Failed to insert");

        let update = UpdateModule::set_name("Chapter One: The Beginning", "2024-01-20T12:00:00Z");
        update_module(&mut conn, "mod-1", &update).expect("Failed to update");

        let retrieved = get_module(&mut conn, "mod-1").expect("Failed to get");
        assert_eq!(retrieved.name, "Chapter One: The Beginning");
    }

    #[test]
    fn test_update_module_number() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let module = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        insert_module(&mut conn, &module).expect("Failed to insert");

        let update = UpdateModule::set_module_number(5, "2024-01-20T12:00:00Z");
        update_module(&mut conn, "mod-1", &update).expect("Failed to update");

        let retrieved = get_module(&mut conn, "mod-1").expect("Failed to get");
        assert_eq!(retrieved.module_number, 5);
    }

    #[test]
    fn test_delete_module() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let module = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        insert_module(&mut conn, &module).expect("Failed to insert");

        assert!(module_exists(&mut conn, "mod-1").expect("Failed to check"));

        delete_module(&mut conn, "mod-1").expect("Failed to delete");

        assert!(!module_exists(&mut conn, "mod-1").expect("Failed to check"));
    }

    #[test]
    fn test_get_module_optional() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let result = get_module_optional(&mut conn, "nonexistent").expect("Failed to query");
        assert!(result.is_none());

        let module = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        insert_module(&mut conn, &module).expect("Failed to insert");

        let result = get_module_optional(&mut conn, "mod-1").expect("Failed to query");
        assert!(result.is_some());
    }

    #[test]
    fn test_count_modules() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        assert_eq!(count_modules(&mut conn, "camp-1").expect("Failed to count"), 0);

        let module1 = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        let module2 = NewModule::new("mod-2", "camp-1", "Chapter 2", 2);
        insert_module(&mut conn, &module1).expect("Failed to insert");
        insert_module(&mut conn, &module2).expect("Failed to insert");

        assert_eq!(count_modules(&mut conn, "camp-1").expect("Failed to count"), 2);
    }

    #[test]
    fn test_reorder_module_move_down() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let m1 = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        let m2 = NewModule::new("mod-2", "camp-1", "Chapter 2", 2);
        let m3 = NewModule::new("mod-3", "camp-1", "Chapter 3", 3);
        insert_module(&mut conn, &m1).unwrap();
        insert_module(&mut conn, &m2).unwrap();
        insert_module(&mut conn, &m3).unwrap();

        // Move module 1 to position 3
        reorder_module(&mut conn, "camp-1", "mod-1", 3).unwrap();

        let modules = list_modules(&mut conn, "camp-1").unwrap();
        assert_eq!(modules[0].id, "mod-2");
        assert_eq!(modules[0].module_number, 1);
        assert_eq!(modules[1].id, "mod-3");
        assert_eq!(modules[1].module_number, 2);
        assert_eq!(modules[2].id, "mod-1");
        assert_eq!(modules[2].module_number, 3);
    }

    #[test]
    fn test_reorder_module_move_up() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let m1 = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        let m2 = NewModule::new("mod-2", "camp-1", "Chapter 2", 2);
        let m3 = NewModule::new("mod-3", "camp-1", "Chapter 3", 3);
        insert_module(&mut conn, &m1).unwrap();
        insert_module(&mut conn, &m2).unwrap();
        insert_module(&mut conn, &m3).unwrap();

        // Move module 3 to position 1
        reorder_module(&mut conn, "camp-1", "mod-3", 1).unwrap();

        let modules = list_modules(&mut conn, "camp-1").unwrap();
        assert_eq!(modules[0].id, "mod-3");
        assert_eq!(modules[0].module_number, 1);
        assert_eq!(modules[1].id, "mod-1");
        assert_eq!(modules[1].module_number, 2);
        assert_eq!(modules[2].id, "mod-2");
        assert_eq!(modules[2].module_number, 3);
    }

    #[test]
    fn test_reorder_module_no_op() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let m1 = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        let m2 = NewModule::new("mod-2", "camp-1", "Chapter 2", 2);
        insert_module(&mut conn, &m1).unwrap();
        insert_module(&mut conn, &m2).unwrap();

        // Move module 1 to position 1 (no-op)
        reorder_module(&mut conn, "camp-1", "mod-1", 1).unwrap();

        let modules = list_modules(&mut conn, "camp-1").unwrap();
        assert_eq!(modules[0].id, "mod-1");
        assert_eq!(modules[1].id, "mod-2");
    }

    #[test]
    fn test_reorder_module_invalid_position() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let m1 = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        insert_module(&mut conn, &m1).unwrap();

        // Position 0 is invalid
        let result = reorder_module(&mut conn, "camp-1", "mod-1", 0);
        assert!(result.is_err());

        // Position 2 is invalid (only 1 module)
        let result = reorder_module(&mut conn, "camp-1", "mod-1", 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_next_module_number() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

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
