//! Fog of War DAL
//!
//! Database operations for fog of war revealed areas.

use diesel::prelude::*;

use crate::models::campaign::{FogRevealedArea, NewFogRevealedArea};
use crate::schema::fog_revealed_areas;

/// Insert a new revealed area.
pub fn insert_fog_revealed_area(
    conn: &mut SqliteConnection,
    area: &NewFogRevealedArea,
) -> QueryResult<String> {
    diesel::insert_into(fog_revealed_areas::table)
        .values(area)
        .execute(conn)?;
    Ok(area.id.to_string())
}

/// Get a revealed area by ID.
pub fn get_fog_revealed_area(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<FogRevealedArea> {
    fog_revealed_areas::table.find(id).first(conn)
}

/// List all revealed areas for a map.
pub fn list_fog_revealed_areas(
    conn: &mut SqliteConnection,
    map_id: &str,
) -> QueryResult<Vec<FogRevealedArea>> {
    fog_revealed_areas::table
        .filter(fog_revealed_areas::map_id.eq(map_id))
        .order(fog_revealed_areas::created_at.asc())
        .load(conn)
}

/// Delete a revealed area.
pub fn delete_fog_revealed_area(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<usize> {
    diesel::delete(fog_revealed_areas::table.find(id)).execute(conn)
}

/// Delete all revealed areas for a map.
pub fn delete_all_fog_revealed_areas(
    conn: &mut SqliteConnection,
    map_id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        fog_revealed_areas::table.filter(fog_revealed_areas::map_id.eq(map_id))
    ).execute(conn)
}

/// Count revealed areas for a map.
pub fn count_fog_revealed_areas(
    conn: &mut SqliteConnection,
    map_id: &str,
) -> QueryResult<i64> {
    fog_revealed_areas::table
        .filter(fog_revealed_areas::map_id.eq(map_id))
        .count()
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::campaign::{insert_campaign, insert_campaign_asset, insert_map};
    use crate::models::campaign::{NewCampaign, NewCampaignAsset, NewMap};

    fn setup_test_map(conn: &mut SqliteConnection) -> String {
        let campaign = NewCampaign::new("camp-test", "Test Campaign", "Default");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");

        let asset = NewCampaignAsset::new("asset-test", "camp-test", "test.uvtt", "application/octet-stream", "/path/to/test.uvtt");
        insert_campaign_asset(conn, &asset).expect("Failed to create asset");

        let map = NewMap::for_campaign("map-test", "camp-test", "Test Map", "asset-test");
        insert_map(conn, &map).expect("Failed to create map");

        "map-test".to_string()
    }

    #[test]
    fn test_insert_and_get_revealed_area() {
        let mut conn = test_connection();
        let map_id = setup_test_map(&mut conn);

        let area = NewFogRevealedArea::new("area-1", &map_id, 10.0, 20.0, 5.0, 5.0);
        insert_fog_revealed_area(&mut conn, &area).expect("Failed to insert");

        let retrieved = get_fog_revealed_area(&mut conn, "area-1").expect("Failed to get");
        assert_eq!(retrieved.x, 10.0);
        assert_eq!(retrieved.y, 20.0);
        assert_eq!(retrieved.width, 5.0);
        assert_eq!(retrieved.height, 5.0);
    }

    #[test]
    fn test_list_revealed_areas() {
        let mut conn = test_connection();
        let map_id = setup_test_map(&mut conn);

        let area1 = NewFogRevealedArea::new("area-1", &map_id, 0.0, 0.0, 5.0, 5.0);
        let area2 = NewFogRevealedArea::new("area-2", &map_id, 10.0, 10.0, 5.0, 5.0);
        insert_fog_revealed_area(&mut conn, &area1).expect("Failed to insert");
        insert_fog_revealed_area(&mut conn, &area2).expect("Failed to insert");

        let areas = list_fog_revealed_areas(&mut conn, &map_id).expect("Failed to list");
        assert_eq!(areas.len(), 2);
    }

    #[test]
    fn test_delete_revealed_area() {
        let mut conn = test_connection();
        let map_id = setup_test_map(&mut conn);

        let area = NewFogRevealedArea::new("area-1", &map_id, 10.0, 20.0, 5.0, 5.0);
        insert_fog_revealed_area(&mut conn, &area).expect("Failed to insert");

        let deleted = delete_fog_revealed_area(&mut conn, "area-1").expect("Failed to delete");
        assert_eq!(deleted, 1);

        let areas = list_fog_revealed_areas(&mut conn, &map_id).expect("Failed to list");
        assert!(areas.is_empty());
    }

    #[test]
    fn test_delete_all_revealed_areas() {
        let mut conn = test_connection();
        let map_id = setup_test_map(&mut conn);

        let area1 = NewFogRevealedArea::new("area-1", &map_id, 0.0, 0.0, 5.0, 5.0);
        let area2 = NewFogRevealedArea::new("area-2", &map_id, 10.0, 10.0, 5.0, 5.0);
        insert_fog_revealed_area(&mut conn, &area1).expect("Failed to insert");
        insert_fog_revealed_area(&mut conn, &area2).expect("Failed to insert");

        let deleted = delete_all_fog_revealed_areas(&mut conn, &map_id).expect("Failed to delete");
        assert_eq!(deleted, 2);

        let areas = list_fog_revealed_areas(&mut conn, &map_id).expect("Failed to list");
        assert!(areas.is_empty());
    }
}
