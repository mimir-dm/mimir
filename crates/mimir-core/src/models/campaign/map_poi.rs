//! MapPoi Model
//!
//! Points of Interest (POIs) placed on maps - markers for locations, secrets, notes, etc.

use crate::schema::map_pois;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A point of interest placed on a map.
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = map_pois)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MapPoi {
    pub id: String,
    pub map_id: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub color: Option<String>,
    pub visible: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl MapPoi {
    /// Check if POI is visible to players.
    pub fn is_visible(&self) -> bool {
        self.visible == 1
    }
}

/// Data for creating a new POI.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = map_pois)]
pub struct NewMapPoi<'a> {
    pub id: &'a str,
    pub map_id: &'a str,
    pub grid_x: i32,
    pub grid_y: i32,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub icon: &'a str,
    pub color: Option<&'a str>,
    pub visible: i32,
}

impl<'a> NewMapPoi<'a> {
    /// Create a new POI with required fields.
    pub fn new(id: &'a str, map_id: &'a str, name: &'a str, grid_x: i32, grid_y: i32) -> Self {
        Self {
            id,
            map_id,
            grid_x,
            grid_y,
            name,
            description: None,
            icon: "pin",
            color: None,
            visible: 0,
        }
    }

    /// Set description.
    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Set icon type.
    pub fn with_icon(mut self, icon: &'a str) -> Self {
        self.icon = icon;
        self
    }

    /// Set color.
    pub fn with_color(mut self, color: &'a str) -> Self {
        self.color = Some(color);
        self
    }

    /// Set as visible to players.
    pub fn visible(mut self) -> Self {
        self.visible = 1;
        self
    }
}

/// Data for updating an existing POI.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = map_pois)]
pub struct UpdateMapPoi<'a> {
    pub grid_x: Option<i32>,
    pub grid_y: Option<i32>,
    pub name: Option<&'a str>,
    pub description: Option<Option<&'a str>>,
    pub icon: Option<&'a str>,
    pub color: Option<Option<&'a str>>,
    pub visible: Option<i32>,
    pub updated_at: Option<&'a str>,
}

impl<'a> UpdateMapPoi<'a> {
    /// Update position.
    pub fn set_position(grid_x: i32, grid_y: i32, updated_at: &'a str) -> Self {
        Self {
            grid_x: Some(grid_x),
            grid_y: Some(grid_y),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Set visibility.
    pub fn set_visible(visible: bool, updated_at: &'a str) -> Self {
        Self {
            visible: Some(if visible { 1 } else { 0 }),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_poi() {
        let poi = NewMapPoi::new("poi-1", "map-1", "Secret Door", 5, 10);
        assert_eq!(poi.id, "poi-1");
        assert_eq!(poi.map_id, "map-1");
        assert_eq!(poi.name, "Secret Door");
        assert_eq!(poi.grid_x, 5);
        assert_eq!(poi.grid_y, 10);
        assert_eq!(poi.icon, "pin");
        assert_eq!(poi.visible, 0);
    }

    #[test]
    fn test_new_poi_with_options() {
        let poi = NewMapPoi::new("poi-1", "map-1", "Treasure", 0, 0)
            .with_description("Hidden chest")
            .with_icon("chest")
            .with_color("#ffcc00")
            .visible();

        assert_eq!(poi.description, Some("Hidden chest"));
        assert_eq!(poi.icon, "chest");
        assert_eq!(poi.color, Some("#ffcc00"));
        assert_eq!(poi.visible, 1);
    }

    #[test]
    fn test_update_position() {
        let update = UpdateMapPoi::set_position(10, 20, "2024-01-20T12:00:00Z");
        assert_eq!(update.grid_x, Some(10));
        assert_eq!(update.grid_y, Some(20));
    }

    #[test]
    fn test_update_visible() {
        let update = UpdateMapPoi::set_visible(true, "2024-01-20T12:00:00Z");
        assert_eq!(update.visible, Some(1));

        let update = UpdateMapPoi::set_visible(false, "2024-01-20T12:00:00Z");
        assert_eq!(update.visible, Some(0));
    }
}
