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
/// For radial modes (island/lake), uses radial sampling to produce a single
/// clean closed polygon. For natural noise, falls back to marching squares
/// contour extraction.
pub fn generate_water(
    noise_map: &NoiseMap,
    config: &WaterConfig,
    alloc: &NodeIdAllocator,
) -> Water {
    generate_water_inner(noise_map, config, alloc, WaterMode::Contour)
}

/// Generate water with radial sampling mode for lake presets.
/// Casts rays from the center outward to find the threshold boundary,
/// producing one clean closed polygon.
pub fn generate_water_radial(
    noise_map: &NoiseMap,
    config: &WaterConfig,
    alloc: &NodeIdAllocator,
) -> Water {
    generate_water_inner(noise_map, config, alloc, WaterMode::Lake)
}

/// Generate water for island-mode maps (water ring around edges).
pub fn generate_water_island(
    noise_map: &NoiseMap,
    config: &WaterConfig,
    alloc: &NodeIdAllocator,
) -> Water {
    generate_water_inner(noise_map, config, alloc, WaterMode::Island)
}

#[derive(Clone, Copy)]
enum WaterMode {
    Contour,
    Lake,
    Island,
}

fn generate_water_inner(
    noise_map: &NoiseMap,
    config: &WaterConfig,
    alloc: &NodeIdAllocator,
    mode: WaterMode,
) -> Water {
    // Island mode uses a special tree structure: ocean rectangle parent with
    // island shoreline child (transparent colors) to punch a hole.
    if let WaterMode::Island = mode {
        return generate_island_water_tree(noise_map, config, alloc);
    }

    let polygons: Vec<Vec<(f64, f64)>> = match mode {
        WaterMode::Lake => radial_water_polygon(noise_map, config),
        WaterMode::Island => unreachable!(),
        WaterMode::Contour => {
            let contours = find_contours(noise_map, config.threshold);
            let smoothed = smooth_contours(contours, config.smooth_iterations);
            smoothed
                .into_iter()
                .filter(|c| c.len() >= config.min_contour_points)
                .map(|c| {
                    c.iter()
                        .map(|&(x, y)| (x * config.pixels_per_cell, y * config.pixels_per_cell))
                        .collect()
                })
                .collect()
        }
    };

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

/// Build island water tree: ocean rectangle parent → island shoreline child (transparent hole).
///
/// This matches the Dungeondraft convention: the parent polygon fills ocean with water colors,
/// and a child polygon with transparent colors ("00000000") cuts a hole for the island.
fn generate_island_water_tree(
    noise_map: &NoiseMap,
    config: &WaterConfig,
    alloc: &NodeIdAllocator,
) -> Water {
    let shoreline = island_shoreline(noise_map, config);
    let ppc = config.pixels_per_cell;
    let w = noise_map.width as f64 * ppc;
    let h = noise_map.height as f64 * ppc;

    // Ocean rectangle — extends 1 grid square beyond map borders (matching reference impl)
    let margin = 256.0; // 1 grid square = 256px
    let ocean_points = vec![
        Vector2::new(-margin, -margin),
        Vector2::new(w + margin, -margin),
        Vector2::new(w + margin, h + margin),
        Vector2::new(-margin, h + margin),
    ];

    // Island shoreline — child with transparent colors to punch a hole
    let shore_points: Vec<Vector2> = shoreline
        .iter()
        .map(|&(x, y)| Vector2::new(x, y))
        .collect();

    let island_hole = WaterTree {
        node_ref: alloc.next().parse::<i64>().unwrap_or(0),
        polygon: PoolVector2Array::from_points(shore_points),
        join: 0,
        end: 0,
        is_open: false,
        deep_color: "00000000".to_string(),
        shallow_color: "00000000".to_string(),
        blend_distance: 0.0,
        children: Vec::new(),
    };

    let ocean = WaterTree {
        node_ref: alloc.next().parse::<i64>().unwrap_or(0),
        polygon: PoolVector2Array::from_points(ocean_points),
        join: 0,
        end: 0,
        is_open: false,
        deep_color: config.deep_color.clone(),
        shallow_color: config.shallow_color.clone(),
        blend_distance: config.blend_distance,
        children: vec![island_hole],
    };

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
            children: vec![ocean],
        }),
    }
}

/// Generate a single closed water polygon by radial sampling from the map center.
///
/// For each angle (0..360), casts a ray outward from the center and finds where
/// the noise value drops below the threshold. This produces one clean closed polygon
/// suitable for lake-mode maps where water is in the center.
fn radial_water_polygon(noise_map: &NoiseMap, config: &WaterConfig) -> Vec<Vec<(f64, f64)>> {
    let cx = noise_map.width as f64 / 2.0;
    let cy = noise_map.height as f64 / 2.0;
    let max_radius = cx.max(cy);
    let num_rays = 120; // one point every 3 degrees
    let step = 0.5_f64; // sample every half cell along each ray

    let mut points: Vec<(f64, f64)> = Vec::new();

    for i in 0..num_rays {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (num_rays as f64);
        let dx = angle.cos();
        let dy = angle.sin();

        // Walk outward from center until noise drops below threshold or we hit the edge
        let mut boundary_r = max_radius; // default to edge if never drops
        let mut r = 0.0;
        while r < max_radius {
            let sx = cx + dx * r;
            let sy = cy + dy * r;

            // Bounds check
            if sx < 0.0 || sy < 0.0 || sx >= noise_map.width as f64 || sy >= noise_map.height as f64
            {
                boundary_r = r;
                break;
            }

            let val = noise_map.sample(sx, sy);
            if val < config.threshold {
                boundary_r = r;
                break;
            }
            r += step;
        }

        let px = (cx + dx * boundary_r) * config.pixels_per_cell;
        let py = (cy + dy * boundary_r) * config.pixels_per_cell;
        points.push((px, py));
    }

    // Close the polygon
    if let Some(&first) = points.first() {
        points.push(first);
    }

    if points.len() >= config.min_contour_points {
        vec![points]
    } else {
        vec![]
    }
}

/// Trace the island shoreline via radial sampling.
///
/// Casts rays from the map center outward, finding where noise exceeds the
/// water threshold. Returns a closed polygon of shoreline points in pixel coordinates.
fn island_shoreline(noise_map: &NoiseMap, config: &WaterConfig) -> Vec<(f64, f64)> {
    let ncx = noise_map.width as f64 / 2.0;
    let ncy = noise_map.height as f64 / 2.0;
    let max_radius = (ncx * ncx + ncy * ncy).sqrt();
    let num_rays: usize = 120;
    let step = 0.5_f64;
    let ppc = config.pixels_per_cell;

    let mut radii: Vec<f64> = Vec::with_capacity(num_rays);

    for i in 0..num_rays {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (num_rays as f64);
        let dx = angle.cos();
        let dy = angle.sin();

        // Walk outward from center. Two phases:
        // 1. "seeking land" — skip any initial above-threshold noise at center
        // 2. "on land" — once we find land (below threshold), look for the shore
        let mut boundary_r = max_radius;
        let mut found_land = false;
        let mut r = 0.0;
        while r < max_radius {
            let sx = ncx + dx * r;
            let sy = ncy + dy * r;
            if sx < 0.0
                || sy < 0.0
                || sx >= noise_map.width as f64
                || sy >= noise_map.height as f64
            {
                boundary_r = r;
                break;
            }
            let val = noise_map.sample(sx, sy);
            if !found_land {
                if val < config.threshold {
                    found_land = true;
                }
            } else if val >= config.threshold {
                boundary_r = r;
                break;
            }
            r += step;
        }

        radii.push(boundary_r);
    }

    // Clamp radii: no ray should deviate more than 40% from the mean radius.
    // This prevents deep concavities from noise valleys cutting into the island.
    let mean_r: f64 = radii.iter().sum::<f64>() / radii.len() as f64;
    let min_r = mean_r * 0.6;
    let max_r = mean_r * 1.4;
    for r in &mut radii {
        *r = r.clamp(min_r, max_r);
    }

    // Heavy smoothing in radius space (circular averaging)
    let smooth_passes = config.smooth_iterations.max(3) * 3; // triple the configured smoothing
    for _ in 0..smooth_passes {
        let prev = radii.clone();
        for i in 0..num_rays {
            let p = if i == 0 { num_rays - 1 } else { i - 1 };
            let n = (i + 1) % num_rays;
            radii[i] = (prev[p] + prev[i] + prev[n]) / 3.0;
        }
    }

    // Convert smoothed radii to pixel coordinates
    let mut shore: Vec<(f64, f64)> = Vec::with_capacity(num_rays + 1);
    for i in 0..num_rays {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (num_rays as f64);
        let dx = angle.cos();
        let dy = angle.sin();
        shore.push(((ncx + dx * radii[i]) * ppc, (ncy + dy * radii[i]) * ppc));
    }

    // Close the polygon
    if let Some(&first) = shore.first() {
        shore.push(first);
    }

    shore
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
