//! LightSource Model
//!
//! Dynamic light sources on maps (beyond UVTT static lights).

use crate::schema::light_sources;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A dynamic light source on a map.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = light_sources)]
pub struct LightSource {
    /// Unique ID (UUID)
    pub id: String,
    /// Map this light is placed on
    pub map_id: String,
    /// Grid X coordinate
    pub grid_x: i32,
    /// Grid Y coordinate
    pub grid_y: i32,
    /// Light name (e.g., "Torch", "Lantern")
    pub name: Option<String>,
    /// Bright light radius in feet
    pub bright_radius: i32,
    /// Dim light radius in feet
    pub dim_radius: i32,
    /// Color (hex, e.g., "#FFAA00" for warm torch light)
    pub color: Option<String>,
    /// Whether the light is active (0=off, 1=on)
    pub active: i32,
    /// ISO8601 timestamp of creation
    pub created_at: String,
    /// ISO8601 timestamp of last update
    pub updated_at: String,
}

impl LightSource {
    /// Check if this light source is active.
    pub fn is_active(&self) -> bool {
        self.active != 0
    }

    /// Get the total light radius (dim extends beyond bright).
    pub fn total_radius(&self) -> i32 {
        self.dim_radius
    }
}

/// Common light source presets
pub mod presets {
    /// Torch: 20 ft bright, 40 ft dim, warm orange
    pub const TORCH_BRIGHT: i32 = 20;
    pub const TORCH_DIM: i32 = 40;
    pub const TORCH_COLOR: &str = "#FFAA00";

    /// Lantern: 30 ft bright, 60 ft dim, warm yellow
    pub const LANTERN_BRIGHT: i32 = 30;
    pub const LANTERN_DIM: i32 = 60;
    pub const LANTERN_COLOR: &str = "#FFD700";

    /// Candle: 5 ft bright, 10 ft dim, warm orange
    pub const CANDLE_BRIGHT: i32 = 5;
    pub const CANDLE_DIM: i32 = 10;
    pub const CANDLE_COLOR: &str = "#FFCC66";

    /// Light spell: 20 ft bright, 40 ft dim, white
    pub const LIGHT_BRIGHT: i32 = 20;
    pub const LIGHT_DIM: i32 = 40;
    pub const LIGHT_COLOR: &str = "#FFFFFF";

    /// Daylight spell: 60 ft bright, 120 ft dim, bright white
    pub const DAYLIGHT_BRIGHT: i32 = 60;
    pub const DAYLIGHT_DIM: i32 = 120;
    pub const DAYLIGHT_COLOR: &str = "#FFFFEE";
}

/// Data for inserting a new light source.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = light_sources)]
pub struct NewLightSource<'a> {
    pub id: &'a str,
    pub map_id: &'a str,
    pub grid_x: i32,
    pub grid_y: i32,
    pub name: Option<&'a str>,
    pub bright_radius: i32,
    pub dim_radius: i32,
    pub color: Option<&'a str>,
    pub active: i32,
}

impl<'a> NewLightSource<'a> {
    /// Create a new light source.
    pub fn new(
        id: &'a str,
        map_id: &'a str,
        grid_x: i32,
        grid_y: i32,
        bright_radius: i32,
        dim_radius: i32,
    ) -> Self {
        Self {
            id,
            map_id,
            grid_x,
            grid_y,
            name: None,
            bright_radius,
            dim_radius,
            color: None,
            active: 1,
        }
    }

    /// Create a torch light source.
    pub fn torch(id: &'a str, map_id: &'a str, grid_x: i32, grid_y: i32) -> Self {
        Self::new(
            id,
            map_id,
            grid_x,
            grid_y,
            presets::TORCH_BRIGHT,
            presets::TORCH_DIM,
        )
        .with_name("Torch")
        .with_color(presets::TORCH_COLOR)
    }

    /// Create a lantern light source.
    pub fn lantern(id: &'a str, map_id: &'a str, grid_x: i32, grid_y: i32) -> Self {
        Self::new(
            id,
            map_id,
            grid_x,
            grid_y,
            presets::LANTERN_BRIGHT,
            presets::LANTERN_DIM,
        )
        .with_name("Lantern")
        .with_color(presets::LANTERN_COLOR)
    }

    /// Create a candle light source.
    pub fn candle(id: &'a str, map_id: &'a str, grid_x: i32, grid_y: i32) -> Self {
        Self::new(
            id,
            map_id,
            grid_x,
            grid_y,
            presets::CANDLE_BRIGHT,
            presets::CANDLE_DIM,
        )
        .with_name("Candle")
        .with_color(presets::CANDLE_COLOR)
    }

    /// Set the name.
    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the color.
    pub fn with_color(mut self, color: &'a str) -> Self {
        self.color = Some(color);
        self
    }

    /// Create in inactive state.
    pub fn inactive(mut self) -> Self {
        self.active = 0;
        self
    }
}

/// Data for updating a light source.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = light_sources)]
pub struct UpdateLightSource<'a> {
    pub grid_x: Option<i32>,
    pub grid_y: Option<i32>,
    pub name: Option<Option<&'a str>>,
    pub bright_radius: Option<i32>,
    pub dim_radius: Option<i32>,
    pub color: Option<Option<&'a str>>,
    pub active: Option<i32>,
    pub updated_at: Option<&'a str>,
}

impl<'a> UpdateLightSource<'a> {
    /// Update position.
    pub fn set_position(grid_x: i32, grid_y: i32, updated_at: &'a str) -> Self {
        Self {
            grid_x: Some(grid_x),
            grid_y: Some(grid_y),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update radii.
    pub fn set_radii(bright_radius: i32, dim_radius: i32, updated_at: &'a str) -> Self {
        Self {
            bright_radius: Some(bright_radius),
            dim_radius: Some(dim_radius),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Update color.
    pub fn set_color(color: Option<&'a str>, updated_at: &'a str) -> Self {
        Self {
            color: Some(color),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Turn light on.
    pub fn turn_on(updated_at: &'a str) -> Self {
        Self {
            active: Some(1),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Turn light off.
    pub fn turn_off(updated_at: &'a str) -> Self {
        Self {
            active: Some(0),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_light_source() {
        let light = NewLightSource::new("ls-1", "map-1", 5, 10, 20, 40);
        assert_eq!(light.bright_radius, 20);
        assert_eq!(light.dim_radius, 40);
        assert_eq!(light.active, 1);
    }

    #[test]
    fn test_torch_preset() {
        let torch = NewLightSource::torch("ls-1", "map-1", 5, 10);
        assert_eq!(torch.bright_radius, presets::TORCH_BRIGHT);
        assert_eq!(torch.dim_radius, presets::TORCH_DIM);
        assert_eq!(torch.name, Some("Torch"));
        assert_eq!(torch.color, Some(presets::TORCH_COLOR));
    }

    #[test]
    fn test_lantern_preset() {
        let lantern = NewLightSource::lantern("ls-1", "map-1", 0, 0);
        assert_eq!(lantern.bright_radius, presets::LANTERN_BRIGHT);
        assert_eq!(lantern.dim_radius, presets::LANTERN_DIM);
        assert_eq!(lantern.name, Some("Lantern"));
    }

    #[test]
    fn test_candle_preset() {
        let candle = NewLightSource::candle("ls-1", "map-1", 0, 0);
        assert_eq!(candle.bright_radius, presets::CANDLE_BRIGHT);
        assert_eq!(candle.dim_radius, presets::CANDLE_DIM);
    }

    #[test]
    fn test_inactive() {
        let light = NewLightSource::new("ls-1", "map-1", 0, 0, 20, 40).inactive();
        assert_eq!(light.active, 0);
    }

    #[test]
    fn test_with_name_and_color() {
        let light = NewLightSource::new("ls-1", "map-1", 0, 0, 60, 120)
            .with_name("Magical Beacon")
            .with_color("#00FF00");
        assert_eq!(light.name, Some("Magical Beacon"));
        assert_eq!(light.color, Some("#00FF00"));
    }

    #[test]
    fn test_update_turn_on_off() {
        let update = UpdateLightSource::turn_on("2024-01-20T12:00:00Z");
        assert_eq!(update.active, Some(1));

        let update = UpdateLightSource::turn_off("2024-01-20T12:00:00Z");
        assert_eq!(update.active, Some(0));
    }

    #[test]
    fn test_update_position() {
        let update = UpdateLightSource::set_position(15, 20, "2024-01-20T12:00:00Z");
        assert_eq!(update.grid_x, Some(15));
        assert_eq!(update.grid_y, Some(20));
    }
}
