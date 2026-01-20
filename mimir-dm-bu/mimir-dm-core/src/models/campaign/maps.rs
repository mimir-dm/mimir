//! Map database models for Visual Display System

use crate::schema::maps;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Grid type for map overlay
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GridType {
    Square,
    Hex,
    None,
}

/// Ambient light level for a map
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AmbientLight {
    Bright,
    Dim,
    Darkness,
}

impl AmbientLight {
    pub fn as_str(&self) -> &'static str {
        match self {
            AmbientLight::Bright => "bright",
            AmbientLight::Dim => "dim",
            AmbientLight::Darkness => "darkness",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dim" => AmbientLight::Dim,
            "darkness" | "dark" => AmbientLight::Darkness,
            _ => AmbientLight::Bright,
        }
    }
}

impl Default for AmbientLight {
    fn default() -> Self {
        AmbientLight::Bright
    }
}

impl GridType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GridType::Square => "square",
            GridType::Hex => "hex",
            GridType::None => "none",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "square" => GridType::Square,
            "hex" => GridType::Hex,
            _ => GridType::None,
        }
    }
}

impl Default for GridType {
    fn default() -> Self {
        GridType::None
    }
}

/// Database model for maps
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Identifiable)]
#[diesel(table_name = maps)]
pub struct Map {
    pub id: i32,
    pub campaign_id: i32,
    pub module_id: Option<i32>,
    pub name: String,
    pub image_path: String,
    pub width_px: i32,
    pub height_px: i32,
    pub original_width_px: Option<i32>,
    pub original_height_px: Option<i32>,
    pub grid_type: String,
    pub grid_size_px: Option<i32>,
    pub grid_offset_x: i32,
    pub grid_offset_y: i32,
    pub created_at: String,
    pub updated_at: String,
    pub preview_path: Option<String>,
    pub fog_enabled: bool,
    pub ambient_light: String,
}

impl Map {
    /// Get the grid type enum
    pub fn grid_type_enum(&self) -> GridType {
        GridType::from_str(&self.grid_type)
    }

    /// Get the ambient light level enum
    pub fn ambient_light_enum(&self) -> AmbientLight {
        AmbientLight::from_str(&self.ambient_light)
    }

    /// Check if this is a campaign-level map (not tied to a module)
    pub fn is_campaign_level(&self) -> bool {
        self.module_id.is_none()
    }
}

/// New map for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = maps)]
pub struct NewMap {
    pub campaign_id: i32,
    pub module_id: Option<i32>,
    pub name: String,
    pub image_path: String,
    pub width_px: i32,
    pub height_px: i32,
    pub original_width_px: Option<i32>,
    pub original_height_px: Option<i32>,
    pub grid_type: String,
    pub grid_size_px: Option<i32>,
    pub grid_offset_x: i32,
    pub grid_offset_y: i32,
    pub ambient_light: String,
    pub fog_enabled: bool,
}

impl NewMap {
    pub fn new(
        campaign_id: i32,
        name: String,
        image_path: String,
        width_px: i32,
        height_px: i32,
        original_width_px: i32,
        original_height_px: i32,
    ) -> Self {
        Self {
            campaign_id,
            module_id: None,
            name,
            image_path,
            width_px,
            height_px,
            original_width_px: Some(original_width_px),
            original_height_px: Some(original_height_px),
            grid_type: GridType::None.as_str().to_string(),
            grid_size_px: None,
            grid_offset_x: 0,
            grid_offset_y: 0,
            ambient_light: AmbientLight::default().as_str().to_string(),
            fog_enabled: true,
        }
    }

    pub fn with_ambient_light(mut self, ambient_light: AmbientLight) -> Self {
        self.ambient_light = ambient_light.as_str().to_string();
        self
    }

    pub fn with_module(mut self, module_id: i32) -> Self {
        self.module_id = Some(module_id);
        self
    }

    pub fn with_grid(mut self, grid_type: GridType, size_px: i32, offset_x: i32, offset_y: i32) -> Self {
        self.grid_type = grid_type.as_str().to_string();
        self.grid_size_px = Some(size_px);
        self.grid_offset_x = offset_x;
        self.grid_offset_y = offset_y;
        self
    }

    pub fn with_fog_enabled(mut self, enabled: bool) -> Self {
        self.fog_enabled = enabled;
        self
    }
}

/// Map update structure
#[derive(Debug, Clone, Default, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = maps)]
pub struct UpdateMap {
    pub name: Option<String>,
    pub grid_type: Option<String>,
    pub grid_size_px: Option<Option<i32>>,
    pub grid_offset_x: Option<i32>,
    pub grid_offset_y: Option<i32>,
    pub updated_at: Option<String>,
    pub fog_enabled: Option<bool>,
    pub ambient_light: Option<String>,
}

/// Summary for listing maps (lighter weight than full Map)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapSummary {
    pub id: i32,
    pub name: String,
    pub module_id: Option<i32>,
    pub module_name: Option<String>,
    pub grid_type: String,
    pub grid_size_px: Option<i32>,
    pub grid_offset_x: i32,
    pub grid_offset_y: i32,
    pub width_px: i32,
    pub height_px: i32,
    pub original_width_px: Option<i32>,
    pub original_height_px: Option<i32>,
    pub fog_enabled: bool,
    pub ambient_light: String,
    /// Path to the map file (UVTT or image)
    pub image_path: String,
}
