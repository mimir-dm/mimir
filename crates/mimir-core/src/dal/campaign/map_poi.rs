//! MapPoi Data Access Layer
//!
//! Database operations for POI (Point of Interest) placements on maps.

use crate::models::campaign::{MapPoi, NewMapPoi, UpdateMapPoi};
use crate::schema::map_pois;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new map POI.
pub fn insert_map_poi(conn: &mut SqliteConnection, poi: &NewMapPoi) -> QueryResult<String> {
    diesel::insert_into(map_pois::table)
        .values(poi)
        .execute(conn)?;

    Ok(poi.id.to_string())
}

/// Get a map POI by ID.
pub fn get_map_poi(conn: &mut SqliteConnection, id: &str) -> QueryResult<MapPoi> {
    map_pois::table.find(id).first(conn)
}

/// Get a map POI by ID, returning None if not found.
pub fn get_map_poi_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<MapPoi>> {
    map_pois::table.find(id).first(conn).optional()
}

/// List all POIs for a map.
pub fn list_map_pois(conn: &mut SqliteConnection, map_id: &str) -> QueryResult<Vec<MapPoi>> {
    map_pois::table
        .filter(map_pois::map_id.eq(map_id))
        .load(conn)
}

/// List visible POIs for a map (for player view).
pub fn list_visible_map_pois(
    conn: &mut SqliteConnection,
    map_id: &str,
) -> QueryResult<Vec<MapPoi>> {
    map_pois::table
        .filter(map_pois::map_id.eq(map_id))
        .filter(map_pois::visible.eq(1))
        .load(conn)
}

/// Update a map POI.
pub fn update_map_poi(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateMapPoi,
) -> QueryResult<usize> {
    diesel::update(map_pois::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a map POI by ID.
pub fn delete_map_poi(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(map_pois::table.find(id)).execute(conn)
}

/// Delete all POIs for a map.
pub fn delete_all_map_pois(conn: &mut SqliteConnection, map_id: &str) -> QueryResult<usize> {
    diesel::delete(map_pois::table.filter(map_pois::map_id.eq(map_id))).execute(conn)
}

/// Check if a map POI exists.
pub fn map_poi_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(map_pois::table.find(id))).get_result(conn)
}

/// Count POIs for a map.
pub fn count_map_pois(conn: &mut SqliteConnection, map_id: &str) -> QueryResult<i64> {
    map_pois::table
        .filter(map_pois::map_id.eq(map_id))
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

        let asset = NewCampaignAsset::for_campaign("asset-1", "camp-1", "dungeon.uvtt", "application/octet-stream", "/blobs/dungeon.uvtt");
        insert_campaign_asset(conn, &asset).expect("Failed to create asset");

        let map = NewMap::for_campaign("map-1", "camp-1", "Dungeon", "asset-1");
        insert_map(conn, &map).expect("Failed to create map");
    }

    #[test]
    fn test_insert_and_get_map_poi() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let poi = NewMapPoi::new("poi-1", "map-1", "Secret Door", 5, 10);
        let id = insert_map_poi(&mut conn, &poi).expect("Failed to insert");
        assert_eq!(id, "poi-1");

        let retrieved = get_map_poi(&mut conn, "poi-1").expect("Failed to get");
        assert_eq!(retrieved.name, "Secret Door");
        assert_eq!(retrieved.grid_x, 5);
        assert_eq!(retrieved.grid_y, 10);
        assert!(!retrieved.is_visible());
    }

    #[test]
    fn test_insert_with_options() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let poi = NewMapPoi::new("poi-1", "map-1", "Treasure", 0, 0)
            .with_description("Hidden chest full of gold")
            .with_icon("chest")
            .with_color("#ffcc00")
            .visible();
        insert_map_poi(&mut conn, &poi).expect("Failed to insert");

        let retrieved = get_map_poi(&mut conn, "poi-1").expect("Failed to get");
        assert!(retrieved.description.is_some());
        assert_eq!(retrieved.icon, "chest");
        assert_eq!(retrieved.color, Some("#ffcc00".to_string()));
        assert!(retrieved.is_visible());
    }

    #[test]
    fn test_list_map_pois() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let poi1 = NewMapPoi::new("poi-1", "map-1", "Door", 1, 1);
        let poi2 = NewMapPoi::new("poi-2", "map-1", "Chest", 2, 2);
        insert_map_poi(&mut conn, &poi1).expect("Failed to insert");
        insert_map_poi(&mut conn, &poi2).expect("Failed to insert");

        let pois = list_map_pois(&mut conn, "map-1").expect("Failed to list");
        assert_eq!(pois.len(), 2);
    }

    #[test]
    fn test_list_visible_pois() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let hidden = NewMapPoi::new("poi-1", "map-1", "Hidden", 1, 1);
        let visible = NewMapPoi::new("poi-2", "map-1", "Obvious", 2, 2).visible();
        insert_map_poi(&mut conn, &hidden).expect("Failed to insert");
        insert_map_poi(&mut conn, &visible).expect("Failed to insert");

        let pois = list_visible_map_pois(&mut conn, "map-1").expect("Failed to list");
        assert_eq!(pois.len(), 1);
        assert_eq!(pois[0].name, "Obvious");
    }

    #[test]
    fn test_update_position() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let poi = NewMapPoi::new("poi-1", "map-1", "Marker", 0, 0);
        insert_map_poi(&mut conn, &poi).expect("Failed to insert");

        let update = UpdateMapPoi::set_position(10, 20, "2024-01-20T12:00:00Z");
        update_map_poi(&mut conn, "poi-1", &update).expect("Failed to update");

        let retrieved = get_map_poi(&mut conn, "poi-1").expect("Failed to get");
        assert_eq!(retrieved.grid_x, 10);
        assert_eq!(retrieved.grid_y, 20);
    }

    #[test]
    fn test_update_visible() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let poi = NewMapPoi::new("poi-1", "map-1", "Marker", 0, 0);
        insert_map_poi(&mut conn, &poi).expect("Failed to insert");

        let update = UpdateMapPoi::set_visible(true, "2024-01-20T12:00:00Z");
        update_map_poi(&mut conn, "poi-1", &update).expect("Failed to update");

        let retrieved = get_map_poi(&mut conn, "poi-1").expect("Failed to get");
        assert!(retrieved.is_visible());
    }

    #[test]
    fn test_delete_map_poi() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let poi = NewMapPoi::new("poi-1", "map-1", "Doomed", 0, 0);
        insert_map_poi(&mut conn, &poi).expect("Failed to insert");

        assert!(map_poi_exists(&mut conn, "poi-1").expect("Failed to check"));

        delete_map_poi(&mut conn, "poi-1").expect("Failed to delete");

        assert!(!map_poi_exists(&mut conn, "poi-1").expect("Failed to check"));
    }

    #[test]
    fn test_count_map_pois() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        assert_eq!(
            count_map_pois(&mut conn, "map-1").expect("Failed to count"),
            0
        );

        let poi1 = NewMapPoi::new("poi-1", "map-1", "POI 1", 1, 1);
        let poi2 = NewMapPoi::new("poi-2", "map-1", "POI 2", 2, 2);
        insert_map_poi(&mut conn, &poi1).expect("Failed to insert");
        insert_map_poi(&mut conn, &poi2).expect("Failed to insert");

        assert_eq!(
            count_map_pois(&mut conn, "map-1").expect("Failed to count"),
            2
        );
    }
}
