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
use crate::models::campaign::{
    FogRevealedArea, FogState, LightSource, MapPoi, MapTrap, NewFogRevealedArea, NewLightSource,
    NewMapPoi, NewMapTrap, UpdateLightSource, UpdateMap, UpdateMapPoi, UpdateMapTrap,
};
use crate::services::{ServiceError, ServiceResult};
use crate::utils::now_rfc3339;

/// Input for creating a light source. Coordinates are grid units — any
/// pixel-to-grid conversion is a presentation concern of the caller.
#[derive(Debug, Clone)]
pub struct CreateLightInput {
    /// Map to place the light on.
    pub map_id: String,
    /// Grid X coordinate.
    pub grid_x: i32,
    /// Grid Y coordinate.
    pub grid_y: i32,
    /// Bright light radius in feet.
    pub bright_radius_ft: i32,
    /// Dim light radius in feet.
    pub dim_radius_ft: i32,
    /// Display name.
    pub name: String,
    /// Optional color (hex).
    pub color: Option<String>,
    /// Whether the light starts lit.
    pub is_active: bool,
}

/// Input for updating a light source. Outer `None` = leave unchanged;
/// `Some(None)` on the double-option fields clears the value.
#[derive(Debug, Clone, Default)]
pub struct UpdateLightInput {
    /// New name (Some(None) clears).
    pub name: Option<Option<String>>,
    /// New bright radius in feet.
    pub bright_radius_ft: Option<i32>,
    /// New dim radius in feet.
    pub dim_radius_ft: Option<i32>,
    /// New color (Some(None) clears).
    pub color: Option<Option<String>>,
    /// New active state.
    pub is_active: Option<bool>,
}

/// Input for creating a map trap.
#[derive(Debug, Clone)]
pub struct CreateTrapInput {
    /// Map to place the trap on.
    pub map_id: String,
    /// Trap name.
    pub name: String,
    /// Grid X coordinate.
    pub grid_x: i32,
    /// Grid Y coordinate.
    pub grid_y: i32,
    /// What the trap is.
    pub description: Option<String>,
    /// What sets it off.
    pub trigger_description: Option<String>,
    /// What happens when it fires.
    pub effect_description: Option<String>,
    /// Save/detection DC.
    pub dc: Option<i32>,
    /// Whether players can see it (default hidden).
    pub visible: bool,
}

/// Input for updating a map trap. `None` = leave unchanged.
#[derive(Debug, Clone, Default)]
pub struct UpdateTrapInput {
    /// New name.
    pub name: Option<String>,
    /// New description.
    pub description: Option<String>,
    /// New trigger description.
    pub trigger_description: Option<String>,
    /// New effect description.
    pub effect_description: Option<String>,
    /// New DC.
    pub dc: Option<i32>,
}

/// Input for creating a map POI.
#[derive(Debug, Clone)]
pub struct CreatePoiInput {
    /// Map to place the POI on.
    pub map_id: String,
    /// POI name.
    pub name: String,
    /// Grid X coordinate.
    pub grid_x: i32,
    /// Grid Y coordinate.
    pub grid_y: i32,
    /// Description.
    pub description: Option<String>,
    /// Icon identifier.
    pub icon: Option<String>,
    /// Color (hex).
    pub color: Option<String>,
    /// Whether players can see it (default hidden).
    pub visible: bool,
}

/// Input for updating a map POI. `None` = leave unchanged.
#[derive(Debug, Clone, Default)]
pub struct UpdatePoiInput {
    /// New name.
    pub name: Option<String>,
    /// New description.
    pub description: Option<String>,
    /// New icon.
    pub icon: Option<String>,
    /// New color.
    pub color: Option<String>,
}

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

    // -- Light sources ----------------------------------------------------------

    /// List all light sources for a map.
    pub fn list_lights(&mut self, map_id: &str) -> ServiceResult<Vec<LightSource>> {
        Ok(dal::list_light_sources(self.conn, map_id)?)
    }

    /// Get a light source by id.
    pub fn get_light(&mut self, id: &str) -> ServiceResult<LightSource> {
        dal::get_light_source(self.conn, id).map_err(|_| ServiceError::NotFound {
            entity_type: "LightSource".to_string(),
            id: id.to_string(),
        })
    }

    /// Create a light source.
    pub fn create_light(&mut self, input: CreateLightInput) -> ServiceResult<LightSource> {
        let id = Uuid::new_v4().to_string();

        let mut light = NewLightSource::new(
            &id,
            &input.map_id,
            input.grid_x,
            input.grid_y,
            input.bright_radius_ft,
            input.dim_radius_ft,
        )
        .with_name(&input.name);

        if let Some(ref color) = input.color {
            light = light.with_color(color);
        }
        if !input.is_active {
            light = light.inactive();
        }

        dal::insert_light_source(self.conn, &light)?;
        self.get_light(&id)
    }

    /// Create a torch (20ft bright, 40ft dim).
    pub fn create_torch(&mut self, map_id: &str, x: i32, y: i32) -> ServiceResult<LightSource> {
        let id = Uuid::new_v4().to_string();
        let light = NewLightSource::torch(&id, map_id, x, y);
        dal::insert_light_source(self.conn, &light)?;
        self.get_light(&id)
    }

    /// Create a lantern (30ft bright, 60ft dim).
    pub fn create_lantern(&mut self, map_id: &str, x: i32, y: i32) -> ServiceResult<LightSource> {
        let id = Uuid::new_v4().to_string();
        let light = NewLightSource::lantern(&id, map_id, x, y);
        dal::insert_light_source(self.conn, &light)?;
        self.get_light(&id)
    }

    /// Toggle a light on/off, returning the updated light.
    pub fn toggle_light(&mut self, id: &str) -> ServiceResult<LightSource> {
        let light = self.get_light(id)?;

        let now = now_rfc3339();
        let update = if light.is_active() {
            UpdateLightSource::turn_off(&now)
        } else {
            UpdateLightSource::turn_on(&now)
        };
        dal::update_light_source(self.conn, id, &update)?;

        self.get_light(id)
    }

    /// Update a light source's properties.
    pub fn update_light(
        &mut self,
        id: &str,
        input: UpdateLightInput,
    ) -> ServiceResult<LightSource> {
        let now = now_rfc3339();

        let name: Option<Option<&str>> = input.name.as_ref().map(|inner| inner.as_deref());
        let color: Option<Option<&str>> = input.color.as_ref().map(|inner| inner.as_deref());

        let update = UpdateLightSource {
            grid_x: None,
            grid_y: None,
            name,
            bright_radius: input.bright_radius_ft,
            dim_radius: input.dim_radius_ft,
            color,
            active: input.is_active.map(|a| if a { 1 } else { 0 }),
            updated_at: Some(&now),
        };
        dal::update_light_source(self.conn, id, &update)?;

        self.get_light(id)
    }

    /// Move a light source to a new grid position.
    pub fn move_light(&mut self, id: &str, x: i32, y: i32) -> ServiceResult<LightSource> {
        let now = now_rfc3339();
        dal::update_light_source(self.conn, id, &UpdateLightSource::set_position(x, y, &now))?;
        self.get_light(id)
    }

    /// Delete a light source.
    pub fn delete_light(&mut self, id: &str) -> ServiceResult<()> {
        dal::delete_light_source(self.conn, id)?;
        Ok(())
    }

    /// Delete all light sources on a map. Returns the number removed.
    pub fn delete_all_lights(&mut self, map_id: &str) -> ServiceResult<i32> {
        let count = dal::delete_all_light_sources(self.conn, map_id)?;
        Ok(count as i32)
    }

    // -- Traps ------------------------------------------------------------------

    /// List all traps for a map.
    pub fn list_traps(&mut self, map_id: &str) -> ServiceResult<Vec<MapTrap>> {
        Ok(dal::list_map_traps(self.conn, map_id)?)
    }

    /// List traps visible to players.
    pub fn list_visible_traps(&mut self, map_id: &str) -> ServiceResult<Vec<MapTrap>> {
        Ok(dal::list_visible_map_traps(self.conn, map_id)?)
    }

    /// List armed (untriggered) traps.
    pub fn list_armed_traps(&mut self, map_id: &str) -> ServiceResult<Vec<MapTrap>> {
        Ok(dal::list_armed_map_traps(self.conn, map_id)?)
    }

    /// Get a trap by id.
    pub fn get_trap(&mut self, id: &str) -> ServiceResult<MapTrap> {
        dal::get_map_trap(self.conn, id).map_err(|_| ServiceError::NotFound {
            entity_type: "MapTrap".to_string(),
            id: id.to_string(),
        })
    }

    /// Create a trap.
    pub fn create_trap(&mut self, input: CreateTrapInput) -> ServiceResult<MapTrap> {
        let id = Uuid::new_v4().to_string();
        let mut trap =
            NewMapTrap::new(&id, &input.map_id, &input.name, input.grid_x, input.grid_y);

        if let Some(ref desc) = input.description {
            trap = trap.with_description(desc);
        }
        if let Some(ref trigger) = input.trigger_description {
            trap = trap.with_trigger(trigger);
        }
        if let Some(ref effect) = input.effect_description {
            trap = trap.with_effect(effect);
        }
        if let Some(dc) = input.dc {
            trap = trap.with_dc(dc);
        }
        if input.visible {
            trap = trap.visible();
        }

        dal::insert_map_trap(self.conn, &trap)?;
        self.get_trap(&id)
    }

    /// Update a trap's authoring fields.
    pub fn update_trap(&mut self, id: &str, input: UpdateTrapInput) -> ServiceResult<MapTrap> {
        let now = now_rfc3339();
        let update = UpdateMapTrap {
            name: input.name.as_deref(),
            description: input.description.as_ref().map(|s| Some(s.as_str())),
            trigger_description: input
                .trigger_description
                .as_ref()
                .map(|s| Some(s.as_str())),
            effect_description: input.effect_description.as_ref().map(|s| Some(s.as_str())),
            dc: input.dc.map(Some),
            updated_at: Some(&now),
            ..Default::default()
        };
        dal::update_map_trap(self.conn, id, &update)?;
        self.get_trap(id)
    }

    /// Move a trap to a new grid position.
    pub fn move_trap(&mut self, id: &str, grid_x: i32, grid_y: i32) -> ServiceResult<MapTrap> {
        let now = now_rfc3339();
        dal::update_map_trap(
            self.conn,
            id,
            &UpdateMapTrap::set_position(grid_x, grid_y, &now),
        )?;
        self.get_trap(id)
    }

    /// Toggle a trap's player visibility.
    pub fn toggle_trap_visibility(&mut self, id: &str) -> ServiceResult<MapTrap> {
        let trap = self.get_trap(id)?;
        let now = now_rfc3339();
        dal::update_map_trap(
            self.conn,
            id,
            &UpdateMapTrap::set_visible(!trap.is_visible(), &now),
        )?;
        self.get_trap(id)
    }

    /// Trigger a trap.
    pub fn trigger_trap(&mut self, id: &str) -> ServiceResult<MapTrap> {
        let now = now_rfc3339();
        dal::update_map_trap(self.conn, id, &UpdateMapTrap::trigger(&now))?;
        self.get_trap(id)
    }

    /// Reset (re-arm) a triggered trap.
    pub fn reset_trap(&mut self, id: &str) -> ServiceResult<MapTrap> {
        let now = now_rfc3339();
        dal::update_map_trap(self.conn, id, &UpdateMapTrap::reset(&now))?;
        self.get_trap(id)
    }

    /// Delete a trap.
    pub fn delete_trap(&mut self, id: &str) -> ServiceResult<()> {
        dal::delete_map_trap(self.conn, id)?;
        Ok(())
    }

    // -- Points of interest --------------------------------------------------------

    /// List all POIs for a map.
    pub fn list_pois(&mut self, map_id: &str) -> ServiceResult<Vec<MapPoi>> {
        Ok(dal::list_map_pois(self.conn, map_id)?)
    }

    /// List POIs visible to players.
    pub fn list_visible_pois(&mut self, map_id: &str) -> ServiceResult<Vec<MapPoi>> {
        Ok(dal::list_visible_map_pois(self.conn, map_id)?)
    }

    /// Get a POI by id.
    pub fn get_poi(&mut self, id: &str) -> ServiceResult<MapPoi> {
        dal::get_map_poi(self.conn, id).map_err(|_| ServiceError::NotFound {
            entity_type: "MapPoi".to_string(),
            id: id.to_string(),
        })
    }

    /// Create a POI.
    pub fn create_poi(&mut self, input: CreatePoiInput) -> ServiceResult<MapPoi> {
        let id = Uuid::new_v4().to_string();
        let mut poi = NewMapPoi::new(&id, &input.map_id, &input.name, input.grid_x, input.grid_y);

        if let Some(ref desc) = input.description {
            poi = poi.with_description(desc);
        }
        if let Some(ref icon) = input.icon {
            poi = poi.with_icon(icon);
        }
        if let Some(ref color) = input.color {
            poi = poi.with_color(color);
        }
        if input.visible {
            poi = poi.visible();
        }

        dal::insert_map_poi(self.conn, &poi)?;
        self.get_poi(&id)
    }

    /// Update a POI's authoring fields.
    pub fn update_poi(&mut self, id: &str, input: UpdatePoiInput) -> ServiceResult<MapPoi> {
        let now = now_rfc3339();
        let update = UpdateMapPoi {
            name: input.name.as_deref(),
            description: input.description.as_ref().map(|s| Some(s.as_str())),
            icon: input.icon.as_deref(),
            color: input.color.as_ref().map(|s| Some(s.as_str())),
            updated_at: Some(&now),
            ..Default::default()
        };
        dal::update_map_poi(self.conn, id, &update)?;
        self.get_poi(id)
    }

    /// Move a POI to a new grid position.
    pub fn move_poi(&mut self, id: &str, grid_x: i32, grid_y: i32) -> ServiceResult<MapPoi> {
        let now = now_rfc3339();
        dal::update_map_poi(
            self.conn,
            id,
            &UpdateMapPoi::set_position(grid_x, grid_y, &now),
        )?;
        self.get_poi(id)
    }

    /// Toggle a POI's player visibility.
    pub fn toggle_poi_visibility(&mut self, id: &str) -> ServiceResult<MapPoi> {
        let poi = self.get_poi(id)?;
        let now = now_rfc3339();
        dal::update_map_poi(
            self.conn,
            id,
            &UpdateMapPoi::set_visible(!poi.is_visible(), &now),
        )?;
        self.get_poi(id)
    }

    /// Delete a POI.
    pub fn delete_poi(&mut self, id: &str) -> ServiceResult<()> {
        dal::delete_map_poi(self.conn, id)?;
        Ok(())
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
    fn torch_and_lantern_presets_are_pinned() {
        let mut conn = test_connection();
        let map_id = setup_map(&mut conn);
        let mut svc = MapStateService::new(&mut conn);

        let torch = svc.create_torch(&map_id, 3, 4).unwrap();
        assert_eq!(torch.bright_radius, 20);
        assert_eq!(torch.dim_radius, 40);
        assert_eq!((torch.grid_x, torch.grid_y), (3, 4));
        assert!(torch.is_active());

        let lantern = svc.create_lantern(&map_id, 5, 6).unwrap();
        assert_eq!(lantern.bright_radius, 30);
        assert_eq!(lantern.dim_radius, 60);
    }

    #[test]
    fn light_toggle_round_trip() {
        let mut conn = test_connection();
        let map_id = setup_map(&mut conn);
        let mut svc = MapStateService::new(&mut conn);

        let light = svc
            .create_light(CreateLightInput {
                map_id: map_id.clone(),
                grid_x: 1,
                grid_y: 1,
                bright_radius_ft: 15,
                dim_radius_ft: 30,
                name: "Candle".to_string(),
                color: Some("#FFAA00".to_string()),
                is_active: true,
            })
            .unwrap();
        assert!(light.is_active());

        let off = svc.toggle_light(&light.id).unwrap();
        assert!(!off.is_active());
        let on = svc.toggle_light(&light.id).unwrap();
        assert!(on.is_active());
    }

    #[test]
    fn light_update_and_move() {
        let mut conn = test_connection();
        let map_id = setup_map(&mut conn);
        let mut svc = MapStateService::new(&mut conn);

        let light = svc
            .create_light(CreateLightInput {
                map_id: map_id.clone(),
                grid_x: 0,
                grid_y: 0,
                bright_radius_ft: 10,
                dim_radius_ft: 20,
                name: "Brazier".to_string(),
                color: None,
                is_active: false,
            })
            .unwrap();
        assert!(!light.is_active());

        let updated = svc
            .update_light(
                &light.id,
                UpdateLightInput {
                    bright_radius_ft: Some(25),
                    is_active: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(updated.bright_radius, 25);
        assert_eq!(updated.dim_radius, 20, "unspecified field unchanged");
        assert!(updated.is_active());

        let moved = svc.move_light(&light.id, 7, 9).unwrap();
        assert_eq!((moved.grid_x, moved.grid_y), (7, 9));
    }

    #[test]
    fn delete_all_lights_reports_count() {
        let mut conn = test_connection();
        let map_id = setup_map(&mut conn);
        let mut svc = MapStateService::new(&mut conn);

        svc.create_torch(&map_id, 0, 0).unwrap();
        svc.create_lantern(&map_id, 1, 1).unwrap();
        assert_eq!(svc.list_lights(&map_id).unwrap().len(), 2);

        assert_eq!(svc.delete_all_lights(&map_id).unwrap(), 2);
        assert!(svc.list_lights(&map_id).unwrap().is_empty());
    }

    #[test]
    fn trap_lifecycle_create_trigger_reset() {
        let mut conn = test_connection();
        let map_id = setup_map(&mut conn);
        let mut svc = MapStateService::new(&mut conn);

        let trap = svc
            .create_trap(CreateTrapInput {
                map_id: map_id.clone(),
                name: "Pit Trap".to_string(),
                grid_x: 4,
                grid_y: 5,
                description: Some("A concealed pit".to_string()),
                trigger_description: Some("Stepping on the cover".to_string()),
                effect_description: Some("2d6 falling damage".to_string()),
                dc: Some(15),
                visible: false,
            })
            .unwrap();
        assert!(!trap.is_visible());
        assert!(!trap.is_triggered());
        assert_eq!(trap.dc, Some(15));

        let fired = svc.trigger_trap(&trap.id).unwrap();
        assert!(fired.is_triggered());

        let rearmed = svc.reset_trap(&trap.id).unwrap();
        assert!(!rearmed.is_triggered());

        // Armed list reflects state
        assert_eq!(svc.list_armed_traps(&map_id).unwrap().len(), 1);
        svc.trigger_trap(&trap.id).unwrap();
        assert!(svc.list_armed_traps(&map_id).unwrap().is_empty());
    }

    #[test]
    fn trap_visibility_toggle_and_update() {
        let mut conn = test_connection();
        let map_id = setup_map(&mut conn);
        let mut svc = MapStateService::new(&mut conn);

        let trap = svc
            .create_trap(CreateTrapInput {
                map_id: map_id.clone(),
                name: "Dart Trap".to_string(),
                grid_x: 0,
                grid_y: 0,
                description: None,
                trigger_description: None,
                effect_description: None,
                dc: None,
                visible: false,
            })
            .unwrap();

        assert!(svc.list_visible_traps(&map_id).unwrap().is_empty());
        let shown = svc.toggle_trap_visibility(&trap.id).unwrap();
        assert!(shown.is_visible());
        assert_eq!(svc.list_visible_traps(&map_id).unwrap().len(), 1);

        let updated = svc
            .update_trap(
                &trap.id,
                UpdateTrapInput {
                    name: Some("Poison Dart Trap".to_string()),
                    dc: Some(13),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(updated.name, "Poison Dart Trap");
        assert_eq!(updated.dc, Some(13));

        let moved = svc.move_trap(&trap.id, 8, 2).unwrap();
        assert_eq!((moved.grid_x, moved.grid_y), (8, 2));

        svc.delete_trap(&trap.id).unwrap();
        assert!(svc.list_traps(&map_id).unwrap().is_empty());
    }

    #[test]
    fn poi_crud_and_visibility() {
        let mut conn = test_connection();
        let map_id = setup_map(&mut conn);
        let mut svc = MapStateService::new(&mut conn);

        let poi = svc
            .create_poi(CreatePoiInput {
                map_id: map_id.clone(),
                name: "Hidden Shrine".to_string(),
                grid_x: 12,
                grid_y: 3,
                description: Some("An old shrine to a forgotten god".to_string()),
                icon: Some("shrine".to_string()),
                color: Some("#88CCFF".to_string()),
                visible: false,
            })
            .unwrap();
        assert!(!poi.is_visible());

        let updated = svc
            .update_poi(
                &poi.id,
                UpdatePoiInput {
                    name: Some("Shrine of Echoes".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(updated.name, "Shrine of Echoes");
        assert_eq!(updated.icon, "shrine", "unspecified field unchanged");

        let moved = svc.move_poi(&poi.id, 1, 1).unwrap();
        assert_eq!((moved.grid_x, moved.grid_y), (1, 1));

        assert!(svc.list_visible_pois(&map_id).unwrap().is_empty());
        svc.toggle_poi_visibility(&poi.id).unwrap();
        assert_eq!(svc.list_visible_pois(&map_id).unwrap().len(), 1);

        svc.delete_poi(&poi.id).unwrap();
        assert!(svc.list_pois(&map_id).unwrap().is_empty());
    }

    #[test]
    fn missing_trap_and_poi_are_not_found() {
        let mut conn = test_connection();
        let mut svc = MapStateService::new(&mut conn);

        assert!(matches!(
            svc.get_trap("nope"),
            Err(ServiceError::NotFound { .. })
        ));
        assert!(matches!(
            svc.get_poi("nope"),
            Err(ServiceError::NotFound { .. })
        ));
    }

    #[test]
    fn toggle_missing_light_is_not_found() {
        let mut conn = test_connection();
        let mut svc = MapStateService::new(&mut conn);

        assert!(matches!(
            svc.toggle_light("no-such-light"),
            Err(ServiceError::NotFound { .. })
        ));
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
