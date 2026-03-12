//! Dungeondraft world and level types.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::entities::*;
use super::godot_types::*;

/// The world section of a `.dungeondraft_map` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub format: u32,
    pub width: u32,
    pub height: u32,
    pub next_node_id: String,
    #[serde(default)]
    pub next_prefab_id: String,
    #[serde(default)]
    pub msi: Option<MapSpaceInfo>,
    pub grid: Grid,
    #[serde(default)]
    pub building_wear: Option<serde_json::Value>,
    #[serde(default)]
    pub wall_shadow: bool,
    #[serde(default)]
    pub object_shadow: bool,
    #[serde(default)]
    pub trace_image_visible: bool,
    #[serde(default)]
    pub embedded: BTreeMap<String, serde_json::Value>,
    pub levels: BTreeMap<String, Level>,
}

/// Map space info for material scatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapSpaceInfo {
    pub offset_map_size: u32,
    pub max_offset_distance: f64,
    pub cell_size: u32,
    pub seed: String,
}

/// Grid configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub color: String,
    pub texture: String,
}

/// A single level (floor) in the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub label: String,
    #[serde(default)]
    pub environment: Environment,
    #[serde(default)]
    pub layers: BTreeMap<String, String>,
    #[serde(default)]
    pub shapes: Shapes,
    #[serde(default)]
    pub tiles: Option<Tiles>,
    #[serde(default)]
    pub patterns: Vec<MapPattern>,
    #[serde(default)]
    pub walls: Vec<MapWall>,
    #[serde(default)]
    pub portals: Vec<MapPortal>,
    #[serde(default)]
    pub cave: Option<Cave>,
    #[serde(default)]
    pub terrain: Option<Terrain>,
    #[serde(default)]
    pub water: Option<Water>,
    #[serde(default)]
    pub materials: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub paths: Vec<MapPath>,
    #[serde(default)]
    pub objects: Vec<MapObject>,
    #[serde(default)]
    pub lights: Vec<MapLight>,
    #[serde(default)]
    pub roofs: Option<Roofs>,
    #[serde(default)]
    pub texts: Vec<MapText>,
    #[serde(default = "default_true")]
    pub texts_vis: bool,
}

/// Lighting environment for a level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    #[serde(default)]
    pub baked_lighting: bool,
    #[serde(default = "default_ambient_light")]
    pub ambient_light: String,
    #[serde(default)]
    pub ambient_energy: Option<f64>,
    #[serde(default)]
    pub shadow_color: Option<String>,
}

fn default_ambient_light() -> String {
    "ffffffff".to_string()
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            baked_lighting: true,
            ambient_light: default_ambient_light(),
            ambient_energy: None,
            shadow_color: None,
        }
    }
}

/// Wall and polygon shapes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Shapes {
    #[serde(default)]
    pub polygons: Vec<serde_json::Value>,
    #[serde(default)]
    pub walls: Vec<serde_json::Value>,
}

/// Tile grid data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tiles {
    pub cells: PoolIntArray,
    #[serde(default)]
    pub colors: Vec<String>,
    #[serde(default)]
    pub lookup: BTreeMap<String, String>,
}

/// Cave bitmap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cave {
    pub bitmap: PoolByteArray,
    #[serde(default = "default_cave_color")]
    pub ground_color: String,
    #[serde(default = "default_cave_color")]
    pub wall_color: String,
    #[serde(default)]
    pub entrance_bitmap: Option<PoolByteArray>,
    #[serde(default = "default_cave_texture")]
    pub texture: String,
}

fn default_cave_color() -> String {
    "ff7f7e71".to_string()
}

fn default_cave_texture() -> String {
    "res://textures/caves/colorable/floor.png".to_string()
}

/// Terrain configuration with 4-texture splat map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terrain {
    pub enabled: bool,
    #[serde(default)]
    pub expand_slots: bool,
    #[serde(default)]
    pub smooth_blending: bool,
    pub texture_1: String,
    pub texture_2: String,
    pub texture_3: String,
    pub texture_4: String,
    pub splat: PoolByteArray,
}

impl Terrain {
    /// Create a new terrain with uniform texture_1 coverage.
    ///
    /// The splat map has 4 bytes per cell at 4x4 cells per grid square.
    /// `width` and `height` are in grid squares.
    pub fn new_uniform(width: u32, height: u32, textures: [String; 4]) -> Self {
        let cell_count = (width * 4 * height * 4) as usize;
        // Uniform texture_1: (255, 0, 0, 0) per cell
        let mut splat_data = Vec::with_capacity(cell_count * 4);
        for _ in 0..cell_count {
            splat_data.extend_from_slice(&[255, 0, 0, 0]);
        }
        Self {
            enabled: true,
            expand_slots: false,
            smooth_blending: false,
            texture_1: textures[0].clone(),
            texture_2: textures[1].clone(),
            texture_3: textures[2].clone(),
            texture_4: textures[3].clone(),
            splat: PoolByteArray(splat_data),
        }
    }
}

/// Water configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Water {
    #[serde(default)]
    pub disable_border: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree: Option<WaterTree>,
}

/// Recursive water polygon tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterTree {
    #[serde(rename = "ref")]
    pub node_ref: i64,
    pub polygon: PoolVector2Array,
    #[serde(default)]
    pub join: i32,
    #[serde(default)]
    pub end: i32,
    #[serde(default)]
    pub is_open: bool,
    pub deep_color: String,
    pub shallow_color: String,
    #[serde(default)]
    pub blend_distance: f64,
    #[serde(default)]
    pub children: Vec<WaterTree>,
}

/// Roof configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roofs {
    #[serde(default = "default_true")]
    pub shade: bool,
    #[serde(default = "default_shade_contrast")]
    pub shade_contrast: f64,
    #[serde(default = "default_sun_direction")]
    pub sun_direction: f64,
    #[serde(default)]
    pub roofs: Vec<serde_json::Value>,
}

fn default_true() -> bool {
    true
}
fn default_shade_contrast() -> f64 {
    0.5
}
fn default_sun_direction() -> f64 {
    45.0
}

impl Default for Roofs {
    fn default() -> Self {
        Self {
            shade: true,
            shade_contrast: 0.5,
            sun_direction: 45.0,
            roofs: Vec::new(),
        }
    }
}

impl Level {
    /// Create a default ground-level level.
    pub fn new_ground(width: u32, height: u32) -> Self {
        let mut layers = BTreeMap::new();
        layers.insert("-400".to_string(), "Below Ground".to_string());
        layers.insert("-100".to_string(), "Below Water".to_string());
        layers.insert("100".to_string(), "User Layer 1".to_string());
        layers.insert("200".to_string(), "User Layer 2".to_string());
        layers.insert("300".to_string(), "User Layer 3".to_string());
        layers.insert("400".to_string(), "User Layer 4".to_string());
        layers.insert("700".to_string(), "Above Walls".to_string());
        layers.insert("900".to_string(), "Above Roofs".to_string());

        let cell_count = (width * height) as usize;
        // Cave bitmap size: w*h*2 + floor(1.5*(w+h)) + 2
        // (empirically determined from Dungeondraft's own format)
        let cave_bitmap_size =
            (width * height * 2 + (3 * (width + height)) / 2 + 2) as usize;

        Self {
            label: "Ground".to_string(),
            environment: Environment::default(),
            layers,
            shapes: Shapes::default(),
            tiles: Some(Tiles {
                cells: PoolIntArray::filled(-1, cell_count),
                colors: vec!["ffffffff".to_string(); cell_count],
                lookup: BTreeMap::new(),
            }),
            patterns: Vec::new(),
            walls: Vec::new(),
            portals: Vec::new(),
            cave: Some(Cave {
                bitmap: PoolByteArray::from_vec(vec![0; cave_bitmap_size]),
                ground_color: default_cave_color(),
                wall_color: default_cave_color(),
                entrance_bitmap: Some(PoolByteArray::from_vec(vec![0; cave_bitmap_size])),
                texture: default_cave_texture(),
            }),
            terrain: None,
            water: Some(Water {
                disable_border: false,
                tree: None,
            }),
            materials: BTreeMap::new(),
            paths: Vec::new(),
            objects: Vec::new(),
            lights: Vec::new(),
            roofs: Some(Roofs::default()),
            texts: Vec::new(),
            texts_vis: true,
        }
    }
}

impl World {
    /// Create a new empty world with the given dimensions (in grid squares).
    pub fn new(width: u32, height: u32) -> Self {
        let mut levels = BTreeMap::new();
        levels.insert("0".to_string(), Level::new_ground(width, height));

        Self {
            format: 3,
            width,
            height,
            next_node_id: "1".to_string(),
            next_prefab_id: "0".to_string(),
            msi: Some(MapSpaceInfo {
                offset_map_size: 512,
                max_offset_distance: 0.2,
                cell_size: 64,
                seed: "00000000".to_string(),
            }),
            grid: Grid {
                color: "7f000000".to_string(),
                texture: "res://textures/grid/dotted_line.png".to_string(),
            },
            building_wear: None,
            wall_shadow: true,
            object_shadow: false,
            trace_image_visible: false,
            embedded: BTreeMap::new(),
            levels,
        }
    }
}
