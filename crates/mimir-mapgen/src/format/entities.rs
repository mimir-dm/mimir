//! Map entity types: objects, paths, lights, walls, patterns, portals, texts.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::godot_types::*;

/// Custom serializer for wall_id: numeric hex strings serialize as integers,
/// non-numeric strings (like "ffffffff") serialize as strings.
/// DD expects `wall_id: 1` for wall-anchored portals and `wall_id: "ffffffff"`
/// for freestanding portals.
fn serialize_wall_id<S: Serializer>(value: &String, serializer: S) -> Result<S::Ok, S::Error> {
    if let Ok(n) = i64::from_str_radix(value, 16) {
        if value != "ffffffff" {
            return serializer.serialize_i64(n);
        }
    }
    serializer.serialize_str(value)
}

/// Custom deserializer for wall_id: accepts both integer and string values.
fn deserialize_wall_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    use serde::de;

    struct WallIdVisitor;

    impl<'de> de::Visitor<'de> for WallIdVisitor {
        type Value = String;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an integer or string wall_id")
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<String, E> {
            Ok(format!("{:x}", v))
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<String, E> {
            Ok(format!("{:x}", v))
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_string())
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<String, E> {
            Ok(v)
        }
    }

    deserializer.deserialize_any(WallIdVisitor)
}

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
    pub node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_color: Option<String>,
    #[serde(default)]
    pub shadow: bool,
}

impl MapObject {
    pub fn new(texture: &str, position: Vector2, node_id: &str) -> Self {
        Self {
            texture: texture.to_string(),
            position,
            scale: Vector2::new(1.0, 1.0),
            rotation: 0.0,
            mirror: false,
            layer: 100,
            node_id: node_id.to_string(),
            custom_color: None,
            shadow: false,
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
    pub node_id: String,
    #[serde(default)]
    pub loop_path: bool,
}

impl MapPath {
    pub fn new(texture: &str, points: Vec<Vector2>, width: f64, node_id: &str) -> Self {
        Self {
            texture: texture.to_string(),
            color: "ffffffff".to_string(),
            points: PoolVector2Array::from_points(points),
            width,
            layer: 100,
            node_id: node_id.to_string(),
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
    pub node_id: String,
}

impl MapLight {
    pub fn new(position: Vector2, color: &str, range: f64, node_id: &str) -> Self {
        Self {
            position,
            color: color.to_string(),
            range,
            intensity: 1.0,
            shadows: true,
            layer: 100,
            node_id: node_id.to_string(),
        }
    }
}

/// A wall segment — a polyline of connected wall segments.
///
/// Walls can be open polylines or closed polygons (rooms).
/// Portals (doors/windows) are embedded inside the wall they belong to.
/// Coordinates are in pixels (256px per grid square).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapWall {
    /// Wall vertices as a polyline in pixel coordinates.
    pub points: PoolVector2Array,
    /// Wall texture path (e.g., "res://textures/walls/battlements.png").
    pub texture: String,
    /// ARGB hex color (e.g., "ff605f58").
    pub color: String,
    /// Whether the wall forms a closed polygon (`true`) or open polyline (`false`).
    #[serde(rename = "loop")]
    pub is_loop: bool,
    /// Wall type: 0 = standard wall.
    #[serde(rename = "type")]
    pub wall_type: i32,
    /// Joint style: 1 = standard.
    pub joint: i32,
    /// UV texture normalization.
    pub normalize_uv: bool,
    /// Whether the wall casts shadows.
    pub shadow: bool,
    /// Unique hex identifier for this wall.
    pub node_id: String,
    /// Portals (doors/windows) embedded in this wall.
    #[serde(default)]
    pub portals: Vec<MapPortal>,
}

impl MapWall {
    /// Create a new closed-polygon wall (room outline) with default settings.
    pub fn new_room(points: Vec<Vector2>, texture: &str, node_id: &str) -> Self {
        Self {
            points: PoolVector2Array::from_points(points),
            texture: texture.to_string(),
            color: "ff605f58".to_string(),
            is_loop: true,
            wall_type: 0,
            joint: 1,
            normalize_uv: true,
            shadow: true,
            node_id: node_id.to_string(),
            portals: Vec::new(),
        }
    }

    /// Create a new open polyline wall with default settings.
    pub fn new_open(points: Vec<Vector2>, texture: &str, node_id: &str) -> Self {
        Self {
            points: PoolVector2Array::from_points(points),
            texture: texture.to_string(),
            color: "ff605f58".to_string(),
            is_loop: false,
            wall_type: 0,
            joint: 1,
            normalize_uv: true,
            shadow: true,
            node_id: node_id.to_string(),
            portals: Vec::new(),
        }
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.color = color.to_string();
        self
    }

    pub fn with_portals(mut self, portals: Vec<MapPortal>) -> Self {
        self.portals = portals;
        self
    }
}

/// A portal (door/window/archway) — an opening in a wall.
///
/// Portals can be wall-attached (stored in `wall.portals[]`) or freestanding
/// (stored in `level.portals[]` with `wall_id: "ffffffff"`).
/// Coordinates are in pixels (256px per grid square).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapPortal {
    /// Center position in pixel coordinates.
    pub position: Vector2,
    /// Rotation in radians: 0 = horizontal (N/S wall), ±π/2 = vertical (E/W wall).
    pub rotation: f64,
    /// Scale factor, typically (1, 1).
    pub scale: Vector2,
    /// Unit normal vector (outward direction from wall).
    pub direction: Vector2,
    /// Portal texture path (cosmetic — determines visual appearance only).
    pub texture: String,
    /// Half-width in pixels (128 = 1 grid square opening, 99.5 = smaller).
    pub radius: f64,
    /// Which wall segment this portal sits on (0-indexed).
    pub point_index: i32,
    /// Parent wall's node_id ("ffffffff" for freestanding portals).
    #[serde(serialize_with = "serialize_wall_id", deserialize_with = "deserialize_wall_id")]
    pub wall_id: String,
    /// Fractional position along wall: integer part = segment index,
    /// fractional part = position within that segment.
    pub wall_distance: f64,
    /// Whether the portal is closed (true) or open (false).
    pub closed: bool,
    /// Unique hex identifier for this portal.
    pub node_id: String,
}

impl MapPortal {
    /// Create a new portal attached to a wall.
    pub fn new(
        position: Vector2,
        rotation: f64,
        direction: Vector2,
        texture: &str,
        radius: f64,
        point_index: i32,
        wall_id: &str,
        wall_distance: f64,
        node_id: &str,
    ) -> Self {
        Self {
            position,
            rotation,
            scale: Vector2::new(1.0, 1.0),
            direction,
            texture: texture.to_string(),
            radius,
            point_index,
            wall_id: wall_id.to_string(),
            wall_distance,
            closed: true,
            node_id: node_id.to_string(),
        }
    }

    /// Create a freestanding portal (not attached to any wall).
    pub fn new_freestanding(
        position: Vector2,
        rotation: f64,
        direction: Vector2,
        texture: &str,
        radius: f64,
        node_id: &str,
    ) -> Self {
        Self {
            position,
            rotation,
            scale: Vector2::new(1.0, 1.0),
            direction,
            texture: texture.to_string(),
            radius,
            point_index: 0,
            wall_id: "ffffffff".to_string(),
            wall_distance: 0.0,
            closed: true,
            node_id: node_id.to_string(),
        }
    }

    pub fn with_closed(mut self, closed: bool) -> Self {
        self.closed = closed;
        self
    }
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
