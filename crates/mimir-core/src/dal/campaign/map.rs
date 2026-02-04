//! Map Data Access Layer
//!
//! Database operations for maps.

use crate::models::campaign::{Map, NewMap, UpdateMap};
use crate::schema::maps;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new map.
pub fn insert_map(conn: &mut SqliteConnection, map: &NewMap) -> QueryResult<String> {
    diesel::insert_into(maps::table)
        .values(map)
        .execute(conn)?;

    Ok(map.id.to_string())
}

/// Get a map by ID.
pub fn get_map(conn: &mut SqliteConnection, id: &str) -> QueryResult<Map> {
    maps::table.find(id).first(conn)
}

/// Get a map by ID, returning None if not found.
pub fn get_map_optional(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Map>> {
    maps::table.find(id).first(conn).optional()
}

/// List all maps for a campaign (including module maps).
pub fn list_campaign_maps(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<Vec<Map>> {
    maps::table
        .filter(maps::campaign_id.eq(campaign_id))
        .order(maps::sort_order.asc())
        .then_order_by(maps::name.asc())
        .load(conn)
}

/// List only campaign-level maps (not in any module).
pub fn list_campaign_level_maps(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<Vec<Map>> {
    maps::table
        .filter(maps::campaign_id.eq(campaign_id))
        .filter(maps::module_id.is_null())
        .order(maps::sort_order.asc())
        .then_order_by(maps::name.asc())
        .load(conn)
}

/// List all maps for a module.
pub fn list_module_maps(conn: &mut SqliteConnection, module_id: &str) -> QueryResult<Vec<Map>> {
    maps::table
        .filter(maps::module_id.eq(module_id))
        .order(maps::sort_order.asc())
        .then_order_by(maps::name.asc())
        .load(conn)
}

/// Update a map.
pub fn update_map(conn: &mut SqliteConnection, id: &str, update: &UpdateMap) -> QueryResult<usize> {
    diesel::update(maps::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a map by ID.
pub fn delete_map(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(maps::table.find(id)).execute(conn)
}

/// Check if a map exists.
pub fn map_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(maps::table.find(id))).get_result(conn)
}

/// Count maps for a campaign.
pub fn count_campaign_maps(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<i64> {
    maps::table
        .filter(maps::campaign_id.eq(campaign_id))
        .count()
        .get_result(conn)
}

/// Count maps for a module.
pub fn count_module_maps(conn: &mut SqliteConnection, module_id: &str) -> QueryResult<i64> {
    maps::table
        .filter(maps::module_id.eq(module_id))
        .count()
        .get_result(conn)
}

/// Get the next sort order for a new map in a campaign.
pub fn get_next_campaign_sort_order(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<i32> {
    use diesel::dsl::max;

    maps::table
        .filter(maps::campaign_id.eq(campaign_id))
        .filter(maps::module_id.is_null())
        .select(max(maps::sort_order))
        .first::<Option<i32>>(conn)
        .map(|opt| opt.unwrap_or(0) + 1)
}

/// Get the next sort order for a new map in a module.
pub fn get_next_module_sort_order(
    conn: &mut SqliteConnection,
    module_id: &str,
) -> QueryResult<i32> {
    use diesel::dsl::max;

    maps::table
        .filter(maps::module_id.eq(module_id))
        .select(max(maps::sort_order))
        .first::<Option<i32>>(conn)
        .map(|opt| opt.unwrap_or(0) + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::campaign::{insert_campaign, insert_module, insert_campaign_asset};
    use crate::models::campaign::{NewCampaign, NewModule, NewCampaignAsset};

    fn setup_test_data(conn: &mut SqliteConnection) {
        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");

        let module = NewModule::new("mod-1", "camp-1", "Chapter 1", 1);
        insert_module(conn, &module).expect("Failed to create module");

        let asset1 = NewCampaignAsset::for_campaign("asset-1", "camp-1", "map1.uvtt", "application/octet-stream", "/blobs/map1.uvtt");
        insert_campaign_asset(conn, &asset1).expect("Failed to create asset 1");

        let asset2 = NewCampaignAsset::for_campaign("asset-2", "camp-1", "map2.uvtt", "application/octet-stream", "/blobs/map2.uvtt");
        insert_campaign_asset(conn, &asset2).expect("Failed to create asset 2");
    }

    #[test]
    fn test_insert_and_get_map() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let map = NewMap::for_campaign("map-1", "camp-1", "World Map", "asset-1");
        let id = insert_map(&mut conn, &map).expect("Failed to insert");
        assert_eq!(id, "map-1");

        let retrieved = get_map(&mut conn, "map-1").expect("Failed to get");
        assert_eq!(retrieved.name, "World Map");
        assert!(retrieved.module_id.is_none());
        assert_eq!(retrieved.lighting_mode, "bright");
    }

    #[test]
    fn test_insert_module_map() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let map = NewMap::for_module("map-1", "camp-1", "mod-1", "Goblin Cave", "asset-1")
            .with_lighting_mode(crate::models::campaign::LightingMode::Dark);
        insert_map(&mut conn, &map).expect("Failed to insert");

        let retrieved = get_map(&mut conn, "map-1").expect("Failed to get");
        assert_eq!(retrieved.module_id, Some("mod-1".to_string()));
        assert_eq!(retrieved.lighting_mode, "dark");
    }

    #[test]
    fn test_list_campaign_maps() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let world_map = NewMap::for_campaign("map-1", "camp-1", "World Map", "asset-1");
        let dungeon = NewMap::for_module("map-2", "camp-1", "mod-1", "Dungeon", "asset-2");
        insert_map(&mut conn, &world_map).expect("Failed to insert");
        insert_map(&mut conn, &dungeon).expect("Failed to insert");

        // All campaign maps (includes module maps)
        let all = list_campaign_maps(&mut conn, "camp-1").expect("Failed to list");
        assert_eq!(all.len(), 2);

        // Only campaign-level maps
        let campaign_level = list_campaign_level_maps(&mut conn, "camp-1").expect("Failed to list");
        assert_eq!(campaign_level.len(), 1);
        assert_eq!(campaign_level[0].name, "World Map");
    }

    #[test]
    fn test_list_module_maps() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let map1 = NewMap::for_module("map-1", "camp-1", "mod-1", "Floor 1", "asset-1")
            .with_sort_order(1);
        let map2 = NewMap::for_module("map-2", "camp-1", "mod-1", "Floor 2", "asset-2")
            .with_sort_order(2);
        let world = NewMap::for_campaign("map-3", "camp-1", "World Map", "asset-1");
        insert_map(&mut conn, &map1).expect("Failed to insert");
        insert_map(&mut conn, &map2).expect("Failed to insert");
        insert_map(&mut conn, &world).expect("Failed to insert");

        let module_maps = list_module_maps(&mut conn, "mod-1").expect("Failed to list");
        assert_eq!(module_maps.len(), 2);
        // Should be sorted by sort_order
        assert_eq!(module_maps[0].name, "Floor 1");
        assert_eq!(module_maps[1].name, "Floor 2");
    }

    #[test]
    fn test_update_map_lighting() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let map = NewMap::for_module("map-1", "camp-1", "mod-1", "Dungeon", "asset-1");
        insert_map(&mut conn, &map).expect("Failed to insert");

        let update = UpdateMap::set_lighting_mode(
            crate::models::campaign::LightingMode::Dim,
            "2024-01-20T12:00:00Z",
        );
        update_map(&mut conn, "map-1", &update).expect("Failed to update");

        let retrieved = get_map(&mut conn, "map-1").expect("Failed to get");
        assert_eq!(retrieved.lighting_mode, "dim");
    }

    #[test]
    fn test_update_map_move_to_module() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let map = NewMap::for_campaign("map-1", "camp-1", "Regional Map", "asset-1");
        insert_map(&mut conn, &map).expect("Failed to insert");

        let update = UpdateMap::move_to_module("mod-1", "2024-01-20T12:00:00Z");
        update_map(&mut conn, "map-1", &update).expect("Failed to update");

        let retrieved = get_map(&mut conn, "map-1").expect("Failed to get");
        assert_eq!(retrieved.module_id, Some("mod-1".to_string()));
    }

    #[test]
    fn test_delete_map() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let map = NewMap::for_campaign("map-1", "camp-1", "Map", "asset-1");
        insert_map(&mut conn, &map).expect("Failed to insert");

        assert!(map_exists(&mut conn, "map-1").expect("Failed to check"));

        delete_map(&mut conn, "map-1").expect("Failed to delete");

        assert!(!map_exists(&mut conn, "map-1").expect("Failed to check"));
    }

    #[test]
    fn test_count_maps() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        assert_eq!(
            count_campaign_maps(&mut conn, "camp-1").expect("Failed to count"),
            0
        );

        let map1 = NewMap::for_campaign("map-1", "camp-1", "World", "asset-1");
        let map2 = NewMap::for_module("map-2", "camp-1", "mod-1", "Dungeon", "asset-2");
        insert_map(&mut conn, &map1).expect("Failed to insert");
        insert_map(&mut conn, &map2).expect("Failed to insert");

        assert_eq!(
            count_campaign_maps(&mut conn, "camp-1").expect("Failed to count"),
            2
        );
        assert_eq!(
            count_module_maps(&mut conn, "mod-1").expect("Failed to count"),
            1
        );
    }

    #[test]
    fn test_get_next_sort_order() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        // First map should get sort order 1
        let next = get_next_campaign_sort_order(&mut conn, "camp-1").expect("Failed to get");
        assert_eq!(next, 1);

        let map = NewMap::for_campaign("map-1", "camp-1", "Map 1", "asset-1")
            .with_sort_order(5);
        insert_map(&mut conn, &map).expect("Failed to insert");

        // Next map should get sort order 6
        let next = get_next_campaign_sort_order(&mut conn, "camp-1").expect("Failed to get");
        assert_eq!(next, 6);
    }
}
