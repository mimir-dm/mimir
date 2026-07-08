//! Map State Service
//!
//! Business logic for table-state on maps: fog of war, light sources, traps,
//! and points of interest. This is the shared implementation behind both the
//! desktop UI and (for authoring operations only) the MCP server.
//!
//! Unlike `MapService`, this service takes no `app_data_dir` — table-state
//! never touches asset files.

use diesel::SqliteConnection;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{FogRevealedArea, FogState, UpdateMap, NewFogRevealedArea};
use crate::services::{ServiceError, ServiceResult};
use crate::utils::now_rfc3339;

/// Service for map table-state: fog, lights, traps, POIs.
pub struct MapStateService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> MapStateService<'a> {
    /// Create a new map state service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    // -- Fog of war -----------------------------------------------------------

    /// Get the fog state for a map: enabled flag plus all revealed areas.
    pub fn fog_state(&mut self, map_id: &str) -> ServiceResult<FogState> {
        let map = dal::get_map(self.conn, map_id).map_err(|_| ServiceError::NotFound {
            entity_type: "Map".to_string(),
            id: map_id.to_string(),
        })?;

        let revealed_areas = dal::list_fog_revealed_areas(self.conn, map_id)?;

        Ok(FogState::new(
            map_id.to_string(),
            map.is_fog_enabled(),
            revealed_areas,
        ))
    }

    /// Enable fog of war for a map.
    pub fn enable_fog(&mut self, map_id: &str) -> ServiceResult<()> {
        let now = now_rfc3339();
        dal::update_map(self.conn, map_id, &UpdateMap::enable_fog(&now))?;
        Ok(())
    }

    /// Disable fog of war for a map.
    pub fn disable_fog(&mut self, map_id: &str) -> ServiceResult<()> {
        let now = now_rfc3339();
        dal::update_map(self.conn, map_id, &UpdateMap::disable_fog(&now))?;
        Ok(())
    }

    /// Toggle fog of war for a map, returning the new enabled state.
    pub fn toggle_fog(&mut self, map_id: &str) -> ServiceResult<bool> {
        let map = dal::get_map(self.conn, map_id).map_err(|_| ServiceError::NotFound {
            entity_type: "Map".to_string(),
            id: map_id.to_string(),
        })?;

        let new_enabled = !map.is_fog_enabled();
        let now = now_rfc3339();
        dal::update_map(
            self.conn,
            map_id,
            &UpdateMap::set_fog_enabled(new_enabled, &now),
        )?;

        Ok(new_enabled)
    }

    /// Reveal a rectangular area on the map.
    pub fn reveal_rect(
        &mut self,
        map_id: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> ServiceResult<FogRevealedArea> {
        let id = Uuid::new_v4().to_string();
        let area = NewFogRevealedArea::rect(&id, map_id, x, y, width, height);
        dal::insert_fog_revealed_area(self.conn, &area)?;
        Ok(dal::get_fog_revealed_area(self.conn, &id)?)
    }

    /// Reveal a circular area on the map (stored as its bounding box).
    pub fn reveal_circle(
        &mut self,
        map_id: &str,
        center_x: f64,
        center_y: f64,
        radius: f64,
    ) -> ServiceResult<FogRevealedArea> {
        let id = Uuid::new_v4().to_string();
        let area = NewFogRevealedArea::circle(&id, map_id, center_x, center_y, radius);
        dal::insert_fog_revealed_area(self.conn, &area)?;
        Ok(dal::get_fog_revealed_area(self.conn, &id)?)
    }

    /// Reveal the entire map (one rect covering the full dimensions).
    pub fn reveal_all(
        &mut self,
        map_id: &str,
        width: f64,
        height: f64,
    ) -> ServiceResult<FogRevealedArea> {
        self.reveal_rect(map_id, 0.0, 0.0, width, height)
    }

    /// Delete a single revealed area.
    pub fn delete_revealed_area(&mut self, id: &str) -> ServiceResult<()> {
        dal::delete_fog_revealed_area(self.conn, id)?;
        Ok(())
    }

    /// Reset fog by clearing all revealed areas for a map. Returns the number
    /// of areas removed.
    pub fn reset_fog(&mut self, map_id: &str) -> ServiceResult<i32> {
        let count = dal::delete_all_fog_revealed_areas(self.conn, map_id)?;
        Ok(count as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::campaign::{insert_campaign, insert_campaign_asset, insert_map};
    use crate::db::test_connection;
    use crate::models::campaign::{NewCampaign, NewCampaignAsset, NewMap};

    /// Create campaign + asset + map; returns the map id.
    fn setup_map(conn: &mut SqliteConnection) -> String {
        let campaign_id = Uuid::new_v4().to_string();
        insert_campaign(conn, &NewCampaign::new(&campaign_id, "Test Campaign"))
            .expect("insert campaign");

        let asset_id = Uuid::new_v4().to_string();
        let asset = NewCampaignAsset {
            id: &asset_id,
            campaign_id: Some(&campaign_id),
            module_id: None,
            filename: "test.uvtt",
            description: None,
            mime_type: "application/json",
            blob_path: "assets/test.uvtt",
            file_size: None,
        };
        insert_campaign_asset(conn, &asset).expect("insert asset");

        let map_id = Uuid::new_v4().to_string();
        let map = NewMap {
            id: &map_id,
            campaign_id: &campaign_id,
            module_id: None,
            name: "Test Map",
            description: None,
            sort_order: 0,
            uvtt_asset_id: &asset_id,
            lighting_mode: "bright",
            fog_enabled: 0,
        };
        insert_map(conn, &map).expect("insert map");

        map_id
    }

    #[test]
    fn toggle_fog_round_trip() {
        let mut conn = test_connection();
        let map_id = setup_map(&mut conn);
        let mut svc = MapStateService::new(&mut conn);

        assert!(!svc.fog_state(&map_id).unwrap().fog_enabled);
        assert!(svc.toggle_fog(&map_id).unwrap());
        assert!(svc.fog_state(&map_id).unwrap().fog_enabled);
        assert!(!svc.toggle_fog(&map_id).unwrap());
        assert!(!svc.fog_state(&map_id).unwrap().fog_enabled);
    }

    #[test]
    fn enable_and_disable_fog() {
        let mut conn = test_connection();
        let map_id = setup_map(&mut conn);
        let mut svc = MapStateService::new(&mut conn);

        svc.enable_fog(&map_id).unwrap();
        assert!(svc.fog_state(&map_id).unwrap().fog_enabled);
        svc.disable_fog(&map_id).unwrap();
        assert!(!svc.fog_state(&map_id).unwrap().fog_enabled);
    }

    #[test]
    fn reveal_shapes_persist_and_circle_becomes_bounding_box() {
        let mut conn = test_connection();
        let map_id = setup_map(&mut conn);
        let mut svc = MapStateService::new(&mut conn);

        let rect = svc.reveal_rect(&map_id, 1.0, 2.0, 3.0, 4.0).unwrap();
        assert_eq!((rect.x, rect.y, rect.width, rect.height), (1.0, 2.0, 3.0, 4.0));

        let circle = svc.reveal_circle(&map_id, 10.0, 10.0, 5.0).unwrap();
        assert_eq!(
            (circle.x, circle.y, circle.width, circle.height),
            (5.0, 5.0, 10.0, 10.0)
        );

        let all = svc.reveal_all(&map_id, 100.0, 80.0).unwrap();
        assert_eq!((all.x, all.y, all.width, all.height), (0.0, 0.0, 100.0, 80.0));

        assert_eq!(svc.fog_state(&map_id).unwrap().revealed_areas.len(), 3);
    }

    #[test]
    fn reset_fog_clears_all_areas_and_reports_count() {
        let mut conn = test_connection();
        let map_id = setup_map(&mut conn);
        let mut svc = MapStateService::new(&mut conn);

        svc.reveal_rect(&map_id, 0.0, 0.0, 1.0, 1.0).unwrap();
        svc.reveal_rect(&map_id, 2.0, 2.0, 1.0, 1.0).unwrap();

        assert_eq!(svc.reset_fog(&map_id).unwrap(), 2);
        assert!(svc.fog_state(&map_id).unwrap().revealed_areas.is_empty());
    }

    #[test]
    fn delete_single_revealed_area() {
        let mut conn = test_connection();
        let map_id = setup_map(&mut conn);
        let mut svc = MapStateService::new(&mut conn);

        let keep = svc.reveal_rect(&map_id, 0.0, 0.0, 1.0, 1.0).unwrap();
        let drop = svc.reveal_rect(&map_id, 2.0, 2.0, 1.0, 1.0).unwrap();

        svc.delete_revealed_area(&drop.id).unwrap();

        let areas = svc.fog_state(&map_id).unwrap().revealed_areas;
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].id, keep.id);
    }

    #[test]
    fn fog_state_for_missing_map_is_not_found() {
        let mut conn = test_connection();
        let mut svc = MapStateService::new(&mut conn);

        assert!(matches!(
            svc.fog_state("no-such-map"),
            Err(ServiceError::NotFound { .. })
        ));
        assert!(matches!(
            svc.toggle_fog("no-such-map"),
            Err(ServiceError::NotFound { .. })
        ));
    }
}
