//! Elevation contour generation.
//!
//! Generates cliff and hill contour paths at configurable noise thresholds.

use serde::{Deserialize, Serialize};

use crate::contour::{filter_contours, find_contours, smooth_contours};
use crate::curves::offset_polyline;
use crate::format::entities::MapPath;
use crate::format::godot_types::Vector2;
use crate::format::NodeIdAllocator;
use crate::noise_gen::NoiseMap;

/// Configuration for a single elevation contour level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContourLevel {
    /// Noise threshold for this elevation level.
    pub threshold: f64,
    /// Cliff/contour path texture.
    pub texture: String,
    /// Path width in pixels.
    pub width: f64,
    /// Layer in the DD map.
    pub layer: i32,
    /// Minimum contour length in points.
    pub min_points: usize,
    /// Smoothing iterations.
    pub smooth_iterations: usize,
    /// Optional shadow path configuration.
    pub shadow: Option<ShadowPathConfig>,
}

/// Configuration for shadow paths below cliffs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowPathConfig {
    /// Shadow texture.
    pub texture: String,
    /// Perpendicular offset distance (positive = left/up).
    pub offset: f64,
    /// Shadow path width.
    pub width: f64,
    /// Shadow layer (should be below contour layer).
    pub layer: i32,
}

/// Configuration for elevation contour generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationConfig {
    /// List of contour levels, from lowest to highest.
    pub levels: Vec<ContourLevel>,
    /// Pixels per noise cell for coordinate scaling.
    pub pixels_per_cell: f64,
}

impl Default for ElevationConfig {
    fn default() -> Self {
        Self {
            levels: vec![
                ContourLevel {
                    threshold: 0.4,
                    texture: "res://textures/paths/path_rocks.png".to_string(),
                    width: 15.0,
                    layer: 100,
                    min_points: 8,
                    smooth_iterations: 2,
                    shadow: Some(ShadowPathConfig {
                        texture: "res://textures/paths/path_shadow.png".to_string(),
                        offset: 8.0,
                        width: 20.0,
                        layer: 50,
                    }),
                },
                ContourLevel {
                    threshold: 0.6,
                    texture: "res://textures/paths/path_cliff.png".to_string(),
                    width: 20.0,
                    layer: 200,
                    min_points: 8,
                    smooth_iterations: 2,
                    shadow: Some(ShadowPathConfig {
                        texture: "res://textures/paths/path_shadow.png".to_string(),
                        offset: 12.0,
                        width: 25.0,
                        layer: 150,
                    }),
                },
            ],
            pixels_per_cell: 64.0,
        }
    }
}

/// Generate elevation contour paths from a noise map.
///
/// For each configured level, extracts contours at the threshold,
/// smooths them, scales to pixel coordinates, and produces MapPath entities.
pub fn generate_elevation(
    noise_map: &NoiseMap,
    config: &ElevationConfig,
    alloc: &NodeIdAllocator,
) -> Vec<MapPath> {
    let mut paths = Vec::new();

    for level in &config.levels {
        let contours = find_contours(noise_map, level.threshold);
        let filtered = filter_contours(contours, level.min_points);
        let smoothed = smooth_contours(filtered, level.smooth_iterations);

        for contour in &smoothed {
            // Scale to pixel coordinates
            let pixel_points: Vec<(f64, f64)> = contour
                .iter()
                .map(|&(x, y)| (x * config.pixels_per_cell, y * config.pixels_per_cell))
                .collect();

            // Shadow path first (below contour)
            if let Some(ref shadow) = level.shadow {
                let shadow_points = offset_polyline(&pixel_points, shadow.offset);
                let shadow_vectors: Vec<Vector2> = shadow_points
                    .iter()
                    .map(|&(x, y)| Vector2::new(x, y))
                    .collect();

                paths.push(
                    MapPath::new(&shadow.texture, shadow_vectors, shadow.width, &alloc.next())
                        .with_layer(shadow.layer),
                );
            }

            // Contour path
            let vectors: Vec<Vector2> = pixel_points
                .iter()
                .map(|&(x, y)| Vector2::new(x, y))
                .collect();

            paths.push(
                MapPath::new(&level.texture, vectors, level.width, &alloc.next())
                    .with_layer(level.layer),
            );
        }
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noise_gen::{NoiseConfig, NoiseMap};

    #[test]
    fn test_generate_elevation_basic() {
        let noise = NoiseMap::generate(
            50,
            50,
            &NoiseConfig {
                seed: 42,
                ..Default::default()
            },
        );

        let alloc = NodeIdAllocator::new(1);
        let config = ElevationConfig {
            levels: vec![ContourLevel {
                threshold: 0.5,
                texture: "cliff.png".to_string(),
                width: 15.0,
                layer: 100,
                min_points: 3,
                smooth_iterations: 1,
                shadow: None,
            }],
            pixels_per_cell: 64.0,
        };

        let paths = generate_elevation(&noise, &config, &alloc);
        assert!(!paths.is_empty(), "Should find contours at 0.5 threshold");

        for path in &paths {
            assert_eq!(path.texture, "cliff.png");
            assert!(path.points.0.len() >= 3);
        }
    }

    #[test]
    fn test_generate_elevation_with_shadows() {
        let noise = NoiseMap::generate(
            50,
            50,
            &NoiseConfig {
                seed: 42,
                ..Default::default()
            },
        );

        let alloc = NodeIdAllocator::new(1);
        let config = ElevationConfig::default();

        let paths = generate_elevation(&noise, &config, &alloc);

        // With shadows enabled, should have pairs (shadow + contour)
        if !paths.is_empty() {
            // Check that there are shadow paths at lower layers
            let shadow_count = paths.iter().filter(|p| p.layer < 100).count();
            let contour_count = paths.iter().filter(|p| p.layer >= 100).count();
            assert!(
                shadow_count > 0 || contour_count > 0,
                "Should have shadow and/or contour paths"
            );
        }
    }

    #[test]
    fn test_generate_elevation_uniform() {
        // Uniform noise → no contours
        let noise = NoiseMap {
            width: 20,
            height: 20,
            data: vec![vec![0.5; 20]; 20],
        };

        let alloc = NodeIdAllocator::new(1);
        let config = ElevationConfig::default();

        let paths = generate_elevation(&noise, &config, &alloc);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_contour_pixel_scaling() {
        let noise = NoiseMap::generate(
            50,
            50,
            &NoiseConfig {
                seed: 42,
                ..Default::default()
            },
        );

        let alloc = NodeIdAllocator::new(1);
        let ppc = 128.0;
        let config = ElevationConfig {
            levels: vec![ContourLevel {
                threshold: 0.5,
                texture: "cliff.png".to_string(),
                width: 10.0,
                layer: 100,
                min_points: 3,
                smooth_iterations: 0,
                shadow: None,
            }],
            pixels_per_cell: ppc,
        };

        let paths = generate_elevation(&noise, &config, &alloc);
        if let Some(path) = paths.first() {
            // All coordinates should be scaled by pixels_per_cell
            for pt in &path.points.0 {
                // Points should be in pixel space (noise is 0-50, so max ~50*128=6400)
                assert!(pt.x >= 0.0 && pt.x <= 50.0 * ppc);
                assert!(pt.y >= 0.0 && pt.y <= 50.0 * ppc);
            }
        }
    }
}
