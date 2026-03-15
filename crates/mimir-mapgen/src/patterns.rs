//! Pattern placement for Dungeondraft maps.
//!
//! Patterns are polygon-bounded texture fills — used for floor tiles inside rooms,
//! water overlays, courtyard surfaces, and noise-gated ground detail.
//! Each produces a `MapPattern` entry pushed to `Level.patterns`.

use serde::{Deserialize, Serialize};

use crate::contour;
use crate::format::entities::MapPattern;
use crate::format::godot_types::Vector2;
use crate::format::NodeIdAllocator;
use crate::noise_gen::NoiseMap;
use crate::pipeline::GeneratedFeatures;

/// Configuration for a pattern placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    /// Region to fill with the pattern.
    pub region: PatternRegion,
    /// Tileset texture path.
    pub texture: String,
    /// Color tint (ARGB hex, use alpha for transparency).
    #[serde(default = "default_color")]
    pub color: String,
    /// Texture rotation in degrees.
    #[serde(default)]
    pub rotation: i32,
    /// DD layer.
    #[serde(default = "default_layer")]
    pub layer: i32,
    /// Outline mode instead of fill.
    #[serde(default)]
    pub outline: bool,
}

fn default_color() -> String {
    "ffffffff".to_string()
}

fn default_layer() -> i32 {
    100
}

/// Region type for pattern placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PatternRegion {
    /// Fill generated water polygon areas.
    Water,
    /// Fill a named room's boundary.
    Room {
        /// Room ID to fill.
        name: String,
    },
    /// Fill a named polygon's boundary.
    Polygon {
        /// Polygon ID to fill.
        name: String,
    },
    /// Fill area where noise is in a threshold range (uses marching squares contour).
    Noise {
        /// Lower noise bound.
        noise_lower: f64,
        /// Upper noise bound.
        noise_upper: f64,
    },
}

/// Generate patterns from configs.
pub fn generate_patterns(
    configs: &[PatternConfig],
    noise_map: &NoiseMap,
    features: &GeneratedFeatures,
    pixel_width: f64,
    pixel_height: f64,
    alloc: &NodeIdAllocator,
) -> Vec<MapPattern> {
    let mut patterns = Vec::new();

    for config in configs {
        match &config.region {
            PatternRegion::Water => {
                for water_poly in &features.water_polygons {
                    if water_poly.len() >= 3 {
                        let vectors = to_vectors(water_poly);
                        patterns.push(make_pattern(config, vectors, alloc));
                    }
                }
            }

            PatternRegion::Room { name } => {
                if let Some(room) = features.get_room(name) {
                    if room.boundary.len() >= 3 {
                        let vectors = to_vectors(&room.boundary);
                        patterns.push(make_pattern(config, vectors, alloc));
                    }
                }
            }

            PatternRegion::Polygon { name } => {
                if let Some(boundary) = features.get_polygon(name) {
                    if boundary.len() >= 3 {
                        let vectors = to_vectors(boundary);
                        patterns.push(make_pattern(config, vectors, alloc));
                    }
                }
            }

            PatternRegion::Noise {
                noise_lower,
                noise_upper,
            } => {
                // Use marching squares to find contour polygons at the midpoint threshold
                let threshold = (noise_lower + noise_upper) / 2.0;
                let contours = contour::find_contours(noise_map, threshold);
                let min_points = 6;

                // Scale noise grid coords to pixel coords
                let scale_x = pixel_width / (noise_map.width - 1) as f64;
                let scale_y = pixel_height / (noise_map.height - 1) as f64;

                for contour_points in contours {
                    if contour_points.len() < min_points {
                        continue;
                    }
                    let pixel_points: Vec<(f64, f64)> = contour_points
                        .iter()
                        .map(|&(x, y)| (x * scale_x, y * scale_y))
                        .collect();
                    let vectors = to_vectors(&pixel_points);
                    patterns.push(make_pattern(config, vectors, alloc));
                }
            }
        }
    }

    patterns
}

/// Convert (f64, f64) tuples to Vector2 list.
fn to_vectors(points: &[(f64, f64)]) -> Vec<Vector2> {
    points.iter().map(|&(x, y)| Vector2::new(x, y)).collect()
}

/// Create a MapPattern from config and boundary polygon.
fn make_pattern(config: &PatternConfig, points: Vec<Vector2>, alloc: &NodeIdAllocator) -> MapPattern {
    MapPattern::new(&config.texture, points, &config.color, &alloc.next())
        .with_layer(config.layer)
        .with_rotation(config.rotation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::RoomGeometry;

    #[test]
    fn test_water_region() {
        let mut features = GeneratedFeatures::default();
        features.water_polygons.push(vec![
            (100.0, 100.0),
            (500.0, 100.0),
            (500.0, 500.0),
            (100.0, 500.0),
        ]);
        let noise = NoiseMap::generate(40, 40, &Default::default());
        let alloc = NodeIdAllocator::new(1);

        let configs = vec![PatternConfig {
            region: PatternRegion::Water,
            texture: "res://textures/tilesets/water_ripple.png".to_string(),
            color: "48ffffff".to_string(),
            rotation: 30,
            layer: 100,
            outline: false,
        }];

        let patterns = generate_patterns(&configs, &noise, &features, 2560.0, 2560.0, &alloc);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].color, "48ffffff");
        assert_eq!(patterns[0].rotation, 30);
    }

    #[test]
    fn test_room_region() {
        let mut features = GeneratedFeatures::default();
        features.rooms.insert("hall".to_string(), RoomGeometry {
            center: (640.0, 640.0),
            boundary: vec![(256.0, 256.0), (1024.0, 256.0), (1024.0, 1024.0), (256.0, 1024.0)],
        });
        let noise = NoiseMap::generate(40, 40, &Default::default());
        let alloc = NodeIdAllocator::new(1);

        let configs = vec![PatternConfig {
            region: PatternRegion::Room { name: "hall".to_string() },
            texture: "res://textures/tilesets/simple/tileset_cobble.png".to_string(),
            color: "ff929292".to_string(),
            rotation: 0,
            layer: 100,
            outline: false,
        }];

        let patterns = generate_patterns(&configs, &noise, &features, 2560.0, 2560.0, &alloc);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].texture, "res://textures/tilesets/simple/tileset_cobble.png");
    }

    #[test]
    fn test_polygon_region() {
        let mut features = GeneratedFeatures::default();
        features.polygons.insert("courtyard".to_string(), vec![
            (256.0, 256.0), (768.0, 256.0), (768.0, 768.0), (256.0, 768.0),
        ]);
        let noise = NoiseMap::generate(40, 40, &Default::default());
        let alloc = NodeIdAllocator::new(1);

        let configs = vec![PatternConfig {
            region: PatternRegion::Polygon { name: "courtyard".to_string() },
            texture: "res://textures/tilesets/gravel.png".to_string(),
            color: "ffffffff".to_string(),
            rotation: 0,
            layer: -100,
            outline: false,
        }];

        let patterns = generate_patterns(&configs, &noise, &features, 2560.0, 2560.0, &alloc);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].layer, -100);
    }

    #[test]
    fn test_noise_region() {
        let noise = NoiseMap::generate(40, 40, &Default::default());
        let features = GeneratedFeatures::default();
        let alloc = NodeIdAllocator::new(1);

        let configs = vec![PatternConfig {
            region: PatternRegion::Noise {
                noise_lower: 0.3,
                noise_upper: 0.7,
            },
            texture: "res://textures/tilesets/mud.png".to_string(),
            color: "80ffffff".to_string(),
            rotation: 0,
            layer: 100,
            outline: false,
        }];

        let patterns = generate_patterns(&configs, &noise, &features, 2560.0, 2560.0, &alloc);
        // Should find some contours at the midpoint threshold
        assert!(!patterns.is_empty(), "Noise region should produce contour patterns");
    }

    #[test]
    fn test_missing_reference() {
        let features = GeneratedFeatures::default();
        let noise = NoiseMap::generate(40, 40, &Default::default());
        let alloc = NodeIdAllocator::new(1);

        let configs = vec![
            PatternConfig {
                region: PatternRegion::Room { name: "nope".to_string() },
                texture: "test.png".to_string(),
                color: "ffffffff".to_string(),
                rotation: 0,
                layer: 100,
                outline: false,
            },
            PatternConfig {
                region: PatternRegion::Polygon { name: "nope".to_string() },
                texture: "test.png".to_string(),
                color: "ffffffff".to_string(),
                rotation: 0,
                layer: 100,
                outline: false,
            },
        ];

        let patterns = generate_patterns(&configs, &noise, &features, 2560.0, 2560.0, &alloc);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_multiple_water_polygons() {
        let mut features = GeneratedFeatures::default();
        features.water_polygons.push(vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0)]);
        features.water_polygons.push(vec![(500.0, 500.0), (600.0, 500.0), (600.0, 600.0)]);
        let noise = NoiseMap::generate(40, 40, &Default::default());
        let alloc = NodeIdAllocator::new(1);

        let configs = vec![PatternConfig {
            region: PatternRegion::Water,
            texture: "water.png".to_string(),
            color: "ffffffff".to_string(),
            rotation: 0,
            layer: 100,
            outline: false,
        }];

        let patterns = generate_patterns(&configs, &noise, &features, 2560.0, 2560.0, &alloc);
        assert_eq!(patterns.len(), 2, "Should produce one pattern per water polygon");
    }
}
