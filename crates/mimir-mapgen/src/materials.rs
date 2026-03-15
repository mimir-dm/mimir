//! Material scatter generation for Dungeondraft maps.
//!
//! Materials are bit-packed bitmaps representing ground-level detail like ice,
//! lava, acid, and debris. Each produces a `MaterialEntry` in `Level.materials`
//! keyed by DD layer string.
//!
//! Bitmap encoding: `(map_width * 2 + 3) × (map_height * 2 + 3)` cells,
//! each cell = 0.5 grid squares, +3 border for blending.
//! Flat bit-packed, LSB-first within each byte.

use serde::{Deserialize, Serialize};

use crate::format::world::MaterialEntry;
use crate::format::NodeIdAllocator;
use crate::noise_gen::NoiseMap;
use crate::pipeline::GeneratedFeatures;

use std::collections::BTreeMap;

/// Configuration for a material scatter placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialScatterConfig {
    /// Region to fill with the material.
    pub region: MaterialRegion,
    /// Material texture path.
    pub texture: String,
    /// DD layer (as string, e.g., "-400" for Below Ground).
    #[serde(default = "default_layer")]
    pub layer: String,
    /// Edge smoothing.
    #[serde(default = "default_true")]
    pub smooth: bool,
}

fn default_layer() -> String {
    "-400".to_string()
}

fn default_true() -> bool {
    true
}

/// Region type for material scatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MaterialRegion {
    /// Scatter where noise falls in a range.
    Noise {
        /// Lower noise bound.
        noise_lower: f64,
        /// Upper noise bound.
        noise_upper: f64,
    },
    /// Fill inside a named room.
    Room {
        /// Room ID.
        name: String,
    },
    /// Fill inside a named polygon.
    Polygon {
        /// Polygon ID.
        name: String,
    },
    /// Scatter within a distance of a named path.
    AlongPath {
        /// Path feature name.
        path: String,
        /// Corridor half-width in grid squares.
        width: f64,
    },
}

/// Generate material scatter from configs.
///
/// Returns a `BTreeMap<String, Vec<MaterialEntry>>` keyed by layer ID string,
/// to be merged into `Level.materials`.
pub fn generate_materials(
    configs: &[MaterialScatterConfig],
    noise_map: &NoiseMap,
    features: &GeneratedFeatures,
    map_width: u32,
    map_height: u32,
    _alloc: &NodeIdAllocator,
) -> BTreeMap<String, Vec<MaterialEntry>> {
    let cell_w = map_width * 2 + 3;
    let cell_h = map_height * 2 + 3;
    // Each cell is 0.5 grid squares = 128 pixels
    let cell_size_px = 128.0;
    // Border offset: 1.5 cells on each side (the +3 in the formula)
    let border_offset = 1.5;

    let pixel_width = map_width as f64 * 256.0;
    let pixel_height = map_height as f64 * 256.0;

    let mut result: BTreeMap<String, Vec<MaterialEntry>> = BTreeMap::new();

    for config in configs {
        let mut entry = MaterialEntry::new(&config.texture, map_width, map_height);
        entry.smooth = config.smooth;

        for cy in 0..cell_h {
            for cx in 0..cell_w {
                // Convert cell coords to pixel coords (accounting for border offset)
                let px = (cx as f64 - border_offset) * cell_size_px;
                let py = (cy as f64 - border_offset) * cell_size_px;

                let should_set = match &config.region {
                    MaterialRegion::Noise {
                        noise_lower,
                        noise_upper,
                    } => {
                        if px < 0.0 || py < 0.0 || px >= pixel_width || py >= pixel_height {
                            false
                        } else {
                            let nx = px / pixel_width * (noise_map.width - 1) as f64;
                            let ny = py / pixel_height * (noise_map.height - 1) as f64;
                            let val = noise_map.sample(nx, ny);
                            val >= *noise_lower && val <= *noise_upper
                        }
                    }

                    MaterialRegion::Room { name } => {
                        if let Some(room) = features.get_room(name) {
                            point_in_polygon(px, py, &room.boundary)
                        } else {
                            false
                        }
                    }

                    MaterialRegion::Polygon { name } => {
                        if let Some(boundary) = features.get_polygon(name) {
                            point_in_polygon(px, py, boundary)
                        } else {
                            false
                        }
                    }

                    MaterialRegion::AlongPath { path, width } => {
                        if let Some(path_points) = features.get_path(path) {
                            let corridor_px = width * 256.0;
                            distance_to_polyline(px, py, path_points) <= corridor_px
                        } else {
                            false
                        }
                    }
                };

                if should_set {
                    entry.set_bit(cx, cy, cell_w);
                }
            }
        }

        result
            .entry(config.layer.clone())
            .or_default()
            .push(entry);
    }

    result
}

/// Point-in-polygon test using ray casting.
fn point_in_polygon(x: f64, y: f64, polygon: &[(f64, f64)]) -> bool {
    let n = polygon.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Minimum distance from a point to a polyline.
fn distance_to_polyline(px: f64, py: f64, points: &[(f64, f64)]) -> f64 {
    let mut min_dist = f64::MAX;
    for window in points.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len_sq = dx * dx + dy * dy;
        if len_sq < f64::EPSILON {
            let d = ((px - x0).powi(2) + (py - y0).powi(2)).sqrt();
            min_dist = min_dist.min(d);
            continue;
        }
        let t = ((px - x0) * dx + (py - y0) * dy) / len_sq;
        let t = t.clamp(0.0, 1.0);
        let closest_x = x0 + t * dx;
        let closest_y = y0 + t * dy;
        let d = ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt();
        min_dist = min_dist.min(d);
    }
    min_dist
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::RoomGeometry;

    #[test]
    fn test_bitmap_dimensions() {
        let entry = MaterialEntry::new("test.png", 35, 20);
        let cell_w = 35 * 2 + 3; // 73
        let cell_h = 20 * 2 + 3; // 43
        let expected_bytes = (cell_w * cell_h + 7) / 8; // 393
        assert_eq!(entry.bitmap.0.len(), expected_bytes as usize);
    }

    #[test]
    fn test_bitmap_dimensions_square() {
        let entry = MaterialEntry::new("test.png", 32, 32);
        let cell_w = 32 * 2 + 3; // 67
        let cell_h = 32 * 2 + 3; // 67
        let expected_bytes = (cell_w * cell_h + 7) / 8; // 561
        assert_eq!(entry.bitmap.0.len(), expected_bytes as usize);
    }

    #[test]
    fn test_set_bit() {
        let mut entry = MaterialEntry::new("test.png", 10, 10);
        let cell_w = 10 * 2 + 3; // 23
        entry.set_bit(5, 3, cell_w);
        // bit index = 3 * 23 + 5 = 74
        // byte 74/8 = 9, bit 74%8 = 2
        assert_eq!(entry.bitmap.0[9] & (1 << 2), 1 << 2);
    }

    #[test]
    fn test_noise_region() {
        let noise = NoiseMap::generate(40, 40, &Default::default());
        let features = GeneratedFeatures::default();
        let alloc = NodeIdAllocator::new(1);

        let configs = vec![MaterialScatterConfig {
            region: MaterialRegion::Noise {
                noise_lower: 0.0,
                noise_upper: 0.5,
            },
            texture: "res://textures/materials/ice_tile.png".to_string(),
            layer: "-400".to_string(),
            smooth: true,
        }];

        let result = generate_materials(&configs, &noise, &features, 10, 10, &alloc);
        assert!(result.contains_key("-400"));
        let entries = &result["-400"];
        assert_eq!(entries.len(), 1);
        // Should have some bits set
        let nonzero = entries[0].bitmap.0.iter().filter(|&&b| b != 0).count();
        assert!(nonzero > 0, "Noise region should set some bits");
    }

    #[test]
    fn test_room_region() {
        let noise = NoiseMap::generate(40, 40, &Default::default());
        let mut features = GeneratedFeatures::default();
        features.rooms.insert("lava_room".to_string(), RoomGeometry {
            center: (1280.0, 1280.0),
            boundary: vec![
                (1024.0, 1024.0), (1536.0, 1024.0),
                (1536.0, 1536.0), (1024.0, 1536.0),
            ],
        });
        let alloc = NodeIdAllocator::new(1);

        let configs = vec![MaterialScatterConfig {
            region: MaterialRegion::Room { name: "lava_room".to_string() },
            texture: "res://textures/materials/lava_tile.png".to_string(),
            layer: "-400".to_string(),
            smooth: true,
        }];

        let result = generate_materials(&configs, &noise, &features, 10, 10, &alloc);
        let entries = &result["-400"];
        assert_eq!(entries.len(), 1);
        let nonzero = entries[0].bitmap.0.iter().filter(|&&b| b != 0).count();
        assert!(nonzero > 0, "Room region should set bits inside room");
    }

    #[test]
    fn test_along_path_region() {
        let noise = NoiseMap::generate(40, 40, &Default::default());
        let mut features = GeneratedFeatures::default();
        features.paths.insert(
            "river".to_string(),
            vec![(0.0, 1280.0), (2560.0, 1280.0)],
        );
        let alloc = NodeIdAllocator::new(1);

        let configs = vec![MaterialScatterConfig {
            region: MaterialRegion::AlongPath {
                path: "river".to_string(),
                width: 1.0,
            },
            texture: "res://textures/materials/acid_tile.png".to_string(),
            layer: "-400".to_string(),
            smooth: true,
        }];

        let result = generate_materials(&configs, &noise, &features, 10, 10, &alloc);
        let entries = &result["-400"];
        assert_eq!(entries.len(), 1);
        let nonzero = entries[0].bitmap.0.iter().filter(|&&b| b != 0).count();
        assert!(nonzero > 0, "Along-path region should set bits near path");
    }

    #[test]
    fn test_missing_reference() {
        let noise = NoiseMap::generate(40, 40, &Default::default());
        let features = GeneratedFeatures::default();
        let alloc = NodeIdAllocator::new(1);

        let configs = vec![MaterialScatterConfig {
            region: MaterialRegion::Room { name: "nope".to_string() },
            texture: "test.png".to_string(),
            layer: "-400".to_string(),
            smooth: true,
        }];

        let result = generate_materials(&configs, &noise, &features, 10, 10, &alloc);
        // Still creates an entry, but bitmap should be all zeros
        let entries = &result["-400"];
        assert_eq!(entries.len(), 1);
        let nonzero = entries[0].bitmap.0.iter().filter(|&&b| b != 0).count();
        assert_eq!(nonzero, 0, "Missing room should produce empty bitmap");
    }

    #[test]
    fn test_point_in_polygon() {
        let square = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        assert!(point_in_polygon(5.0, 5.0, &square));
        assert!(!point_in_polygon(15.0, 5.0, &square));
        assert!(!point_in_polygon(-1.0, 5.0, &square));
    }

    #[test]
    fn test_distance_to_polyline() {
        let line = vec![(0.0, 0.0), (10.0, 0.0)];
        assert!((distance_to_polyline(5.0, 3.0, &line) - 3.0).abs() < 0.01);
        assert!((distance_to_polyline(0.0, 0.0, &line) - 0.0).abs() < 0.01);
        assert!((distance_to_polyline(15.0, 0.0, &line) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_multiple_layers() {
        let noise = NoiseMap::generate(40, 40, &Default::default());
        let features = GeneratedFeatures::default();
        let alloc = NodeIdAllocator::new(1);

        let configs = vec![
            MaterialScatterConfig {
                region: MaterialRegion::Noise { noise_lower: 0.0, noise_upper: 0.3 },
                texture: "ice.png".to_string(),
                layer: "-400".to_string(),
                smooth: true,
            },
            MaterialScatterConfig {
                region: MaterialRegion::Noise { noise_lower: 0.7, noise_upper: 1.0 },
                texture: "lava.png".to_string(),
                layer: "-400".to_string(),
                smooth: true,
            },
        ];

        let result = generate_materials(&configs, &noise, &features, 10, 10, &alloc);
        let entries = &result["-400"];
        assert_eq!(entries.len(), 2, "Should have two materials on same layer");
    }
}
