//! ModuleNpc Data Access Layer
//!
//! Database operations for module NPCs (custom DM-created characters).

use crate::models::campaign::{ModuleNpc, NewModuleNpc, UpdateModuleNpc};
use crate::schema::module_npcs;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new module NPC.
pub fn insert_module_npc(conn: &mut SqliteConnection, npc: &NewModuleNpc) -> QueryResult<String> {
    diesel::insert_into(module_npcs::table)
        .values(npc)
        .execute(conn)?;

    Ok(npc.id.to_string())
}

/// Get a module NPC by ID.
pub fn get_module_npc(conn: &mut SqliteConnection, id: &str) -> QueryResult<ModuleNpc> {
    module_npcs::table.find(id).first(conn)
}

/// Get a module NPC by ID, returning None if not found.
pub fn get_module_npc_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<ModuleNpc>> {
    module_npcs::table.find(id).first(conn).optional()
}

/// List all NPCs for a module.
pub fn list_module_npcs(conn: &mut SqliteConnection, module_id: &str) -> QueryResult<Vec<ModuleNpc>> {
    module_npcs::table
        .filter(module_npcs::module_id.eq(module_id))
        .order(module_npcs::name.asc())
        .load(conn)
}

/// List NPCs by role.
pub fn list_module_npcs_by_role(
    conn: &mut SqliteConnection,
    module_id: &str,
    role: &str,
) -> QueryResult<Vec<ModuleNpc>> {
    module_npcs::table
        .filter(module_npcs::module_id.eq(module_id))
        .filter(module_npcs::role.eq(role))
        .order(module_npcs::name.asc())
        .load(conn)
}

/// Update a module NPC.
pub fn update_module_npc(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateModuleNpc,
) -> QueryResult<usize> {
    diesel::update(module_npcs::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a module NPC by ID.
pub fn delete_module_npc(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(module_npcs::table.find(id)).execute(conn)
}

/// Delete all NPCs for a module.
pub fn delete_all_module_npcs(conn: &mut SqliteConnection, module_id: &str) -> QueryResult<usize> {
    diesel::delete(module_npcs::table.filter(module_npcs::module_id.eq(module_id))).execute(conn)
}

/// Check if a module NPC exists.
pub fn module_npc_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(module_npcs::table.find(id))).get_result(conn)
}

/// Count NPCs for a module.
pub fn count_module_npcs(conn: &mut SqliteConnection, module_id: &str) -> QueryResult<i64> {
    module_npcs::table
        .filter(module_npcs::module_id.eq(module_id))
        .count()
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::campaign::{insert_campaign, insert_module};
    use crate::models::campaign::{NewCampaign, NewModule};

    fn setup_test_data(conn: &mut SqliteConnection) {
        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");

        let module = NewModule::new("mod-1", "camp-1", "Dungeon", 1);
        insert_module(conn, &module).expect("Failed to create module");
    }

    #[test]
    fn test_insert_and_get_module_npc() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let npc = NewModuleNpc::new("npc-1", "mod-1", "Sildar Hallwinter");
        let id = insert_module_npc(&mut conn, &npc).expect("Failed to insert");
        assert_eq!(id, "npc-1");

        let retrieved = get_module_npc(&mut conn, "npc-1").expect("Failed to get");
        assert_eq!(retrieved.name, "Sildar Hallwinter");
    }

    #[test]
    fn test_insert_with_details() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let npc = NewModuleNpc::new("npc-1", "mod-1", "Gundren Rockseeker")
            .with_role("Quest Giver")
            .with_description("A dwarf entrepreneur")
            .with_personality("Friendly but secretive")
            .with_motivation("Find the lost mine")
            .with_secrets("Knows the location of Wave Echo Cave");
        insert_module_npc(&mut conn, &npc).expect("Failed to insert");

        let retrieved = get_module_npc(&mut conn, "npc-1").expect("Failed to get");
        assert_eq!(retrieved.role, Some("Quest Giver".to_string()));
        assert!(retrieved.secrets.is_some());
    }

    #[test]
    fn test_list_module_npcs() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let npc1 = NewModuleNpc::new("npc-1", "mod-1", "Gundren");
        let npc2 = NewModuleNpc::new("npc-2", "mod-1", "Sildar");
        let npc3 = NewModuleNpc::new("npc-3", "mod-1", "Barthen");
        insert_module_npc(&mut conn, &npc1).expect("Failed to insert");
        insert_module_npc(&mut conn, &npc2).expect("Failed to insert");
        insert_module_npc(&mut conn, &npc3).expect("Failed to insert");

        let npcs = list_module_npcs(&mut conn, "mod-1").expect("Failed to list");
        assert_eq!(npcs.len(), 3);
        // Sorted alphabetically
        assert_eq!(npcs[0].name, "Barthen");
        assert_eq!(npcs[1].name, "Gundren");
        assert_eq!(npcs[2].name, "Sildar");
    }

    #[test]
    fn test_list_by_role() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let merchant = NewModuleNpc::new("npc-1", "mod-1", "Barthen")
            .with_role("Merchant");
        let quest_giver1 = NewModuleNpc::new("npc-2", "mod-1", "Gundren")
            .with_role("Quest Giver");
        let quest_giver2 = NewModuleNpc::new("npc-3", "mod-1", "Halia")
            .with_role("Quest Giver");
        insert_module_npc(&mut conn, &merchant).expect("Failed to insert");
        insert_module_npc(&mut conn, &quest_giver1).expect("Failed to insert");
        insert_module_npc(&mut conn, &quest_giver2).expect("Failed to insert");

        let quest_givers =
            list_module_npcs_by_role(&mut conn, "mod-1", "Quest Giver").expect("Failed to list");
        assert_eq!(quest_givers.len(), 2);

        let merchants =
            list_module_npcs_by_role(&mut conn, "mod-1", "Merchant").expect("Failed to list");
        assert_eq!(merchants.len(), 1);
    }

    #[test]
    fn test_update_module_npc() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let npc = NewModuleNpc::new("npc-1", "mod-1", "Unknown");
        insert_module_npc(&mut conn, &npc).expect("Failed to insert");

        let update = UpdateModuleNpc::set_name("Revealed Name", "2024-01-20T12:00:00Z");
        update_module_npc(&mut conn, "npc-1", &update).expect("Failed to update");

        let retrieved = get_module_npc(&mut conn, "npc-1").expect("Failed to get");
        assert_eq!(retrieved.name, "Revealed Name");
    }

    #[test]
    fn test_delete_module_npc() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let npc = NewModuleNpc::new("npc-1", "mod-1", "Doomed NPC");
        insert_module_npc(&mut conn, &npc).expect("Failed to insert");

        assert!(module_npc_exists(&mut conn, "npc-1").expect("Failed to check"));

        delete_module_npc(&mut conn, "npc-1").expect("Failed to delete");

        assert!(!module_npc_exists(&mut conn, "npc-1").expect("Failed to check"));
    }

    #[test]
    fn test_count_module_npcs() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        assert_eq!(
            count_module_npcs(&mut conn, "mod-1").expect("Failed to count"),
            0
        );

        let npc1 = NewModuleNpc::new("npc-1", "mod-1", "NPC 1");
        let npc2 = NewModuleNpc::new("npc-2", "mod-1", "NPC 2");
        insert_module_npc(&mut conn, &npc1).expect("Failed to insert");
        insert_module_npc(&mut conn, &npc2).expect("Failed to insert");

        assert_eq!(
            count_module_npcs(&mut conn, "mod-1").expect("Failed to count"),
            2
        );
    }
}
