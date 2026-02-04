//! TokenPlacement Data Access Layer
//!
//! Database operations for token placements on maps.

use crate::models::campaign::{NewTokenPlacement, TokenPlacement, UpdateTokenPlacement};
use crate::schema::token_placements;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new token placement.
pub fn insert_token_placement(
    conn: &mut SqliteConnection,
    placement: &NewTokenPlacement,
) -> QueryResult<String> {
    diesel::insert_into(token_placements::table)
        .values(placement)
        .execute(conn)?;

    Ok(placement.id.to_string())
}

/// Get a token placement by ID.
pub fn get_token_placement(conn: &mut SqliteConnection, id: &str) -> QueryResult<TokenPlacement> {
    token_placements::table.find(id).first(conn)
}

/// Get a token placement by ID, returning None if not found.
pub fn get_token_placement_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<TokenPlacement>> {
    token_placements::table.find(id).first(conn).optional()
}

/// List all token placements for a map.
pub fn list_token_placements(
    conn: &mut SqliteConnection,
    map_id: &str,
) -> QueryResult<Vec<TokenPlacement>> {
    token_placements::table
        .filter(token_placements::map_id.eq(map_id))
        .load(conn)
}

/// List visible token placements for a map (for player view).
pub fn list_visible_token_placements(
    conn: &mut SqliteConnection,
    map_id: &str,
) -> QueryResult<Vec<TokenPlacement>> {
    token_placements::table
        .filter(token_placements::map_id.eq(map_id))
        .filter(token_placements::hidden.eq(0))
        .load(conn)
}

/// List token placements for a specific monster.
pub fn list_token_placements_for_monster(
    conn: &mut SqliteConnection,
    module_monster_id: &str,
) -> QueryResult<Vec<TokenPlacement>> {
    token_placements::table
        .filter(token_placements::module_monster_id.eq(module_monster_id))
        .load(conn)
}

/// List token placements for a specific NPC.
pub fn list_token_placements_for_npc(
    conn: &mut SqliteConnection,
    module_npc_id: &str,
) -> QueryResult<Vec<TokenPlacement>> {
    token_placements::table
        .filter(token_placements::module_npc_id.eq(module_npc_id))
        .load(conn)
}

/// Update a token placement.
pub fn update_token_placement(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateTokenPlacement,
) -> QueryResult<usize> {
    diesel::update(token_placements::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a token placement by ID.
pub fn delete_token_placement(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(token_placements::table.find(id)).execute(conn)
}

/// Delete all token placements for a map.
pub fn delete_all_token_placements(
    conn: &mut SqliteConnection,
    map_id: &str,
) -> QueryResult<usize> {
    diesel::delete(token_placements::table.filter(token_placements::map_id.eq(map_id)))
        .execute(conn)
}

/// Check if a token placement exists.
pub fn token_placement_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(token_placements::table.find(id))).get_result(conn)
}

/// Count token placements for a map.
pub fn count_token_placements(conn: &mut SqliteConnection, map_id: &str) -> QueryResult<i64> {
    token_placements::table
        .filter(token_placements::map_id.eq(map_id))
        .count()
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::campaign::{insert_campaign, insert_module, insert_campaign_asset, insert_map, insert_module_monster, insert_module_npc};
    use crate::models::campaign::{NewCampaign, NewModule, NewCampaignAsset, NewMap, NewModuleMonster, NewModuleNpc};

    fn setup_test_data(conn: &mut SqliteConnection) {
        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");

        let module = NewModule::new("mod-1", "camp-1", "Dungeon", 1);
        insert_module(conn, &module).expect("Failed to create module");

        let asset = NewCampaignAsset::for_campaign("asset-1", "camp-1", "goblin-cave.uvtt", "application/octet-stream", "/blobs/goblin-cave.uvtt");
        insert_campaign_asset(conn, &asset).expect("Failed to create asset");

        let map = NewMap::for_campaign("map-1", "camp-1", "Goblin Cave", "asset-1");
        insert_map(conn, &map).expect("Failed to create map");

        let monster = NewModuleMonster::new("mm-1", "mod-1", "Goblin", "MM");
        insert_module_monster(conn, &monster).expect("Failed to create monster");

        let npc = NewModuleNpc::new("npc-1", "mod-1", "Sildar");
        insert_module_npc(conn, &npc).expect("Failed to create npc");
    }

    #[test]
    fn test_insert_and_get_monster_placement() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let placement = NewTokenPlacement::for_monster("tp-1", "map-1", "mm-1", 5, 10);
        let id = insert_token_placement(&mut conn, &placement).expect("Failed to insert");
        assert_eq!(id, "tp-1");

        let retrieved = get_token_placement(&mut conn, "tp-1").expect("Failed to get");
        assert!(retrieved.is_monster());
        assert!(!retrieved.is_npc());
        assert_eq!(retrieved.grid_x, 5);
        assert_eq!(retrieved.grid_y, 10);
    }

    #[test]
    fn test_insert_and_get_npc_placement() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let placement = NewTokenPlacement::for_npc("tp-1", "map-1", "npc-1", 3, 7);
        insert_token_placement(&mut conn, &placement).expect("Failed to insert");

        let retrieved = get_token_placement(&mut conn, "tp-1").expect("Failed to get");
        assert!(!retrieved.is_monster());
        assert!(retrieved.is_npc());
    }

    #[test]
    fn test_insert_with_options() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let placement = NewTokenPlacement::for_monster("tp-1", "map-1", "mm-1", 0, 0)
            .with_label("Goblin Boss")
            .with_faction_color("#FF0000")
            .hidden();
        insert_token_placement(&mut conn, &placement).expect("Failed to insert");

        let retrieved = get_token_placement(&mut conn, "tp-1").expect("Failed to get");
        assert_eq!(retrieved.label, Some("Goblin Boss".to_string()));
        assert_eq!(retrieved.faction_color, Some("#FF0000".to_string()));
        assert!(retrieved.is_hidden());
    }

    #[test]
    fn test_list_token_placements() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let p1 = NewTokenPlacement::for_monster("tp-1", "map-1", "mm-1", 1, 1);
        let p2 = NewTokenPlacement::for_npc("tp-2", "map-1", "npc-1", 2, 2);
        insert_token_placement(&mut conn, &p1).expect("Failed to insert");
        insert_token_placement(&mut conn, &p2).expect("Failed to insert");

        let placements = list_token_placements(&mut conn, "map-1").expect("Failed to list");
        assert_eq!(placements.len(), 2);
    }

    #[test]
    fn test_list_visible_placements() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let visible = NewTokenPlacement::for_monster("tp-1", "map-1", "mm-1", 1, 1);
        let hidden = NewTokenPlacement::for_npc("tp-2", "map-1", "npc-1", 2, 2).hidden();
        insert_token_placement(&mut conn, &visible).expect("Failed to insert");
        insert_token_placement(&mut conn, &hidden).expect("Failed to insert");

        let placements = list_visible_token_placements(&mut conn, "map-1").expect("Failed to list");
        assert_eq!(placements.len(), 1);
        assert_eq!(placements[0].id, "tp-1");
    }

    #[test]
    fn test_update_position() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let placement = NewTokenPlacement::for_monster("tp-1", "map-1", "mm-1", 0, 0);
        insert_token_placement(&mut conn, &placement).expect("Failed to insert");

        let update = UpdateTokenPlacement::set_position(15, 20);
        update_token_placement(&mut conn, "tp-1", &update).expect("Failed to update");

        let retrieved = get_token_placement(&mut conn, "tp-1").expect("Failed to get");
        assert_eq!(retrieved.grid_x, 15);
        assert_eq!(retrieved.grid_y, 20);
    }

    #[test]
    fn test_update_hidden() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let placement = NewTokenPlacement::for_monster("tp-1", "map-1", "mm-1", 0, 0);
        insert_token_placement(&mut conn, &placement).expect("Failed to insert");

        let update = UpdateTokenPlacement::set_hidden(true);
        update_token_placement(&mut conn, "tp-1", &update).expect("Failed to update");

        let retrieved = get_token_placement(&mut conn, "tp-1").expect("Failed to get");
        assert!(retrieved.is_hidden());
    }

    #[test]
    fn test_delete_token_placement() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let placement = NewTokenPlacement::for_monster("tp-1", "map-1", "mm-1", 0, 0);
        insert_token_placement(&mut conn, &placement).expect("Failed to insert");

        assert!(token_placement_exists(&mut conn, "tp-1").expect("Failed to check"));

        delete_token_placement(&mut conn, "tp-1").expect("Failed to delete");

        assert!(!token_placement_exists(&mut conn, "tp-1").expect("Failed to check"));
    }

    #[test]
    fn test_count_token_placements() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        assert_eq!(
            count_token_placements(&mut conn, "map-1").expect("Failed to count"),
            0
        );

        let p1 = NewTokenPlacement::for_monster("tp-1", "map-1", "mm-1", 1, 1);
        let p2 = NewTokenPlacement::for_npc("tp-2", "map-1", "npc-1", 2, 2);
        insert_token_placement(&mut conn, &p1).expect("Failed to insert");
        insert_token_placement(&mut conn, &p2).expect("Failed to insert");

        assert_eq!(
            count_token_placements(&mut conn, "map-1").expect("Failed to count"),
            2
        );
    }
}
