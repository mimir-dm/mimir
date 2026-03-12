//! Dungeondraft map header types.

use serde::{Deserialize, Serialize};

use super::godot_types::{NullableVector2, Vector2};

/// The top-level header of a `.dungeondraft_map` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub creation_build: String,
    pub creation_date: CreationDate,
    pub uses_default_assets: bool,
    #[serde(default)]
    pub asset_manifest: Vec<AssetPackRef>,
    pub editor_state: EditorState,
}

/// Date/time the map was created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationDate {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub weekday: u32,
    pub dst: bool,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

/// Reference to a third-party asset pack used by the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPackRef {
    pub name: String,
    pub id: String,
    pub version: String,
    pub author: String,
    pub keywords: Option<serde_json::Value>,
    #[serde(default)]
    pub allow_3rd_party_mapping_software_to_read: bool,
    #[serde(default)]
    pub custom_color_overrides: Option<CustomColorOverrides>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomColorOverrides {
    pub enabled: bool,
    pub min_redness: f64,
    pub min_saturation: f64,
    pub red_tolerance: f64,
}

/// Editor state preserved in the map file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorState {
    pub current_level: i32,
    pub camera_position: Vector2,
    pub camera_zoom: i32,
    pub guide_position: NullableVector2,
    pub trace_image: Option<serde_json::Value>,
    #[serde(default)]
    pub color_palettes: Option<ColorPalettes>,
    #[serde(default)]
    pub object_tags_memory: Option<serde_json::Value>,
    #[serde(default)]
    pub scatter_tags_memory: Option<serde_json::Value>,
    #[serde(default)]
    pub object_library_memory: Option<serde_json::Value>,
    #[serde(default)]
    pub scatter_library_memory: Option<serde_json::Value>,
    #[serde(default)]
    pub path_library_memory: Option<serde_json::Value>,
    #[serde(default)]
    pub sharpen_fonts: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColorPalettes {
    #[serde(default)]
    pub object_custom_colors: Vec<String>,
    #[serde(default)]
    pub scatter_custom_colors: Vec<String>,
    #[serde(default)]
    pub light_colors: Vec<String>,
    #[serde(default)]
    pub grid_colors: Vec<String>,
    #[serde(default)]
    pub deep_water_colors: Vec<String>,
    #[serde(default)]
    pub shallow_water_colors: Vec<String>,
    #[serde(default)]
    pub cave_ground_colors: Vec<String>,
    #[serde(default)]
    pub cave_wall_colors: Vec<String>,
}

impl Header {
    /// Create a minimal header for a new map.
    pub fn new(uses_default_assets: bool) -> Self {
        let now = chrono::Local::now();
        Self {
            creation_build: "mimir-mapgen 0.6.1".to_string(),
            creation_date: CreationDate {
                year: now.format("%Y").to_string().parse().unwrap_or(2026),
                month: now.format("%m").to_string().parse().unwrap_or(1),
                day: now.format("%d").to_string().parse().unwrap_or(1),
                weekday: now.format("%u").to_string().parse().unwrap_or(1),
                dst: false,
                hour: now.format("%H").to_string().parse().unwrap_or(0),
                minute: now.format("%M").to_string().parse().unwrap_or(0),
                second: now.format("%S").to_string().parse().unwrap_or(0),
            },
            uses_default_assets,
            asset_manifest: Vec::new(),
            editor_state: EditorState::default(),
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            current_level: 0,
            camera_position: Vector2::new(0.0, 0.0),
            camera_zoom: 8,
            guide_position: NullableVector2::Null,
            trace_image: None,
            color_palettes: Some(ColorPalettes::default()),
            object_tags_memory: Some(serde_json::json!({"set": 0, "tags": []})),
            scatter_tags_memory: Some(serde_json::json!({"set": 0, "tags": []})),
            object_library_memory: None,
            scatter_library_memory: None,
            path_library_memory: None,
            sharpen_fonts: Some(true),
        }
    }
}
