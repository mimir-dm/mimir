//! Fog of War service for Visual Display System.
//!
//! Manages fog of war state on maps - tracking revealed areas
//! and toggling fog visibility.

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::campaign::{FogRevealedArea, FogRevealedAreaSummary, NewFogRevealedArea, UpdateMap};
use crate::schema::{fog_revealed_areas, maps};
use diesel::prelude::*;

/// Service for managing fog of war
pub struct FogOfWarService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> FogOfWarService<'a> {
    /// Create a new fog of war service.
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Enable fog of war on a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(())` - If successful
    pub fn enable_fog(&mut self, map_id: i32) -> Result<()> {
        let update = UpdateMap {
            fog_enabled: Some(true),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            ..Default::default()
        };

        diesel::update(maps::table.find(map_id))
            .set(&update)
            .execute(self.conn)?;

        Ok(())
    }

    /// Disable fog of war on a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(())` - If successful
    pub fn disable_fog(&mut self, map_id: i32) -> Result<()> {
        let update = UpdateMap {
            fog_enabled: Some(false),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            ..Default::default()
        };

        diesel::update(maps::table.find(map_id))
            .set(&update)
            .execute(self.conn)?;

        Ok(())
    }

    /// Toggle fog of war on a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(bool)` - The new fog_enabled state
    pub fn toggle_fog(&mut self, map_id: i32) -> Result<bool> {
        // Get current state
        let current: bool = maps::table
            .find(map_id)
            .select(maps::fog_enabled)
            .first(self.conn)?;

        let new_state = !current;
        let update = UpdateMap {
            fog_enabled: Some(new_state),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            ..Default::default()
        };

        diesel::update(maps::table.find(map_id))
            .set(&update)
            .execute(self.conn)?;

        Ok(new_state)
    }

    /// Check if fog is enabled on a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(bool)` - Whether fog is enabled
    pub fn is_fog_enabled(&mut self, map_id: i32) -> Result<bool> {
        maps::table
            .find(map_id)
            .select(maps::fog_enabled)
            .first(self.conn)
            .map_err(Into::into)
    }

    /// Reveal an area on the map.
    ///
    /// # Arguments
    /// * `new_area` - The area to reveal
    ///
    /// # Returns
    /// * `Ok(FogRevealedArea)` - The created revealed area record
    pub fn reveal_area(&mut self, new_area: NewFogRevealedArea) -> Result<FogRevealedArea> {
        diesel::insert_into(fog_revealed_areas::table)
            .values(&new_area)
            .returning(FogRevealedArea::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Reveal a rectangular area by coordinates.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    /// * `x` - X coordinate (pixels)
    /// * `y` - Y coordinate (pixels)
    /// * `width` - Width of area (pixels)
    /// * `height` - Height of area (pixels)
    ///
    /// # Returns
    /// * `Ok(FogRevealedArea)` - The created revealed area
    pub fn reveal_rect(
        &mut self,
        map_id: i32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<FogRevealedArea> {
        let new_area = NewFogRevealedArea::new(map_id, x, y, width, height);
        self.reveal_area(new_area)
    }

    /// Reveal a circular area (stored as bounding box).
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    /// * `center_x` - Center X coordinate (pixels)
    /// * `center_y` - Center Y coordinate (pixels)
    /// * `radius` - Radius of the circle (pixels)
    ///
    /// # Returns
    /// * `Ok(FogRevealedArea)` - The created revealed area
    pub fn reveal_circle(
        &mut self,
        map_id: i32,
        center_x: f32,
        center_y: f32,
        radius: f32,
    ) -> Result<FogRevealedArea> {
        let new_area = NewFogRevealedArea::from_circle(map_id, center_x, center_y, radius);
        self.reveal_area(new_area)
    }

    /// Get all revealed areas for a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(Vec<FogRevealedArea>)` - All revealed areas on the map
    pub fn get_revealed_areas(&mut self, map_id: i32) -> Result<Vec<FogRevealedArea>> {
        fog_revealed_areas::table
            .filter(fog_revealed_areas::map_id.eq(map_id))
            .order(fog_revealed_areas::id.asc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Get revealed area summaries for a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(Vec<FogRevealedAreaSummary>)` - Summaries of all revealed areas
    pub fn get_revealed_area_summaries(&mut self, map_id: i32) -> Result<Vec<FogRevealedAreaSummary>> {
        let areas = self.get_revealed_areas(map_id)?;
        Ok(areas.into_iter().map(FogRevealedAreaSummary::from).collect())
    }

    /// Delete a specific revealed area (re-fog that area).
    ///
    /// # Arguments
    /// * `id` - Database ID of the revealed area
    ///
    /// # Returns
    /// * `Ok(())` - If deletion succeeds
    pub fn delete_revealed_area(&mut self, id: i32) -> Result<()> {
        diesel::delete(fog_revealed_areas::table.find(id))
            .execute(self.conn)?;
        Ok(())
    }

    /// Clear all revealed areas for a map (reset fog to full coverage).
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of revealed areas deleted
    pub fn reset_fog(&mut self, map_id: i32) -> Result<usize> {
        let count = diesel::delete(
            fog_revealed_areas::table.filter(fog_revealed_areas::map_id.eq(map_id)),
        )
        .execute(self.conn)?;
        Ok(count)
    }

    /// Reveal all (remove all fog from a map).
    /// This is useful for "reveal all" button functionality.
    ///
    /// Note: This doesn't create revealed areas - instead it should be
    /// handled on the frontend by disabling fog or creating a full-map
    /// revealed area.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    /// * `map_width` - Width of the map in pixels
    /// * `map_height` - Height of the map in pixels
    ///
    /// # Returns
    /// * `Ok(FogRevealedArea)` - A revealed area covering the entire map
    pub fn reveal_all(
        &mut self,
        map_id: i32,
        map_width: f32,
        map_height: f32,
    ) -> Result<FogRevealedArea> {
        self.reveal_rect(map_id, 0.0, 0.0, map_width, map_height)
    }

    /// Count revealed areas on a map.
    ///
    /// # Arguments
    /// * `map_id` - Database ID of the map
    ///
    /// # Returns
    /// * `Ok(i64)` - Number of revealed areas
    pub fn count_revealed_areas(&mut self, map_id: i32) -> Result<i64> {
        fog_revealed_areas::table
            .filter(fog_revealed_areas::map_id.eq(map_id))
            .count()
            .get_result(self.conn)
            .map_err(Into::into)
    }
}
