//! Map entity types: objects, paths, lights, walls, patterns, portals, texts.

use serde::{Deserialize, Serialize};

use super::godot_types::*;

/// A placed object on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapObject {
    pub texture: String,
    pub position: Vector2,
    pub scale: Vector2,
    pub rotation: f64,
    #[serde(default)]
    pub mirror: bool,
    pub layer: i32,
    #[serde(rename = "ref")]
    pub node_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow: Option<bool>,
}

impl MapObject {
    pub fn new(texture: &str, position: Vector2, node_ref: &str) -> Self {
        Self {
            texture: texture.to_string(),
            position,
            scale: Vector2::new(1.0, 1.0),
            rotation: 0.0,
            mirror: false,
            layer: 100,
            node_ref: node_ref.to_string(),
            custom_color: None,
            shadow: None,
        }
    }

    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = Vector2::new(scale, scale);
        self
    }

    pub fn with_rotation(mut self, radians: f64) -> Self {
        self.rotation = radians;
        self
    }

    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    pub fn with_mirror(mut self, mirror: bool) -> Self {
        self.mirror = mirror;
        self
    }

    pub fn with_custom_color(mut self, color: &str) -> Self {
        self.custom_color = Some(color.to_string());
        self
    }
}

/// A path on the map (roads, rivers, cliffs, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapPath {
    pub texture: String,
    pub color: String,
    pub points: PoolVector2Array,
    pub width: f64,
    pub layer: i32,
    #[serde(rename = "ref")]
    pub node_ref: String,
    #[serde(default)]
    pub loop_path: bool,
}

impl MapPath {
    pub fn new(texture: &str, points: Vec<Vector2>, width: f64, node_ref: &str) -> Self {
        Self {
            texture: texture.to_string(),
            color: "ffffffff".to_string(),
            points: PoolVector2Array::from_points(points),
            width,
            layer: 100,
            node_ref: node_ref.to_string(),
            loop_path: false,
        }
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.color = color.to_string();
        self
    }

    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    pub fn with_loop(mut self, loop_path: bool) -> Self {
        self.loop_path = loop_path;
        self
    }
}

/// A light source on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLight {
    pub position: Vector2,
    pub color: String,
    pub range: f64,
    pub intensity: f64,
    pub shadows: bool,
    pub layer: i32,
    #[serde(rename = "ref")]
    pub node_ref: String,
}

impl MapLight {
    pub fn new(position: Vector2, color: &str, range: f64, node_ref: &str) -> Self {
        Self {
            position,
            color: color.to_string(),
            range,
            intensity: 1.0,
            shadows: true,
            layer: 100,
            node_ref: node_ref.to_string(),
        }
    }
}

/// A wall segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapWall {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// A portal (door/window).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapPortal {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// A pattern (repeating texture region).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapPattern {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// A text label on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapText {
    pub text: String,
    pub position: Vector2,
    pub font_name: String,
    pub font_size: u32,
    pub font_color: String,
    #[serde(default)]
    pub box_shape: i32,
    pub node_id: String,
}
