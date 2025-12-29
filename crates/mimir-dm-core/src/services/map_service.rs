//! Map management service for Visual Display System.
//!
//! Provides business logic for managing battle maps, dungeon maps, and
//! regional maps for visual display during in-person play sessions.

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::campaign::{Map, MapSummary, NewMap, UpdateMap};
use crate::schema::maps;
use diesel::prelude::*;

/// Service for managing maps
pub struct MapService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> MapService<'a> {
    /// Create a new map service.
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new map.
    ///
    /// # Arguments
    /// * `new_map` - The map data to insert
    ///
    /// # Returns
    /// * `Ok(Map)` - The created map record
    pub fn create_map(&mut self, new_map: NewMap) -> Result<Map> {
        diesel::insert_into(maps::table)
            .values(&new_map)
            .returning(Map::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Get a map by ID.
    ///
    /// # Arguments
    /// * `id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(Some(Map))` - If found
    /// * `Ok(None)` - If no map exists with that ID
    pub fn get_map(&mut self, id: i32) -> Result<Option<Map>> {
        maps::table
            .find(id)
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// List all campaign-level maps (not tied to a module).
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    ///
    /// # Returns
    /// * `Ok(Vec<Map>)` - Campaign-level maps
    pub fn list_campaign_maps(&mut self, campaign_id: i32) -> Result<Vec<Map>> {
        maps::table
            .filter(maps::campaign_id.eq(campaign_id))
            .filter(maps::module_id.is_null())
            .order(maps::name.asc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List all maps for a specific module.
    ///
    /// # Arguments
    /// * `module_id` - Database ID of the module
    ///
    /// # Returns
    /// * `Ok(Vec<Map>)` - Maps associated with the module
    pub fn list_module_maps(&mut self, module_id: i32) -> Result<Vec<Map>> {
        maps::table
            .filter(maps::module_id.eq(module_id))
            .order(maps::name.asc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List all maps for a campaign (both campaign-level and module-level).
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    ///
    /// # Returns
    /// * `Ok(Vec<Map>)` - All maps in the campaign
    pub fn list_all_campaign_maps(&mut self, campaign_id: i32) -> Result<Vec<Map>> {
        maps::table
            .filter(maps::campaign_id.eq(campaign_id))
            .order(maps::name.asc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Get map summaries for a campaign with module names.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    ///
    /// # Returns
    /// * `Ok(Vec<MapSummary>)` - Map summaries with module info
    pub fn list_map_summaries(&mut self, campaign_id: i32) -> Result<Vec<MapSummary>> {
        use crate::schema::modules;

        // Get all maps for the campaign
        let all_maps: Vec<Map> = maps::table
            .filter(maps::campaign_id.eq(campaign_id))
            .order(maps::name.asc())
            .load(self.conn)?;

        // Get module names for maps that have module_id
        let module_ids: Vec<i32> = all_maps
            .iter()
            .filter_map(|m| m.module_id)
            .collect();

        let module_names: Vec<(i32, String)> = if !module_ids.is_empty() {
            modules::table
                .filter(modules::id.eq_any(&module_ids))
                .select((modules::id, modules::name))
                .load(self.conn)?
        } else {
            vec![]
        };

        // Build summaries
        let summaries = all_maps
            .into_iter()
            .map(|m| {
                let module_name = m.module_id.and_then(|mid| {
                    module_names
                        .iter()
                        .find(|(id, _)| *id == mid)
                        .map(|(_, name)| name.clone())
                });

                MapSummary {
                    id: m.id,
                    name: m.name,
                    module_id: m.module_id,
                    module_name,
                    grid_type: m.grid_type,
                    grid_size_px: m.grid_size_px,
                    grid_offset_x: m.grid_offset_x,
                    grid_offset_y: m.grid_offset_y,
                    width_px: m.width_px,
                    height_px: m.height_px,
                    original_width_px: m.original_width_px,
                    original_height_px: m.original_height_px,
                    fog_enabled: m.fog_enabled,
                    ambient_light: m.ambient_light,
                    image_path: m.image_path,
                }
            })
            .collect();

        Ok(summaries)
    }

    /// Update a map.
    ///
    /// # Arguments
    /// * `id` - Database ID of the map
    /// * `update` - Fields to update
    ///
    /// # Returns
    /// * `Ok(Map)` - The updated map
    pub fn update_map(&mut self, id: i32, mut update: UpdateMap) -> Result<Map> {
        // Set updated_at timestamp
        update.updated_at = Some(chrono::Utc::now().to_rfc3339());

        diesel::update(maps::table.find(id))
            .set(&update)
            .returning(Map::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Delete a map.
    ///
    /// Note: This only deletes the database record. The caller is responsible
    /// for deleting the associated image file.
    ///
    /// # Arguments
    /// * `id` - Database ID of the map to delete
    ///
    /// # Returns
    /// * `Ok(())` - If deletion succeeds
    pub fn delete_map(&mut self, id: i32) -> Result<()> {
        diesel::delete(maps::table.find(id))
            .execute(self.conn)?;
        Ok(())
    }

    /// Update grid configuration for a map.
    ///
    /// Convenience method for updating just the grid settings.
    ///
    /// # Arguments
    /// * `id` - Database ID of the map
    /// * `grid_type` - Grid type ("square", "hex", or "none")
    /// * `grid_size_px` - Pixels per grid cell (None to remove grid)
    /// * `offset_x` - Grid X offset for alignment
    /// * `offset_y` - Grid Y offset for alignment
    pub fn update_grid_config(
        &mut self,
        id: i32,
        grid_type: &str,
        grid_size_px: Option<i32>,
        offset_x: i32,
        offset_y: i32,
    ) -> Result<Map> {
        let update = UpdateMap {
            grid_type: Some(grid_type.to_string()),
            grid_size_px: Some(grid_size_px),
            grid_offset_x: Some(offset_x),
            grid_offset_y: Some(offset_y),
            ..Default::default()
        };

        self.update_map(id, update)
    }
}
