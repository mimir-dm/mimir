//! Light source database models for Vision and Lighting System
//!
//! Light sources represent illumination placed on maps - torches, lanterns, spells, etc.
//! They can be standalone or attached to tokens that move.

use crate::schema::light_sources;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Light source type - what kind of light this represents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LightType {
    Torch,
    Lantern,
    Candle,
    Spell,
    Custom,
}

impl LightType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LightType::Torch => "torch",
            LightType::Lantern => "lantern",
            LightType::Candle => "candle",
            LightType::Spell => "spell",
            LightType::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "torch" => LightType::Torch,
            "lantern" => LightType::Lantern,
            "candle" => LightType::Candle,
            "spell" => LightType::Spell,
            "custom" => LightType::Custom,
            _ => LightType::Torch,
        }
    }

    /// Get a human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            LightType::Torch => "Torch",
            LightType::Lantern => "Lantern",
            LightType::Candle => "Candle",
            LightType::Spell => "Spell",
            LightType::Custom => "Custom",
        }
    }

    /// Get default bright radius in feet for this light type
    pub fn default_bright_radius_ft(&self) -> f32 {
        match self {
            LightType::Torch => 20.0,
            LightType::Lantern => 30.0,  // Bullseye lantern
            LightType::Candle => 5.0,
            LightType::Spell => 20.0,    // Default for Light cantrip
            LightType::Custom => 20.0,
        }
    }

    /// Get default dim radius in feet for this light type
    pub fn default_dim_radius_ft(&self) -> f32 {
        match self {
            LightType::Torch => 40.0,
            LightType::Lantern => 60.0,
            LightType::Candle => 10.0,
            LightType::Spell => 40.0,
            LightType::Custom => 40.0,
        }
    }

    /// Get default color for this light type (hex format)
    pub fn default_color(&self) -> Option<&'static str> {
        match self {
            LightType::Torch => Some("#ff9933"),    // Warm orange
            LightType::Lantern => Some("#ffcc66"),  // Warmer yellow
            LightType::Candle => Some("#ffaa44"),   // Soft orange
            LightType::Spell => None,               // Pure white (no tint)
            LightType::Custom => None,
        }
    }
}

impl Default for LightType {
    fn default() -> Self {
        LightType::Torch
    }
}

/// Database model for light sources
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Identifiable)]
#[diesel(table_name = light_sources)]
pub struct LightSource {
    pub id: i32,
    pub map_id: i32,
    pub token_id: Option<i32>,
    pub name: String,
    pub light_type: String,
    pub x: f32,
    pub y: f32,
    pub bright_radius_ft: f32,
    pub dim_radius_ft: f32,
    pub color: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl LightSource {
    /// Get the light type enum
    pub fn light_type_enum(&self) -> LightType {
        LightType::from_str(&self.light_type)
    }

    /// Check if this light is attached to a token
    pub fn is_attached_to_token(&self) -> bool {
        self.token_id.is_some()
    }

    /// Get the effective position (for token-attached lights, use token position)
    /// Note: This returns the stored position; caller should use token position if attached
    pub fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}

/// New light source for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = light_sources)]
pub struct NewLightSource {
    pub map_id: i32,
    pub token_id: Option<i32>,
    pub name: String,
    pub light_type: String,
    pub x: f32,
    pub y: f32,
    pub bright_radius_ft: f32,
    pub dim_radius_ft: f32,
    pub color: Option<String>,
    pub is_active: bool,
}

impl NewLightSource {
    /// Create a new standalone light source at a position
    pub fn new(map_id: i32, name: String, light_type: LightType, x: f32, y: f32) -> Self {
        Self {
            map_id,
            token_id: None,
            name,
            light_type: light_type.as_str().to_string(),
            x,
            y,
            bright_radius_ft: light_type.default_bright_radius_ft(),
            dim_radius_ft: light_type.default_dim_radius_ft(),
            color: light_type.default_color().map(|s| s.to_string()),
            is_active: true,
        }
    }

    /// Create a light source attached to a token
    pub fn attached_to_token(map_id: i32, token_id: i32, name: String, light_type: LightType) -> Self {
        Self {
            map_id,
            token_id: Some(token_id),
            name,
            light_type: light_type.as_str().to_string(),
            x: 0.0,  // Position comes from token
            y: 0.0,
            bright_radius_ft: light_type.default_bright_radius_ft(),
            dim_radius_ft: light_type.default_dim_radius_ft(),
            color: light_type.default_color().map(|s| s.to_string()),
            is_active: true,
        }
    }

    /// Create a torch at a position
    pub fn torch(map_id: i32, x: f32, y: f32) -> Self {
        Self::new(map_id, "Torch".to_string(), LightType::Torch, x, y)
    }

    /// Create a lantern at a position
    pub fn lantern(map_id: i32, x: f32, y: f32) -> Self {
        Self::new(map_id, "Lantern".to_string(), LightType::Lantern, x, y)
    }

    /// Create a candle at a position
    pub fn candle(map_id: i32, x: f32, y: f32) -> Self {
        Self::new(map_id, "Candle".to_string(), LightType::Candle, x, y)
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_radii(mut self, bright_ft: f32, dim_ft: f32) -> Self {
        self.bright_radius_ft = bright_ft;
        self.dim_radius_ft = dim_ft;
        self
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }

    pub fn inactive(mut self) -> Self {
        self.is_active = false;
        self
    }
}

/// Light source update structure
#[derive(Debug, Clone, Default, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = light_sources)]
pub struct UpdateLightSource {
    pub name: Option<String>,
    pub light_type: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub bright_radius_ft: Option<f32>,
    pub dim_radius_ft: Option<f32>,
    pub color: Option<Option<String>>,
    pub is_active: Option<bool>,
    pub updated_at: Option<String>,
}

impl UpdateLightSource {
    /// Create an update for just the position
    pub fn position(x: f32, y: f32) -> Self {
        Self {
            x: Some(x),
            y: Some(y),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            ..Default::default()
        }
    }

    /// Create an update to toggle active state
    pub fn toggle_active(active: bool) -> Self {
        Self {
            is_active: Some(active),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            ..Default::default()
        }
    }

    /// Create an update for radii
    pub fn radii(bright_ft: f32, dim_ft: f32) -> Self {
        Self {
            bright_radius_ft: Some(bright_ft),
            dim_radius_ft: Some(dim_ft),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            ..Default::default()
        }
    }
}

/// Summary for listing light sources (includes token info if attached)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightSourceSummary {
    pub id: i32,
    pub map_id: i32,
    pub token_id: Option<i32>,
    pub token_name: Option<String>,
    pub name: String,
    pub light_type: String,
    pub x: f32,
    pub y: f32,
    pub bright_radius_ft: f32,
    pub dim_radius_ft: f32,
    pub color: Option<String>,
    pub is_active: bool,
}
