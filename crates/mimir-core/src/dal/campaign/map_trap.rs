//! MapTrap Data Access Layer
//!
//! Database operations for trap placements on maps.

use crate::models::campaign::{MapTrap, NewMapTrap, UpdateMapTrap};
use crate::schema::map_traps;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new map trap.
pub fn insert_map_trap(conn: &mut SqliteConnection, trap: &NewMapTrap) -> QueryResult<String> {
    diesel::insert_into(map_traps::table)
        .values(trap)
        .execute(conn)?;

    Ok(trap.id.to_string())
}

/// Get a map trap by ID.
pub fn get_map_trap(conn: &mut SqliteConnection, id: &str) -> QueryResult<MapTrap> {
    map_traps::table.find(id).first(conn)
}

/// Get a map trap by ID, returning None if not found.
pub fn get_map_trap_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<MapTrap>> {
    map_traps::table.find(id).first(conn).optional()
}

/// List all traps for a map.
pub fn list_map_traps(conn: &mut SqliteConnection, map_id: &str) -> QueryResult<Vec<MapTrap>> {
    map_traps::table
        .filter(map_traps::map_id.eq(map_id))
        .load(conn)
}

/// List visible traps for a map (for player view).
pub fn list_visible_map_traps(
    conn: &mut SqliteConnection,
    map_id: &str,
) -> QueryResult<Vec<MapTrap>> {
    map_traps::table
        .filter(map_traps::map_id.eq(map_id))
        .filter(map_traps::visible.eq(1))
        .load(conn)
}

/// List armed (not triggered) traps for a map.
pub fn list_armed_map_traps(
    conn: &mut SqliteConnection,
    map_id: &str,
) -> QueryResult<Vec<MapTrap>> {
    map_traps::table
        .filter(map_traps::map_id.eq(map_id))
        .filter(map_traps::triggered.eq(0))
        .load(conn)
}

/// Update a map trap.
pub fn update_map_trap(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateMapTrap,
) -> QueryResult<usize> {
    diesel::update(map_traps::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a map trap by ID.
pub fn delete_map_trap(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(map_traps::table.find(id)).execute(conn)
}

/// Delete all traps for a map.
pub fn delete_all_map_traps(conn: &mut SqliteConnection, map_id: &str) -> QueryResult<usize> {
    diesel::delete(map_traps::table.filter(map_traps::map_id.eq(map_id))).execute(conn)
}

/// Check if a map trap exists.
pub fn map_trap_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(map_traps::table.find(id))).get_result(conn)
}

/// Count traps for a map.
pub fn count_map_traps(conn: &mut SqliteConnection, map_id: &str) -> QueryResult<i64> {
    map_traps::table
        .filter(map_traps::map_id.eq(map_id))
        .count()
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::campaign::{insert_campaign, insert_campaign_asset, insert_map};
    use crate::models::campaign::{NewCampaign, NewCampaignAsset, NewMap};

    fn setup_test_data(conn: &mut SqliteConnection) {
        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");

        let asset = NewCampaignAsset::for_campaign("asset-1", "camp-1", "goblin-cave.uvtt", "application/octet-stream", "/blobs/goblin-cave.uvtt");
        insert_campaign_asset(conn, &asset).expect("Failed to create asset");

        let map = NewMap::for_campaign("map-1", "camp-1", "Goblin Cave", "asset-1");
        insert_map(conn, &map).expect("Failed to create map");
    }

    #[test]
    fn test_insert_and_get_map_trap() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let trap = NewMapTrap::new("trap-1", "map-1", "Pit Trap", 5, 10);
        let id = insert_map_trap(&mut conn, &trap).expect("Failed to insert");
        assert_eq!(id, "trap-1");

        let retrieved = get_map_trap(&mut conn, "trap-1").expect("Failed to get");
        assert_eq!(retrieved.name, "Pit Trap");
        assert_eq!(retrieved.grid_x, 5);
        assert_eq!(retrieved.grid_y, 10);
        assert!(!retrieved.is_triggered());
        assert!(!retrieved.is_visible());
    }

    #[test]
    fn test_insert_with_details() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let trap = NewMapTrap::new("trap-1", "map-1", "Arrow Trap", 0, 0)
            .with_description("A pressure plate triggers arrows")
            .with_trigger("Stepping on the plate")
            .with_effect("2d6 piercing damage")
            .with_dc(15);
        insert_map_trap(&mut conn, &trap).expect("Failed to insert");

        let retrieved = get_map_trap(&mut conn, "trap-1").expect("Failed to get");
        assert!(retrieved.description.is_some());
        assert!(retrieved.trigger_description.is_some());
        assert!(retrieved.effect_description.is_some());
        assert_eq!(retrieved.dc, Some(15));
    }

    #[test]
    fn test_list_map_traps() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let trap1 = NewMapTrap::new("trap-1", "map-1", "Pit", 1, 1);
        let trap2 = NewMapTrap::new("trap-2", "map-1", "Arrow", 2, 2);
        insert_map_trap(&mut conn, &trap1).expect("Failed to insert");
        insert_map_trap(&mut conn, &trap2).expect("Failed to insert");

        let traps = list_map_traps(&mut conn, "map-1").expect("Failed to list");
        assert_eq!(traps.len(), 2);
    }

    #[test]
    fn test_list_visible_traps() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let hidden = NewMapTrap::new("trap-1", "map-1", "Hidden Trap", 1, 1);
        let visible = NewMapTrap::new("trap-2", "map-1", "Obvious Trap", 2, 2).visible();
        insert_map_trap(&mut conn, &hidden).expect("Failed to insert");
        insert_map_trap(&mut conn, &visible).expect("Failed to insert");

        let traps = list_visible_map_traps(&mut conn, "map-1").expect("Failed to list");
        assert_eq!(traps.len(), 1);
        assert_eq!(traps[0].name, "Obvious Trap");
    }

    #[test]
    fn test_list_armed_traps() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let trap1 = NewMapTrap::new("trap-1", "map-1", "Armed Trap", 1, 1);
        insert_map_trap(&mut conn, &trap1).expect("Failed to insert");

        // Trigger trap-1
        let update = UpdateMapTrap::trigger("2024-01-20T12:00:00Z");
        update_map_trap(&mut conn, "trap-1", &update).expect("Failed to update");

        let trap2 = NewMapTrap::new("trap-2", "map-1", "Still Armed", 2, 2);
        insert_map_trap(&mut conn, &trap2).expect("Failed to insert");

        let armed = list_armed_map_traps(&mut conn, "map-1").expect("Failed to list");
        assert_eq!(armed.len(), 1);
        assert_eq!(armed[0].name, "Still Armed");
    }

    #[test]
    fn test_trigger_and_reset() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let trap = NewMapTrap::new("trap-1", "map-1", "Resettable Trap", 0, 0);
        insert_map_trap(&mut conn, &trap).expect("Failed to insert");

        // Trigger it
        let update = UpdateMapTrap::trigger("2024-01-20T12:00:00Z");
        update_map_trap(&mut conn, "trap-1", &update).expect("Failed to update");

        let retrieved = get_map_trap(&mut conn, "trap-1").expect("Failed to get");
        assert!(retrieved.is_triggered());

        // Reset it
        let update = UpdateMapTrap::reset("2024-01-20T12:01:00Z");
        update_map_trap(&mut conn, "trap-1", &update).expect("Failed to update");

        let retrieved = get_map_trap(&mut conn, "trap-1").expect("Failed to get");
        assert!(!retrieved.is_triggered());
        assert!(retrieved.is_armed());
    }

    #[test]
    fn test_update_visible() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let trap = NewMapTrap::new("trap-1", "map-1", "Trap", 0, 0);
        insert_map_trap(&mut conn, &trap).expect("Failed to insert");

        let update = UpdateMapTrap::set_visible(true, "2024-01-20T12:00:00Z");
        update_map_trap(&mut conn, "trap-1", &update).expect("Failed to update");

        let retrieved = get_map_trap(&mut conn, "trap-1").expect("Failed to get");
        assert!(retrieved.is_visible());
    }

    #[test]
    fn test_delete_map_trap() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let trap = NewMapTrap::new("trap-1", "map-1", "Doomed Trap", 0, 0);
        insert_map_trap(&mut conn, &trap).expect("Failed to insert");

        assert!(map_trap_exists(&mut conn, "trap-1").expect("Failed to check"));

        delete_map_trap(&mut conn, "trap-1").expect("Failed to delete");

        assert!(!map_trap_exists(&mut conn, "trap-1").expect("Failed to check"));
    }

    #[test]
    fn test_count_map_traps() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        assert_eq!(
            count_map_traps(&mut conn, "map-1").expect("Failed to count"),
            0
        );

        let trap1 = NewMapTrap::new("trap-1", "map-1", "Trap 1", 1, 1);
        let trap2 = NewMapTrap::new("trap-2", "map-1", "Trap 2", 2, 2);
        insert_map_trap(&mut conn, &trap1).expect("Failed to insert");
        insert_map_trap(&mut conn, &trap2).expect("Failed to insert");

        assert_eq!(
            count_map_traps(&mut conn, "map-1").expect("Failed to count"),
            2
        );
    }
}
