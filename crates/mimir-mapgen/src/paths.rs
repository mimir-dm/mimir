//! Road and river generation.
//!
//! Greedy pathfinding along noise ridges/valleys with Bezier smoothing.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

use crate::curves::{bezier_smooth, offset_polyline};
use crate::format::entities::MapPath;
use crate::format::godot_types::Vector2;
use crate::format::NodeIdAllocator;
use crate::noise_gen::NoiseMap;

/// Which map edge a road/river starts or ends at.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

/// Configuration for road generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadConfig {
    /// Starting edge.
    pub from: Edge,
    /// Ending edge.
    pub to: Edge,
    /// Road texture path.
    pub texture: String,
    /// Road width in pixels.
    pub width: f64,
    /// Layer in the DD map.
    pub layer: i32,
    /// Step distance for greedy walk (pixels).
    pub step_distance: f64,
    /// Field of view angle (radians) for candidate evaluation.
    pub fov: f64,
    /// Weight for noise vs progress (0.0 = all progress, 1.0 = all noise).
    pub noise_weight: f64,
    /// Margin from map edge in pixels.
    pub margin: f64,
    /// Bezier smoothing density (points per segment).
    pub smooth_density: usize,
    /// Optional edge paths (border textures along road sides).
    pub edge_paths: Option<EdgePathConfig>,
}

/// Configuration for edge/border paths along roads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgePathConfig {
    /// Texture for edge paths.
    pub texture: String,
    /// Offset distance from road center.
    pub offset: f64,
    /// Width of edge paths.
    pub width: f64,
    /// Layer for edge paths.
    pub layer: i32,
}

/// Configuration for river generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiverConfig {
    /// Starting edge.
    pub from: Edge,
    /// Ending edge.
    pub to: Edge,
    /// River width in pixels.
    pub width: f64,
    /// Step distance for greedy walk.
    pub step_distance: f64,
    /// FOV angle for candidates.
    pub fov: f64,
    /// Noise weight (rivers follow valleys, so low noise).
    pub noise_weight: f64,
    /// Margin from map edge.
    pub margin: f64,
    /// Bezier smoothing density.
    pub smooth_density: usize,
    /// Deep water color (ARGB hex).
    pub deep_color: String,
    /// Shallow water color (ARGB hex).
    pub shallow_color: String,
    /// Bank path texture.
    pub bank_texture: String,
    /// Bank path width.
    pub bank_width: f64,
    /// Bank layer.
    pub bank_layer: i32,
}

impl Default for RoadConfig {
    fn default() -> Self {
        Self {
            from: Edge::Left,
            to: Edge::Right,
            texture: "res://textures/paths/path_dirt.png".to_string(),
            width: 80.0,
            layer: 100,
            step_distance: 64.0,
            fov: PI / 3.0,
            noise_weight: 0.5,
            margin: 128.0,
            smooth_density: 8,
            edge_paths: None,
        }
    }
}

impl Default for RiverConfig {
    fn default() -> Self {
        Self {
            from: Edge::Top,
            to: Edge::Bottom,
            width: 120.0,
            step_distance: 64.0,
            fov: PI / 3.0,
            noise_weight: 0.5,
            margin: 128.0,
            smooth_density: 8,
            deep_color: "ff3aa19a".to_string(),
            shallow_color: "ff3ac3b2".to_string(),
            bank_texture: "res://textures/paths/path_rocks.png".to_string(),
            bank_width: 20.0,
            bank_layer: 100,
        }
    }
}

/// Result of road generation.
pub struct RoadResult {
    /// The main road path.
    pub road: MapPath,
    /// Optional edge/border paths.
    pub edge_paths: Vec<MapPath>,
    /// Road center waypoints (for corridor clearing / terrain modification).
    pub corridor_points: Vec<(f64, f64)>,
    /// Half-width of the corridor.
    pub corridor_half_width: f64,
}

/// Result of river generation.
pub struct RiverResult {
    /// River bank paths (left and right).
    pub bank_paths: Vec<MapPath>,
    /// River center waypoints (for water polygon generation).
    pub corridor_points: Vec<(f64, f64)>,
    /// Half-width of the river corridor.
    pub corridor_half_width: f64,
    /// Water polygon points for constructing a WaterTree.
    pub water_polygon: Vec<(f64, f64)>,
}

/// Generate a road across the map using greedy pathfinding.
pub fn generate_road(
    noise_map: &NoiseMap,
    config: &RoadConfig,
    pixel_width: f64,
    pixel_height: f64,
    alloc: &NodeIdAllocator,
    rng: &mut impl Rng,
) -> Option<RoadResult> {
    let start = random_edge_point(config.from, pixel_width, pixel_height, config.margin, rng);
    let target = random_edge_point(config.to, pixel_width, pixel_height, config.margin, rng);

    let waypoints = greedy_walk(
        noise_map,
        start,
        target,
        pixel_width,
        pixel_height,
        config.step_distance,
        config.fov,
        config.noise_weight,
        true, // Roads follow ridges (high noise)
        rng,
    );

    if waypoints.len() < 2 {
        return None;
    }

    let smoothed = bezier_smooth(&waypoints, config.smooth_density);

    let road_vectors: Vec<Vector2> = smoothed.iter().map(|&(x, y)| Vector2::new(x, y)).collect();
    let road = MapPath::new(&config.texture, road_vectors, config.width, &alloc.next())
        .with_layer(config.layer);

    let mut edge_paths = Vec::new();
    if let Some(ref ep) = config.edge_paths {
        let left = offset_polyline(&smoothed, ep.offset);
        let right = offset_polyline(&smoothed, -ep.offset);

        let left_vectors: Vec<Vector2> = left.iter().map(|&(x, y)| Vector2::new(x, y)).collect();
        let right_vectors: Vec<Vector2> =
            right.iter().map(|&(x, y)| Vector2::new(x, y)).collect();

        edge_paths.push(
            MapPath::new(&ep.texture, left_vectors, ep.width, &alloc.next())
                .with_layer(ep.layer),
        );
        edge_paths.push(
            MapPath::new(&ep.texture, right_vectors, ep.width, &alloc.next())
                .with_layer(ep.layer),
        );
    }

    Some(RoadResult {
        road,
        edge_paths,
        corridor_points: smoothed,
        corridor_half_width: config.width / 2.0,
    })
}

/// Generate a river across the map.
pub fn generate_river(
    noise_map: &NoiseMap,
    config: &RiverConfig,
    pixel_width: f64,
    pixel_height: f64,
    alloc: &NodeIdAllocator,
    rng: &mut impl Rng,
) -> Option<RiverResult> {
    let start = random_edge_point(config.from, pixel_width, pixel_height, config.margin, rng);
    let target = random_edge_point(config.to, pixel_width, pixel_height, config.margin, rng);

    let waypoints = greedy_walk(
        noise_map,
        start,
        target,
        pixel_width,
        pixel_height,
        config.step_distance,
        config.fov,
        config.noise_weight,
        false, // Rivers follow valleys (low noise)
        rng,
    );

    if waypoints.len() < 2 {
        return None;
    }

    let smoothed = bezier_smooth(&waypoints, config.smooth_density);

    // Generate bank paths on both sides
    let half_w = config.width / 2.0;
    let left_bank = offset_polyline(&smoothed, half_w);
    let right_bank = offset_polyline(&smoothed, -half_w);

    let left_vectors: Vec<Vector2> =
        left_bank.iter().map(|&(x, y)| Vector2::new(x, y)).collect();
    let right_vectors: Vec<Vector2> = right_bank
        .iter()
        .map(|&(x, y)| Vector2::new(x, y))
        .collect();

    let bank_paths = vec![
        MapPath::new(
            &config.bank_texture,
            left_vectors,
            config.bank_width,
            &alloc.next(),
        )
        .with_layer(config.bank_layer),
        MapPath::new(
            &config.bank_texture,
            right_vectors,
            config.bank_width,
            &alloc.next(),
        )
        .with_layer(config.bank_layer),
    ];

    // Build water polygon from the corridor outline
    let mut water_polygon = left_bank.clone();
    let mut right_rev = right_bank;
    right_rev.reverse();
    water_polygon.extend(right_rev);
    // Close the polygon
    if let Some(&first) = water_polygon.first() {
        water_polygon.push(first);
    }

    Some(RiverResult {
        bank_paths,
        corridor_points: smoothed,
        corridor_half_width: half_w,
        water_polygon,
    })
}

/// Greedy pathfinding walk from start toward target.
///
/// At each step, evaluates candidate positions within an FOV cone,
/// scores them by a weighted combination of noise value and progress
/// toward target, and picks the best.
fn greedy_walk(
    noise_map: &NoiseMap,
    start: (f64, f64),
    target: (f64, f64),
    pixel_width: f64,
    pixel_height: f64,
    step_distance: f64,
    fov: f64,
    noise_weight: f64,
    prefer_high_noise: bool,
    rng: &mut impl Rng,
) -> Vec<(f64, f64)> {
    let mut path = vec![start];
    let mut current = start;
    let max_steps = ((pixel_width + pixel_height) / step_distance * 3.0) as usize;

    let noise_scale_x = noise_map.width as f64 / pixel_width;
    let noise_scale_y = noise_map.height as f64 / pixel_height;

    for _ in 0..max_steps {
        let dx = target.0 - current.0;
        let dy = target.1 - current.1;
        let dist_to_target = (dx * dx + dy * dy).sqrt();

        if dist_to_target < step_distance * 1.5 {
            path.push(target);
            break;
        }

        let base_angle = dy.atan2(dx);
        let num_candidates = 7;
        let mut best_score = f64::NEG_INFINITY;
        let mut best_pos = current;

        for i in 0..num_candidates {
            let angle_offset = (i as f64 / (num_candidates - 1) as f64 - 0.5) * fov;
            let angle = base_angle + angle_offset;

            // Add slight randomness
            let jitter = rng.gen_range(-0.05..0.05);
            let angle = angle + jitter;

            let nx = current.0 + angle.cos() * step_distance;
            let ny = current.1 + angle.sin() * step_distance;

            // Bounds check
            if nx < 0.0 || nx >= pixel_width || ny < 0.0 || ny >= pixel_height {
                continue;
            }

            let noise_val = noise_map.sample(nx * noise_scale_x, ny * noise_scale_y);
            let noise_score = if prefer_high_noise {
                noise_val
            } else {
                1.0 - noise_val
            };

            let new_dx = target.0 - nx;
            let new_dy = target.1 - ny;
            let new_dist = (new_dx * new_dx + new_dy * new_dy).sqrt();
            let progress = (dist_to_target - new_dist) / step_distance;

            let score = noise_score * noise_weight + progress * (1.0 - noise_weight);

            if score > best_score {
                best_score = score;
                best_pos = (nx, ny);
            }
        }

        if best_pos == current {
            break; // Stuck
        }

        current = best_pos;
        path.push(current);
    }

    path
}

/// Pick a random point along a map edge.
fn random_edge_point(
    edge: Edge,
    width: f64,
    height: f64,
    margin: f64,
    rng: &mut impl Rng,
) -> (f64, f64) {
    match edge {
        Edge::Left => (0.0, rng.gen_range(margin..(height - margin))),
        Edge::Right => (width, rng.gen_range(margin..(height - margin))),
        Edge::Top => (rng.gen_range(margin..(width - margin)), 0.0),
        Edge::Bottom => (rng.gen_range(margin..(width - margin)), height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noise_gen::{NoiseConfig, NoiseMap};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn test_noise() -> NoiseMap {
        NoiseMap::generate(
            100,
            100,
            &NoiseConfig {
                seed: 42,
                ..Default::default()
            },
        )
    }

    #[test]
    fn test_greedy_walk_reaches_target() {
        let noise = test_noise();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let path = greedy_walk(
            &noise,
            (0.0, 1280.0),
            (2560.0, 1280.0),
            2560.0,
            2560.0,
            64.0,
            PI / 3.0,
            0.3,
            true,
            &mut rng,
        );

        assert!(path.len() >= 2);
        // Should reach roughly the target edge
        let last = path.last().unwrap();
        assert!(
            last.0 > 2400.0,
            "Should reach near right edge, got x={}",
            last.0
        );
    }

    #[test]
    fn test_generate_road() {
        let noise = test_noise();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let config = RoadConfig {
            from: Edge::Left,
            to: Edge::Right,
            ..Default::default()
        };

        let result =
            generate_road(&noise, &config, 2560.0, 2560.0, &alloc, &mut rng);
        assert!(result.is_some());

        let road = result.unwrap();
        assert!(road.corridor_points.len() >= 2);
        assert!(road.road.points.0.len() >= 2);
        assert_eq!(road.edge_paths.len(), 0); // No edge config
    }

    #[test]
    fn test_generate_road_with_edges() {
        let noise = test_noise();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let config = RoadConfig {
            from: Edge::Left,
            to: Edge::Right,
            edge_paths: Some(EdgePathConfig {
                texture: "res://textures/paths/cliff.png".to_string(),
                offset: 50.0,
                width: 10.0,
                layer: 100,
            }),
            ..Default::default()
        };

        let result =
            generate_road(&noise, &config, 2560.0, 2560.0, &alloc, &mut rng);
        let road = result.unwrap();
        assert_eq!(road.edge_paths.len(), 2); // Left + right edge
    }

    #[test]
    fn test_generate_river() {
        let noise = test_noise();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let config = RiverConfig::default();

        let result =
            generate_river(&noise, &config, 2560.0, 2560.0, &alloc, &mut rng);
        assert!(result.is_some());

        let river = result.unwrap();
        assert_eq!(river.bank_paths.len(), 2);
        assert!(river.water_polygon.len() >= 4);
        assert!(river.corridor_points.len() >= 2);
    }

    #[test]
    fn test_road_deterministic() {
        let noise = test_noise();

        let alloc1 = NodeIdAllocator::new(1);
        let alloc2 = NodeIdAllocator::new(1);
        let mut rng1 = ChaCha8Rng::seed_from_u64(99);
        let mut rng2 = ChaCha8Rng::seed_from_u64(99);
        let config = RoadConfig::default();

        let r1 = generate_road(&noise, &config, 2560.0, 2560.0, &alloc1, &mut rng1).unwrap();
        let r2 = generate_road(&noise, &config, 2560.0, 2560.0, &alloc2, &mut rng2).unwrap();

        assert_eq!(r1.corridor_points.len(), r2.corridor_points.len());
        for (a, b) in r1.corridor_points.iter().zip(r2.corridor_points.iter()) {
            assert_eq!(a.0, b.0);
            assert_eq!(a.1, b.1);
        }
    }

    #[test]
    fn test_random_edge_point() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let p = random_edge_point(Edge::Left, 100.0, 100.0, 10.0, &mut rng);
        assert_eq!(p.0, 0.0);
        assert!(p.1 >= 10.0 && p.1 <= 90.0);

        let p = random_edge_point(Edge::Right, 100.0, 100.0, 10.0, &mut rng);
        assert_eq!(p.0, 100.0);

        let p = random_edge_point(Edge::Top, 100.0, 100.0, 10.0, &mut rng);
        assert_eq!(p.1, 0.0);

        let p = random_edge_point(Edge::Bottom, 100.0, 100.0, 10.0, &mut rng);
        assert_eq!(p.1, 100.0);
    }
}
