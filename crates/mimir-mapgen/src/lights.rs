//! Point light placement for Dungeondraft maps.
//!
//! Three placement modes:
//! - **Scatter**: Poisson disc placement gated by noise bounds + probability.
//! - **Along path**: Sample points at intervals along a named path feature.
//! - **With objects**: One light per placed object in a named group.

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::distribution::PoissonDisc;
use crate::format::entities::MapLight;
use crate::format::godot_types::Vector2;
use crate::format::NodeIdAllocator;
use crate::noise_gen::NoiseMap;
use crate::pipeline::GeneratedFeatures;

/// Configuration for a single light placement group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightConfig {
    /// Placement mode.
    pub placement: LightPlacement,
    /// Light color (ARGB hex, e.g., "ffeaefca").
    pub color: String,
    /// Light intensity (0.0–1.0 typical).
    pub intensity: f64,
    /// Light range in grid squares (converted to pixels internally).
    pub range: f64,
    /// Whether the light casts shadows.
    #[serde(default)]
    pub shadows: bool,
    /// DD layer for the light.
    #[serde(default = "default_layer")]
    pub layer: i32,
}

fn default_layer() -> i32 {
    100
}

/// How lights are placed on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LightPlacement {
    /// Scatter lights across the map using Poisson disc + noise gating.
    Scatter {
        /// Minimum distance between lights in grid squares.
        density: f64,
        /// Lower noise bound for placement (0.0–1.0).
        #[serde(default)]
        noise_lower: f64,
        /// Upper noise bound for placement (0.0–1.0).
        #[serde(default = "default_one")]
        noise_upper: f64,
        /// Probability of keeping each valid point (0.0–1.0).
        #[serde(default = "default_one")]
        probability: f64,
        /// Margin from map edge as fraction (0.0–0.5).
        #[serde(default = "default_margin")]
        margin: f64,
    },
    /// Place lights at intervals along a named path feature.
    AlongPath {
        /// Name of the path feature to follow (road id, river id, etc.).
        path: String,
        /// Lights per grid square of path length.
        density: f64,
    },
    /// Place a light at every object in a named group.
    WithObjects {
        /// Name of the object group (tree id, clutter id, etc.).
        group: String,
    },
}

fn default_one() -> f64 {
    1.0
}

fn default_margin() -> f64 {
    0.1
}

/// Generate lights from a list of configs.
///
/// Returns a `Vec<MapLight>` to be pushed to `Level.lights`.
pub fn generate_lights(
    configs: &[LightConfig],
    noise_map: &NoiseMap,
    features: &GeneratedFeatures,
    pixel_width: f64,
    pixel_height: f64,
    alloc: &NodeIdAllocator,
    rng: &mut ChaCha8Rng,
) -> Vec<MapLight> {
    let mut lights = Vec::new();

    for config in configs {
        match &config.placement {
            LightPlacement::Scatter {
                density,
                noise_lower,
                noise_upper,
                probability,
                margin,
            } => {
                let min_distance = density * 256.0; // grid squares to pixels
                let margin_px = margin * pixel_width.min(pixel_height);

                let candidates = PoissonDisc::sample(
                    pixel_width,
                    pixel_height,
                    min_distance,
                    rng,
                    30,
                );

                for (x, y) in candidates {
                    // Skip margin zone
                    if x < margin_px
                        || y < margin_px
                        || x > pixel_width - margin_px
                        || y > pixel_height - margin_px
                    {
                        continue;
                    }

                    // Noise gate — convert pixel coords to noise grid coords
                    // Noise grid is 4 cells per grid square, pixel space is 256px per grid square
                    let nx = x / pixel_width * (noise_map.width - 1) as f64;
                    let ny = y / pixel_height * (noise_map.height - 1) as f64;
                    let noise_val = noise_map.sample(nx, ny);
                    if noise_val < *noise_lower || noise_val > *noise_upper {
                        continue;
                    }

                    // Probability gate
                    if rng.gen::<f64>() > *probability {
                        continue;
                    }

                    lights.push(make_light(config, x, y, alloc));
                }
            }

            LightPlacement::AlongPath { path, density } => {
                if let Some(path_points) = features.get_path(path) {
                    let interval = if *density > 0.0 {
                        256.0 / density // grid squares between lights
                    } else {
                        256.0
                    };

                    let sampled = sample_along_path(path_points, interval);
                    for (x, y) in sampled {
                        lights.push(make_light(config, x, y, alloc));
                    }
                }
            }

            LightPlacement::WithObjects { group } => {
                if let Some(positions) = features.get_object_positions(group) {
                    for &(x, y) in positions {
                        lights.push(make_light(config, x, y, alloc));
                    }
                }
            }
        }
    }

    lights
}

/// Create a MapLight from config and position.
fn make_light(config: &LightConfig, x: f64, y: f64, alloc: &NodeIdAllocator) -> MapLight {
    MapLight {
        position: Vector2::new(x, y),
        color: config.color.clone(),
        range: config.range * 256.0, // grid squares to pixels
        intensity: config.intensity,
        shadows: config.shadows,
        layer: config.layer,
        node_id: alloc.next(),
    }
}

/// Sample points at regular intervals along a polyline.
fn sample_along_path(points: &[(f64, f64)], interval: f64) -> Vec<(f64, f64)> {
    if points.len() < 2 || interval <= 0.0 {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut accumulated = 0.0;

    for window in points.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        let dx = x1 - x0;
        let dy = y1 - y0;
        let seg_len = (dx * dx + dy * dy).sqrt();

        if seg_len < f64::EPSILON {
            continue;
        }

        while accumulated < seg_len {
            let t = accumulated / seg_len;
            result.push((x0 + dx * t, y0 + dy * t));
            accumulated += interval;
        }
        accumulated -= seg_len;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_along_path_basic() {
        let points = vec![(0.0, 0.0), (256.0, 0.0)]; // 1 grid square
        let sampled = sample_along_path(&points, 128.0); // every half grid sq
        assert_eq!(sampled.len(), 2);
        assert!((sampled[0].0 - 0.0).abs() < 1.0);
        assert!((sampled[1].0 - 128.0).abs() < 1.0);
    }

    #[test]
    fn test_sample_along_path_multi_segment() {
        let points = vec![(0.0, 0.0), (256.0, 0.0), (256.0, 256.0)];
        let sampled = sample_along_path(&points, 128.0);
        assert!(sampled.len() >= 3);
    }

    #[test]
    fn test_sample_along_path_empty() {
        let sampled = sample_along_path(&[], 128.0);
        assert!(sampled.is_empty());

        let sampled = sample_along_path(&[(0.0, 0.0)], 128.0);
        assert!(sampled.is_empty());
    }

    #[test]
    fn test_make_light() {
        let alloc = NodeIdAllocator::new(1);
        let config = LightConfig {
            placement: LightPlacement::Scatter {
                density: 2.0,
                noise_lower: 0.0,
                noise_upper: 1.0,
                probability: 1.0,
                margin: 0.0,
            },
            color: "ffeaefca".to_string(),
            intensity: 0.5,
            range: 5.0,
            shadows: false,
            layer: 100,
        };
        let light = make_light(&config, 100.0, 200.0, &alloc);
        assert_eq!(light.position.x, 100.0);
        assert_eq!(light.position.y, 200.0);
        assert_eq!(light.range, 5.0 * 256.0); // converted to pixels
        assert_eq!(light.intensity, 0.5);
        assert_eq!(light.color, "ffeaefca");
    }

    #[test]
    fn test_scatter_generates_lights() {
        use rand::SeedableRng;

        let noise = NoiseMap::generate(40, 40, &Default::default());
        let features = GeneratedFeatures::default();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let configs = vec![LightConfig {
            placement: LightPlacement::Scatter {
                density: 3.0,
                noise_lower: 0.0,
                noise_upper: 1.0,
                probability: 1.0,
                margin: 0.0,
            },
            color: "ffffffff".to_string(),
            intensity: 1.0,
            range: 5.0,
            shadows: false,
            layer: 100,
        }];

        let lights = generate_lights(
            &configs,
            &noise,
            &features,
            10.0 * 256.0,
            10.0 * 256.0,
            &alloc,
            &mut rng,
        );
        assert!(!lights.is_empty(), "Scatter should produce lights");
    }

    #[test]
    fn test_along_path_generates_lights() {
        use rand::SeedableRng;

        let noise = NoiseMap::generate(40, 40, &Default::default());
        let mut features = GeneratedFeatures::default();
        features.paths.insert(
            "test_road".to_string(),
            vec![(0.0, 128.0), (2560.0, 128.0)],
        );
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let configs = vec![LightConfig {
            placement: LightPlacement::AlongPath {
                path: "test_road".to_string(),
                density: 0.5,
            },
            color: "ffffffff".to_string(),
            intensity: 0.5,
            range: 3.0,
            shadows: false,
            layer: 100,
        }];

        let lights = generate_lights(
            &configs,
            &noise,
            &features,
            2560.0,
            256.0,
            &alloc,
            &mut rng,
        );
        assert!(!lights.is_empty(), "Along-path should produce lights");
        // 10 grid squares at 0.5 density = ~5 lights
        assert!(lights.len() >= 3 && lights.len() <= 15);
    }

    #[test]
    fn test_with_objects_generates_lights() {
        use rand::SeedableRng;

        let noise = NoiseMap::generate(40, 40, &Default::default());
        let mut features = GeneratedFeatures::default();
        features.object_positions.insert(
            "trees_0".to_string(),
            vec![(100.0, 100.0), (500.0, 500.0), (900.0, 900.0)],
        );
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let configs = vec![LightConfig {
            placement: LightPlacement::WithObjects {
                group: "trees_0".to_string(),
            },
            color: "ffeaefca".to_string(),
            intensity: 0.6,
            range: 1.5,
            shadows: false,
            layer: 300,
        }];

        let lights = generate_lights(
            &configs,
            &noise,
            &features,
            2560.0,
            2560.0,
            &alloc,
            &mut rng,
        );
        assert_eq!(lights.len(), 3, "Should produce one light per object");
    }

    #[test]
    fn test_missing_reference_produces_no_lights() {
        use rand::SeedableRng;

        let noise = NoiseMap::generate(40, 40, &Default::default());
        let features = GeneratedFeatures::default();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let configs = vec![
            LightConfig {
                placement: LightPlacement::AlongPath {
                    path: "nonexistent".to_string(),
                    density: 0.5,
                },
                color: "ffffffff".to_string(),
                intensity: 1.0,
                range: 5.0,
                shadows: false,
                layer: 100,
            },
            LightConfig {
                placement: LightPlacement::WithObjects {
                    group: "nonexistent".to_string(),
                },
                color: "ffffffff".to_string(),
                intensity: 1.0,
                range: 5.0,
                shadows: false,
                layer: 100,
            },
        ];

        let lights = generate_lights(
            &configs,
            &noise,
            &features,
            2560.0,
            2560.0,
            &alloc,
            &mut rng,
        );
        assert!(lights.is_empty(), "Missing references should produce no lights");
    }
}
