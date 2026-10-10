//! The geometry of a UVTT map: walls, doors and lights, in grid units.
//!
//! A UVTT file (Universal VTT, `.uvtt` / `.dd2vtt`) is JSON with the map
//! image inside it as base64. The parser reads only the geometry; serde
//! skips the image.

use serde::{Deserialize, Serialize};

/// A point in grid units (1.0 = one cell).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GridPoint {
    pub x: f64,
    pub y: f64,
}

/// A door.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UvttPortal {
    pub position: GridPoint,
    pub bounds: [GridPoint; 2],
    pub rotation: f64,
    pub closed: bool,
    pub freestanding: bool,
}

/// A light drawn into the map.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UvttLight {
    pub position: GridPoint,
    /// Range in grid units.
    pub range: f64,
    pub intensity: f64,
    /// RRGGBBAA hex, as in the file.
    pub color: String,
    pub shadows: bool,
}

/// What blocks sight and light on a map.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapGeometry {
    /// Pixels per grid cell in the served image (scaled when the image was
    /// resized).
    pub pixels_per_grid: f64,
    /// Grid columns and rows.
    pub columns: f64,
    pub rows: f64,
    /// Walls: each is a polyline.
    pub walls: Vec<Vec<GridPoint>>,
    pub portals: Vec<UvttPortal>,
    pub lights: Vec<UvttLight>,
    /// The map's ambient light (RRGGBBAA hex), if the file has one.
    pub ambient_light: Option<String>,
    pub baked_lighting: bool,
}

#[derive(Deserialize)]
struct RawFile {
    #[serde(default)]
    resolution: Option<RawResolution>,
    #[serde(default)]
    line_of_sight: Vec<Vec<GridPoint>>,
    #[serde(default)]
    portals: Vec<RawPortal>,
    #[serde(default)]
    lights: Vec<RawLight>,
    #[serde(default)]
    environment: Option<RawEnvironment>,
}

#[derive(Deserialize)]
struct RawResolution {
    #[serde(default)]
    pixels_per_grid: Option<f64>,
    #[serde(default)]
    map_size: Option<GridPoint>,
}

#[derive(Deserialize)]
struct RawPortal {
    position: GridPoint,
    bounds: Vec<GridPoint>,
    #[serde(default)]
    rotation: f64,
    #[serde(default = "yes")]
    closed: bool,
    #[serde(default)]
    freestanding: bool,
}

#[derive(Deserialize)]
struct RawLight {
    position: GridPoint,
    #[serde(default = "five")]
    range: f64,
    #[serde(default = "one")]
    intensity: f64,
    #[serde(default = "white")]
    color: String,
    #[serde(default)]
    shadows: bool,
}

#[derive(Deserialize)]
struct RawEnvironment {
    #[serde(default)]
    baked_lighting: bool,
    #[serde(default)]
    ambient_light: Option<String>,
}

fn yes() -> bool {
    true
}
fn five() -> f64 {
    5.0
}
fn one() -> f64 {
    1.0
}
fn white() -> String {
    "ffffffff".to_string()
}

/// Parse the geometry of a UVTT file. `scaled_ppg` is the pixels per grid of
/// the served image when known (the resolution sidecar); else the file's.
pub fn parse_geometry(bytes: &[u8], scaled_ppg: Option<f64>) -> Result<MapGeometry, String> {
    let raw: RawFile =
        serde_json::from_slice(bytes).map_err(|e| format!("not a UVTT file: {e}"))?;
    let resolution = raw.resolution.as_ref();
    let size = resolution.and_then(|r| r.map_size);
    let file_ppg = resolution.and_then(|r| r.pixels_per_grid).unwrap_or(70.0);
    let portals = raw
        .portals
        .into_iter()
        .filter(|p| p.bounds.len() >= 2)
        .map(|p| UvttPortal {
            position: p.position,
            bounds: [p.bounds[0], p.bounds[1]],
            rotation: p.rotation,
            closed: p.closed,
            freestanding: p.freestanding,
        })
        .collect();
    let lights = raw
        .lights
        .into_iter()
        .map(|l| UvttLight {
            position: l.position,
            range: l.range,
            intensity: l.intensity,
            color: l.color,
            shadows: l.shadows,
        })
        .collect();
    let (ambient_light, baked_lighting) = match raw.environment {
        Some(env) => (env.ambient_light, env.baked_lighting),
        None => (None, false),
    };
    Ok(MapGeometry {
        pixels_per_grid: scaled_ppg.unwrap_or(file_ppg),
        columns: size.map(|s| s.x).unwrap_or(25.0),
        rows: size.map(|s| s.y).unwrap_or(25.0),
        walls: raw.line_of_sight,
        portals,
        lights,
        ambient_light,
        baked_lighting,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_walls_doors_and_lights() {
        let json = br#"{
            "format": 0.3,
            "resolution": {"map_origin": {"x": 0, "y": 0}, "map_size": {"x": 10, "y": 8}, "pixels_per_grid": 140},
            "line_of_sight": [[{"x": 0, "y": 0}, {"x": 10, "y": 0}], [{"x": 2, "y": 2}, {"x": 2, "y": 5}, {"x": 4, "y": 5}]],
            "portals": [{"position": {"x": 3, "y": 0}, "bounds": [{"x": 2.5, "y": 0}, {"x": 3.5, "y": 0}], "rotation": 0, "closed": false, "freestanding": false}],
            "lights": [{"position": {"x": 5, "y": 5}, "range": 6, "intensity": 1, "color": "ffeeaa88", "shadows": true}],
            "environment": {"baked_lighting": true, "ambient_light": "ff202020"},
            "image": "AAAA"
        }"#;
        let g = parse_geometry(json, Some(70.0)).unwrap();
        assert_eq!(g.pixels_per_grid, 70.0, "the scaled value wins");
        assert_eq!((g.columns, g.rows), (10.0, 8.0));
        assert_eq!(g.walls.len(), 2);
        assert_eq!(g.walls[1].len(), 3);
        assert_eq!(g.portals.len(), 1);
        assert!(!g.portals[0].closed);
        assert_eq!(g.lights[0].range, 6.0);
        assert_eq!(g.ambient_light.as_deref(), Some("ff202020"));
        assert!(g.baked_lighting);
    }

    #[test]
    fn missing_parts_have_defaults() {
        let g = parse_geometry(br#"{"resolution": {"pixels_per_grid": 100}}"#, None).unwrap();
        assert_eq!(g.pixels_per_grid, 100.0);
        assert_eq!((g.columns, g.rows), (25.0, 25.0));
        assert!(g.walls.is_empty() && g.portals.is_empty() && g.lights.is_empty());
        assert!(parse_geometry(b"not json", None).is_err());
    }
}
