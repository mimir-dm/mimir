//! Light Source service for Vision and Lighting System.
//!
//! Manages light sources on maps - torches, lanterns, spells, and other
//! illumination that affects visibility for tokens with different vision types.

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::campaign::{
    LightSource, LightSourceSummary, LightType, NewLightSource, UpdateLightSource,
};
use crate::schema::{light_sources, tokens};
use diesel::prelude::*;

/// Service for managing light sources
pub struct LightSourceService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> LightSourceService<'a> {
    /// Create a new light source service.
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new light source on a map.
    ///
    /// # Arguments
    /// * `new_light` - The light source to create
    ///
    /// # Returns
    /// * `Ok(LightSource)` - The created light source
    pub fn create_light_source(&mut self, new_light: NewLightSource) -> Result<LightSource> {
        diesel::insert_into(light_sources::table)
            .values(&new_light)
            .returning(LightSource::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Create a torch at a position.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    /// * `x` - X coordinate in pixels
    /// * `y` - Y coordinate in pixels
    ///
    /// # Returns
    /// * `Ok(LightSource)` - The created torch
    pub fn create_torch(&mut self, map_id: i32, x: f32, y: f32) -> Result<LightSource> {
        let new_light = NewLightSource::torch(map_id, x, y);
        self.create_light_source(new_light)
    }

    /// Create a lantern at a position.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    /// * `x` - X coordinate in pixels
    /// * `y` - Y coordinate in pixels
    ///
    /// # Returns
    /// * `Ok(LightSource)` - The created lantern
    pub fn create_lantern(&mut self, map_id: i32, x: f32, y: f32) -> Result<LightSource> {
        let new_light = NewLightSource::lantern(map_id, x, y);
        self.create_light_source(new_light)
    }

    /// Create a light source attached to a token.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    /// * `token_id` - Database ID of the token to attach to
    /// * `name` - Name of the light source
    /// * `light_type` - Type of light (torch, lantern, etc.)
    ///
    /// # Returns
    /// * `Ok(LightSource)` - The created light source
    pub fn create_attached_light(
        &mut self,
        map_id: i32,
        token_id: i32,
        name: String,
        light_type: LightType,
    ) -> Result<LightSource> {
        let new_light = NewLightSource::attached_to_token(map_id, token_id, name, light_type);
        self.create_light_source(new_light)
    }

    /// Get a light source by ID.
    ///
    /// # Arguments
    /// * `id` - Database ID of the light source
    ///
    /// # Returns
    /// * `Ok(LightSource)` - The light source
    pub fn get_light_source(&mut self, id: i32) -> Result<LightSource> {
        light_sources::table
            .find(id)
            .first(self.conn)
            .map_err(Into::into)
    }

    /// Get all light sources for a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(Vec<LightSource>)` - All light sources on the map
    pub fn get_light_sources_for_map(&mut self, map_id: i32) -> Result<Vec<LightSource>> {
        light_sources::table
            .filter(light_sources::map_id.eq(map_id))
            .order(light_sources::id.asc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Get all active light sources for a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(Vec<LightSource>)` - All active light sources on the map
    pub fn get_active_light_sources(&mut self, map_id: i32) -> Result<Vec<LightSource>> {
        light_sources::table
            .filter(light_sources::map_id.eq(map_id))
            .filter(light_sources::is_active.eq(true))
            .order(light_sources::id.asc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Get light source summaries for a map (includes token info).
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(Vec<LightSourceSummary>)` - Summaries of all light sources
    pub fn get_light_source_summaries(&mut self, map_id: i32) -> Result<Vec<LightSourceSummary>> {
        let lights = self.get_light_sources_for_map(map_id)?;

        // Get token names for attached lights
        let token_ids: Vec<i32> = lights
            .iter()
            .filter_map(|l| l.token_id)
            .collect();

        let token_names: Vec<(i32, String)> = if token_ids.is_empty() {
            vec![]
        } else {
            tokens::table
                .filter(tokens::id.eq_any(&token_ids))
                .select((tokens::id, tokens::name))
                .load(self.conn)?
        };

        let summaries = lights
            .into_iter()
            .map(|l| {
                let token_name = l.token_id.and_then(|tid| {
                    token_names
                        .iter()
                        .find(|(id, _)| *id == tid)
                        .map(|(_, name)| name.clone())
                });

                LightSourceSummary {
                    id: l.id,
                    map_id: l.map_id,
                    token_id: l.token_id,
                    token_name,
                    name: l.name,
                    light_type: l.light_type,
                    x: l.x,
                    y: l.y,
                    bright_radius_ft: l.bright_radius_ft,
                    dim_radius_ft: l.dim_radius_ft,
                    color: l.color,
                    is_active: l.is_active,
                }
            })
            .collect();

        Ok(summaries)
    }

    /// Get light sources attached to a specific token.
    ///
    /// # Arguments
    /// * `token_id` - Database ID of the token
    ///
    /// # Returns
    /// * `Ok(Vec<LightSource>)` - Light sources attached to the token
    pub fn get_lights_for_token(&mut self, token_id: i32) -> Result<Vec<LightSource>> {
        light_sources::table
            .filter(light_sources::token_id.eq(token_id))
            .order(light_sources::id.asc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Update a light source.
    ///
    /// # Arguments
    /// * `id` - Database ID of the light source
    /// * `update` - Fields to update
    ///
    /// # Returns
    /// * `Ok(LightSource)` - The updated light source
    pub fn update_light_source(
        &mut self,
        id: i32,
        mut update: UpdateLightSource,
    ) -> Result<LightSource> {
        if update.updated_at.is_none() {
            update.updated_at = Some(chrono::Utc::now().to_rfc3339());
        }

        diesel::update(light_sources::table.find(id))
            .set(&update)
            .returning(LightSource::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Move a light source to a new position.
    ///
    /// # Arguments
    /// * `id` - Database ID of the light source
    /// * `x` - New X coordinate in pixels
    /// * `y` - New Y coordinate in pixels
    ///
    /// # Returns
    /// * `Ok(LightSource)` - The updated light source
    pub fn move_light_source(&mut self, id: i32, x: f32, y: f32) -> Result<LightSource> {
        let update = UpdateLightSource::position(x, y);
        self.update_light_source(id, update)
    }

    /// Toggle a light source active state.
    ///
    /// # Arguments
    /// * `id` - Database ID of the light source
    ///
    /// # Returns
    /// * `Ok(LightSource)` - The updated light source
    pub fn toggle_light_source(&mut self, id: i32) -> Result<LightSource> {
        // Get current state
        let current: bool = light_sources::table
            .find(id)
            .select(light_sources::is_active)
            .first(self.conn)?;

        let update = UpdateLightSource::toggle_active(!current);
        self.update_light_source(id, update)
    }

    /// Activate a light source.
    ///
    /// # Arguments
    /// * `id` - Database ID of the light source
    ///
    /// # Returns
    /// * `Ok(LightSource)` - The updated light source
    pub fn activate_light_source(&mut self, id: i32) -> Result<LightSource> {
        let update = UpdateLightSource::toggle_active(true);
        self.update_light_source(id, update)
    }

    /// Deactivate a light source.
    ///
    /// # Arguments
    /// * `id` - Database ID of the light source
    ///
    /// # Returns
    /// * `Ok(LightSource)` - The updated light source
    pub fn deactivate_light_source(&mut self, id: i32) -> Result<LightSource> {
        let update = UpdateLightSource::toggle_active(false);
        self.update_light_source(id, update)
    }

    /// Update light source radii.
    ///
    /// # Arguments
    /// * `id` - Database ID of the light source
    /// * `bright_ft` - Bright light radius in feet
    /// * `dim_ft` - Dim light radius in feet
    ///
    /// # Returns
    /// * `Ok(LightSource)` - The updated light source
    pub fn update_radii(&mut self, id: i32, bright_ft: f32, dim_ft: f32) -> Result<LightSource> {
        let update = UpdateLightSource::radii(bright_ft, dim_ft);
        self.update_light_source(id, update)
    }

    /// Delete a light source.
    ///
    /// # Arguments
    /// * `id` - Database ID of the light source
    ///
    /// # Returns
    /// * `Ok(())` - If deletion succeeds
    pub fn delete_light_source(&mut self, id: i32) -> Result<()> {
        diesel::delete(light_sources::table.find(id)).execute(self.conn)?;
        Ok(())
    }

    /// Delete all light sources attached to a token.
    /// (Called automatically when a token is deleted due to CASCADE, but useful for manual cleanup)
    ///
    /// # Arguments
    /// * `token_id` - Database ID of the token
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of light sources deleted
    pub fn delete_lights_for_token(&mut self, token_id: i32) -> Result<usize> {
        let count = diesel::delete(
            light_sources::table.filter(light_sources::token_id.eq(token_id)),
        )
        .execute(self.conn)?;
        Ok(count)
    }

    /// Delete all light sources on a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of light sources deleted
    pub fn delete_all_for_map(&mut self, map_id: i32) -> Result<usize> {
        let count =
            diesel::delete(light_sources::table.filter(light_sources::map_id.eq(map_id)))
                .execute(self.conn)?;
        Ok(count)
    }

    /// Count light sources on a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(i64)` - Number of light sources
    pub fn count_light_sources(&mut self, map_id: i32) -> Result<i64> {
        light_sources::table
            .filter(light_sources::map_id.eq(map_id))
            .count()
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Count active light sources on a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(i64)` - Number of active light sources
    pub fn count_active_light_sources(&mut self, map_id: i32) -> Result<i64> {
        light_sources::table
            .filter(light_sources::map_id.eq(map_id))
            .filter(light_sources::is_active.eq(true))
            .count()
            .get_result(self.conn)
            .map_err(Into::into)
    }
}
