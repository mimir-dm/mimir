//! Declarative lake generation.
//!
//! Lakes are placed features with center, radius, and roughness. The shoreline
//! is noise-perturbed for organic shapes. The noise map is depressed within the
//! lake boundary to create a natural basin.

use serde::{Deserialize, Serialize};

use crate::format::godot_types::{PoolVector2Array, Vector2};
use crate::format::world::WaterTree;
use crate::format::NodeIdAllocator;
use crate::noise_gen::NoiseMap;

/// Configuration for a declarative lake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LakeConfig {
    /// Unique identifier for cross-referencing.
    pub id: String,
    /// Center position in grid squares [x, y].
    pub center: [f64; 2],
    /// Approximate radius in grid squares.
    pub radius: f64,
    /// Shoreline roughness (0.0 = smooth oval, 1.0 = very jagged).
    #[serde(default = "default_roughness")]
    pub roughness: f64,
    /// Deep water color (ARGB hex).
    #[serde(default = "default_deep_color")]
    pub deep_color: String,
    /// Shallow water color (ARGB hex).
    #[serde(default = "default_shallow_color")]
    pub shallow_color: String,
    /// Blend distance for water edge.
    #[serde(default = "default_blend_distance")]
    pub blend_distance: f64,
}

fn default_roughness() -> f64 {
    0.4
}
fn default_deep_color() -> String {
    "ff2a7f6f".to_string()
}
fn default_shallow_color() -> String {
    "ff3ac3b2".to_string()
}
fn default_blend_distance() -> f64 {
    3.0
}

/// Result of lake generation.
pub struct LakeResult {
    /// Shoreline polygon in pixel coordinates (closed).
    pub shoreline: Vec<(f64, f64)>,
    /// Water tree node for the DD water section.
    pub water_tree: WaterTree,
}

/// Generate a lake with organic noise-perturbed shoreline.
///
/// Returns the shoreline polygon and a DD water tree node.
pub fn generate_lake(
    config: &LakeConfig,
    noise_map: &NoiseMap,
    _alloc: &NodeIdAllocator,
) -> LakeResult {
    let cx = config.center[0] * 256.0;
    let cy = config.center[1] * 256.0;
    let base_radius = config.radius * 256.0;

    // Generate shoreline by sampling noise at points around the circle
    let num_points = 120;
    let mut shoreline = Vec::with_capacity(num_points + 1);

    for i in 0..num_points {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (num_points as f64);
        let dx = angle.cos();
        let dy = angle.sin();

        // Sample noise at the point on the base circle
        let sample_x = cx + dx * base_radius;
        let sample_y = cy + dy * base_radius;

        // Convert pixel coords to noise grid coords
        // Noise grid has 4 cells per grid square, each grid square is 256px
        // So pixel_to_noise = pixel / (256.0 / 4.0) = pixel / 64.0
        let nx = (sample_x / 64.0).clamp(0.0, (noise_map.width - 1) as f64);
        let ny = (sample_y / 64.0).clamp(0.0, (noise_map.height - 1) as f64);
        let noise_val = noise_map.sample(nx, ny);

        // Perturb radius by noise, scaled by roughness
        // noise_val is 0.0-1.0, center it around 0.5 so perturbation goes both ways
        let perturbation = (noise_val - 0.5) * 2.0 * config.roughness * base_radius * 0.3;
        let r = (base_radius + perturbation).max(base_radius * 0.3);

        let px = cx + dx * r;
        let py = cy + dy * r;
        shoreline.push((px, py));
    }

    // Bezier smooth for organic curves
    let smoothed = crate::curves::bezier_smooth(&shoreline, 4);

    // Close the polygon
    let mut closed = smoothed.clone();
    if let Some(&first) = closed.first() {
        closed.push(first);
    }

    // Build water tree node
    let points: Vec<Vector2> = closed.iter()
        .map(|&(x, y)| Vector2::new(x, y))
        .collect();

    let water_tree = WaterTree {
        node_ref: rand_ref(),
        polygon: PoolVector2Array::from_points(points),
        join: 0,
        end: 0,
        is_open: false,
        deep_color: config.deep_color.clone(),
        shallow_color: config.shallow_color.clone(),
        blend_distance: config.blend_distance,
        children: Vec::new(),
    };

    LakeResult {
        shoreline: smoothed,
        water_tree,
    }
}

/// Depress the noise map within a lake shoreline polygon.
///
/// Sets noise values inside the polygon to a low floor value, with
/// blending at the edges for smooth terrain transitions.
pub fn depress_noise_for_lake(
    noise_map: &mut NoiseMap,
    shoreline: &[(f64, f64)],
    blend_distance_px: f64,
) {
    let floor_value = 0.05; // very low noise inside lake

    for y in 0..noise_map.height {
        for x in 0..noise_map.width {
            // Convert noise grid to pixel coords (noise is 4 cells per grid sq)
            let px = x as f64 * 256.0 / 4.0;
            let py = y as f64 * 256.0 / 4.0;

            if point_in_polygon(px, py, shoreline) {
                // Inside lake — find distance to nearest shoreline edge for blending
                let dist = distance_to_polygon_edge(px, py, shoreline);
                if dist > blend_distance_px {
                    // Deep inside — full depression
                    noise_map.data[y][x] = floor_value;
                } else {
                    // Near edge — blend between current value and floor
                    let t = dist / blend_distance_px;
                    let current = noise_map.data[y][x];
                    noise_map.data[y][x] = current * (1.0 - t) + floor_value * t;
                }
            }
        }
    }
}

/// Generate a random reference number for water tree nodes.
fn rand_ref() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(42);
    seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) & 0x7FFFFFFF
}

/// Check if a point is inside a lake polygon (public for object exclusion).
pub fn point_in_lake(x: f64, y: f64, polygon: &[(f64, f64)]) -> bool {
    point_in_polygon(x, y, polygon)
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

/// Distance from a point to the nearest edge of a polygon.
fn distance_to_polygon_edge(px: f64, py: f64, polygon: &[(f64, f64)]) -> f64 {
    let mut min_dist = f64::MAX;
    let n = polygon.len();
    for i in 0..n {
        let j = (i + 1) % n;
        let (x0, y0) = polygon[i];
        let (x1, y1) = polygon[j];
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len_sq = dx * dx + dy * dy;
        if len_sq < f64::EPSILON {
            continue;
        }
        let t = ((px - x0) * dx + (py - y0) * dy) / len_sq;
        let t = t.clamp(0.0, 1.0);
        let cx = x0 + t * dx;
        let cy = y0 + t * dy;
        let d = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
        min_dist = min_dist.min(d);
    }
    min_dist
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noise_gen::NoiseConfig;

    #[test]
    fn test_generate_lake_basic() {
        let noise = NoiseMap::generate(40, 40, &NoiseConfig { seed: 42, ..Default::default() });
        let alloc = NodeIdAllocator::new(1);

        let config = LakeConfig {
            id: "test_lake".to_string(),
            center: [5.0, 5.0],
            radius: 3.0,
            roughness: 0.4,
            deep_color: "ff2a7f6f".to_string(),
            shallow_color: "ff3ac3b2".to_string(),
            blend_distance: 3.0,
        };

        let result = generate_lake(&config, &noise, &alloc);
        assert!(result.shoreline.len() > 10, "Shoreline should have many points");
        assert_eq!(result.water_tree.deep_color, "ff2a7f6f");
    }

    #[test]
    fn test_shoreline_is_organic() {
        let noise = NoiseMap::generate(40, 40, &NoiseConfig { seed: 42, ..Default::default() });
        let alloc = NodeIdAllocator::new(1);

        let config = LakeConfig {
            id: "test".to_string(),
            center: [5.0, 5.0],
            radius: 3.0,
            roughness: 0.8,
            deep_color: default_deep_color(),
            shallow_color: default_shallow_color(),
            blend_distance: 3.0,
        };

        let result = generate_lake(&config, &noise, &alloc);
        // With roughness 0.8, distances from center should vary
        let cx = 5.0 * 256.0;
        let cy = 5.0 * 256.0;
        let distances: Vec<f64> = result.shoreline.iter()
            .map(|&(x, y)| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt())
            .collect();
        let min_d = distances.iter().cloned().fold(f64::MAX, f64::min);
        let max_d = distances.iter().cloned().fold(0.0_f64, f64::max);
        assert!(max_d - min_d > 50.0, "Organic shoreline should have varied distances, got range {}", max_d - min_d);
    }

    #[test]
    fn test_zero_roughness_is_smooth() {
        let noise = NoiseMap::generate(40, 40, &NoiseConfig { seed: 42, ..Default::default() });
        let alloc = NodeIdAllocator::new(1);

        let config = LakeConfig {
            id: "test".to_string(),
            center: [5.0, 5.0],
            radius: 3.0,
            roughness: 0.0,
            deep_color: default_deep_color(),
            shallow_color: default_shallow_color(),
            blend_distance: 3.0,
        };

        let result = generate_lake(&config, &noise, &alloc);
        let cx = 5.0 * 256.0;
        let cy = 5.0 * 256.0;
        let distances: Vec<f64> = result.shoreline.iter()
            .map(|&(x, y)| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt())
            .collect();
        let min_d = distances.iter().cloned().fold(f64::MAX, f64::min);
        let max_d = distances.iter().cloned().fold(0.0_f64, f64::max);
        // Zero roughness should still have some variation from Bezier smoothing but minimal
        assert!(max_d - min_d < 100.0, "Zero roughness should be nearly circular, got range {}", max_d - min_d);
    }

    #[test]
    fn test_depress_noise() {
        let mut noise = NoiseMap::generate(40, 40, &NoiseConfig { seed: 42, ..Default::default() });
        let original_center = noise.data[20][20];

        // Simple square "lake" polygon
        let shoreline = vec![
            (1024.0, 1024.0), (1536.0, 1024.0),
            (1536.0, 1536.0), (1024.0, 1536.0),
        ];

        depress_noise_for_lake(&mut noise, &shoreline, 64.0);

        // Center should be depressed
        assert!(noise.data[20][20] < original_center,
            "Noise at lake center should be depressed: {} vs original {}",
            noise.data[20][20], original_center);
    }

    #[test]
    fn test_point_in_polygon() {
        let square = vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
        assert!(point_in_polygon(50.0, 50.0, &square));
        assert!(!point_in_polygon(150.0, 50.0, &square));
    }
}
