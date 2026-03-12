//! Water body generation.
//!
//! Extracts water polygons from noise thresholds using Marching Squares,
//! outputting DD-compatible WaterTree structures.

use serde::{Deserialize, Serialize};

use crate::contour::{find_contours, smooth_contours};
use crate::format::godot_types::{PoolVector2Array, Vector2};
use crate::format::world::{Water, WaterTree};
use crate::format::NodeIdAllocator;
use crate::noise_gen::NoiseMap;

/// Configuration for water body generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterConfig {
    /// Noise threshold — values above this become water.
    pub threshold: f64,
    /// Deep water color (ARGB hex).
    pub deep_color: String,
    /// Shallow water color (ARGB hex).
    pub shallow_color: String,
    /// Blend distance for water edges.
    pub blend_distance: f64,
    /// Minimum contour length (in points) to keep.
    pub min_contour_points: usize,
    /// Smoothing iterations for water edges.
    pub smooth_iterations: usize,
    /// Pixels per noise cell (for coordinate scaling).
    pub pixels_per_cell: f64,
    /// Whether to disable the water border effect.
    #[serde(default)]
    pub disable_border: bool,
}

impl Default for WaterConfig {
    fn default() -> Self {
        Self {
            threshold: 0.75,
            deep_color: "ff3aa19a".to_string(),
            shallow_color: "ff3ac3b2".to_string(),
            blend_distance: 40.0,
            min_contour_points: 6,
            smooth_iterations: 2,
            pixels_per_cell: 256.0 / 4.0, // 64px per noise cell at default resolution
            disable_border: false,
        }
    }
}

/// Generate water bodies from a noise map.
///
/// Extracts contour polygons at the threshold, converts to pixel coordinates,
/// and builds a DD WaterTree structure.
pub fn generate_water(
    noise_map: &NoiseMap,
    config: &WaterConfig,
    alloc: &NodeIdAllocator,
) -> Water {
    let contours = find_contours(noise_map, config.threshold);
    let smoothed = smooth_contours(contours, config.smooth_iterations);

    // Filter small contours and convert to pixel coordinates
    let polygons: Vec<Vec<(f64, f64)>> = smoothed
        .into_iter()
        .filter(|c| c.len() >= config.min_contour_points)
        .map(|c| {
            c.iter()
                .map(|&(x, y)| (x * config.pixels_per_cell, y * config.pixels_per_cell))
                .collect()
        })
        .collect();

    // Build root water tree with children for each polygon
    let children: Vec<WaterTree> = polygons
        .into_iter()
        .map(|poly| {
            let points: Vec<Vector2> = poly.iter().map(|&(x, y)| Vector2::new(x, y)).collect();
            WaterTree {
                node_ref: alloc.next().parse::<i64>().unwrap_or(0),
                polygon: PoolVector2Array::from_points(points),
                join: 0,
                end: 0,
                is_open: false,
                deep_color: config.deep_color.clone(),
                shallow_color: config.shallow_color.clone(),
                blend_distance: config.blend_distance,
                children: Vec::new(),
            }
        })
        .collect();

    Water {
        disable_border: config.disable_border,
        tree: Some(WaterTree {
            node_ref: alloc.next().parse::<i64>().unwrap_or(-1),
            polygon: PoolVector2Array::new(),
            join: 0,
            end: 0,
            is_open: false,
            deep_color: "00000000".to_string(),
            shallow_color: "00000000".to_string(),
            blend_distance: 0.0,
            children,
        }),
    }
}

/// Generate water from a river corridor polygon.
///
/// Takes the water polygon points from `RiverResult` and builds a WaterTree.
pub fn water_from_polygon(
    polygon: &[(f64, f64)],
    config: &WaterConfig,
    alloc: &NodeIdAllocator,
) -> WaterTree {
    let points: Vec<Vector2> = polygon
        .iter()
        .map(|&(x, y)| Vector2::new(x, y))
        .collect();

    WaterTree {
        node_ref: alloc.next().parse::<i64>().unwrap_or(0),
        polygon: PoolVector2Array::from_points(points),
        join: 0,
        end: 0,
        is_open: false,
        deep_color: config.deep_color.clone(),
        shallow_color: config.shallow_color.clone(),
        blend_distance: config.blend_distance,
        children: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noise_gen::{NoiseConfig, NoiseMap};

    #[test]
    fn test_generate_water_basic() {
        let mut noise = NoiseMap::generate(
            50,
            50,
            &NoiseConfig {
                seed: 42,
                ..Default::default()
            },
        );
        // Apply island mode so edges have high noise (water)
        noise.apply_island_mode(1.5);

        let alloc = NodeIdAllocator::new(1);
        let config = WaterConfig {
            threshold: 0.7,
            min_contour_points: 3,
            ..Default::default()
        };

        let water = generate_water(&noise, &config, &alloc);
        // Should have at least the root tree
        let tree = water.tree.as_ref().expect("Water should have a tree");
        assert_eq!(tree.deep_color, "00000000"); // Root is transparent
        // With island mode, should find water contours at edges
    }

    #[test]
    fn test_generate_water_uniform() {
        // Uniform noise below threshold → no water polygons
        let noise = NoiseMap {
            width: 20,
            height: 20,
            data: vec![vec![0.3; 20]; 20],
        };

        let alloc = NodeIdAllocator::new(1);
        let config = WaterConfig::default();

        let water = generate_water(&noise, &config, &alloc);
        let tree = water.tree.as_ref().expect("Water should have a tree");
        assert!(tree.children.is_empty());
    }

    #[test]
    fn test_water_from_polygon() {
        let alloc = NodeIdAllocator::new(1);
        let config = WaterConfig::default();

        let polygon = vec![
            (100.0, 100.0),
            (200.0, 100.0),
            (200.0, 200.0),
            (100.0, 200.0),
            (100.0, 100.0),
        ];

        let tree = water_from_polygon(&polygon, &config, &alloc);
        assert_eq!(tree.polygon.0.len(), 5);
        assert_eq!(tree.deep_color, "ff3aa19a");
        assert_eq!(tree.shallow_color, "ff3ac3b2");
    }

    #[test]
    fn test_water_colors() {
        let config = WaterConfig {
            deep_color: "ffff0000".to_string(),
            shallow_color: "ff00ff00".to_string(),
            ..Default::default()
        };

        let alloc = NodeIdAllocator::new(1);
        let polygon = vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 0.0)];
        let tree = water_from_polygon(&polygon, &config, &alloc);

        assert_eq!(tree.deep_color, "ffff0000");
        assert_eq!(tree.shallow_color, "ff00ff00");
    }
}
