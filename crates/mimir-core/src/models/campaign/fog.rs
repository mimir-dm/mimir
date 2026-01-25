//! Fog of War Models
//!
//! Models for managing fog of war revealed areas on maps.

use crate::schema::fog_revealed_areas;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A revealed area in the fog of war.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = fog_revealed_areas)]
pub struct FogRevealedArea {
    /// Unique ID (UUID)
    pub id: String,
    /// Map this revealed area belongs to
    pub map_id: String,
    /// X coordinate (grid units)
    pub x: f64,
    /// Y coordinate (grid units)
    pub y: f64,
    /// Width (grid units)
    pub width: f64,
    /// Height (grid units)
    pub height: f64,
    /// ISO8601 timestamp of creation
    pub created_at: String,
}

/// Data for inserting a new revealed area.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = fog_revealed_areas)]
pub struct NewFogRevealedArea<'a> {
    pub id: &'a str,
    pub map_id: &'a str,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl<'a> NewFogRevealedArea<'a> {
    /// Create a new revealed area.
    pub fn new(id: &'a str, map_id: &'a str, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            id,
            map_id,
            x,
            y,
            width,
            height,
        }
    }

    /// Create a revealed area from a rectangle.
    pub fn rect(id: &'a str, map_id: &'a str, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::new(id, map_id, x, y, width, height)
    }

    /// Create a revealed area from a circle (converted to bounding box).
    pub fn circle(id: &'a str, map_id: &'a str, center_x: f64, center_y: f64, radius: f64) -> Self {
        Self::new(
            id,
            map_id,
            center_x - radius,
            center_y - radius,
            radius * 2.0,
            radius * 2.0,
        )
    }
}

/// Fog state for a map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FogState {
    /// Map ID
    pub map_id: String,
    /// Whether fog is enabled
    pub fog_enabled: bool,
    /// Revealed areas
    pub revealed_areas: Vec<FogRevealedArea>,
}

impl FogState {
    /// Create a new fog state.
    pub fn new(map_id: String, fog_enabled: bool, revealed_areas: Vec<FogRevealedArea>) -> Self {
        Self {
            map_id,
            fog_enabled,
            revealed_areas,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_revealed_area() {
        let area = NewFogRevealedArea::new("area-1", "map-1", 10.0, 20.0, 5.0, 5.0);
        assert_eq!(area.x, 10.0);
        assert_eq!(area.y, 20.0);
        assert_eq!(area.width, 5.0);
        assert_eq!(area.height, 5.0);
    }

    #[test]
    fn test_circle_to_rect() {
        let area = NewFogRevealedArea::circle("area-1", "map-1", 10.0, 10.0, 5.0);
        // Circle at (10, 10) with radius 5 becomes rect at (5, 5) with size (10, 10)
        assert_eq!(area.x, 5.0);
        assert_eq!(area.y, 5.0);
        assert_eq!(area.width, 10.0);
        assert_eq!(area.height, 10.0);
    }

    #[test]
    fn test_fog_state() {
        let state = FogState::new("map-1".to_string(), true, vec![]);
        assert!(state.fog_enabled);
        assert!(state.revealed_areas.is_empty());
    }
}
