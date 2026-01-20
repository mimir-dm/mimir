//! Fog of War database models for Visual Display System
//!
//! Tracks revealed areas on maps for the fog of war feature.

use crate::schema::fog_revealed_areas;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Database model for fog revealed areas
/// Each record represents a rectangular region that has been revealed on a map
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Identifiable)]
#[diesel(table_name = fog_revealed_areas)]
pub struct FogRevealedArea {
    pub id: i32,
    pub map_id: i32,
    /// X coordinate (pixel position)
    pub x: f32,
    /// Y coordinate (pixel position)
    pub y: f32,
    /// Width of revealed area (pixels)
    pub width: f32,
    /// Height of revealed area (pixels)
    pub height: f32,
    pub created_at: String,
}

impl FogRevealedArea {
    /// Check if a point is within this revealed area
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }

    /// Check if this area overlaps with another rectangle
    pub fn overlaps(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        self.x < x + width && self.x + self.width > x && self.y < y + height && self.y + self.height > y
    }
}

/// New fog revealed area for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = fog_revealed_areas)]
pub struct NewFogRevealedArea {
    pub map_id: i32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl NewFogRevealedArea {
    /// Create a new revealed area
    pub fn new(map_id: i32, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            map_id,
            x,
            y,
            width,
            height,
        }
    }

    /// Create a revealed area from a center point and radius (circular reveal stored as bounding box)
    pub fn from_circle(map_id: i32, center_x: f32, center_y: f32, radius: f32) -> Self {
        Self {
            map_id,
            x: center_x - radius,
            y: center_y - radius,
            width: radius * 2.0,
            height: radius * 2.0,
        }
    }
}

/// Summary for listing fog areas (same as model for now, but allows future extension)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FogRevealedAreaSummary {
    pub id: i32,
    pub map_id: i32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl From<FogRevealedArea> for FogRevealedAreaSummary {
    fn from(area: FogRevealedArea) -> Self {
        Self {
            id: area.id,
            map_id: area.map_id,
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
        }
    }
}
