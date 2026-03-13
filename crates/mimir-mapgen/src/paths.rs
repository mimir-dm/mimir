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
use crate::rooms::ExclusionZone;

/// Which map edge a road/river starts or ends at.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

/// Path style: how the centerline is generated.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PathStyle {
    /// Greedy walk following noise ridges/valleys — roughly straight with gentle bends.
    Straight,
    /// Sinusoidal meander — wandering S-curves like a natural stream.
    Meandering,
}

impl Default for PathStyle {
    fn default() -> Self {
        PathStyle::Straight
    }
}

/// Configuration for road generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
    /// Path style: "straight" (greedy walk) or "meandering" (sine wave).
    pub style: PathStyle,
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
#[serde(default)]
pub struct RiverConfig {
    /// Starting edge.
    pub from: Edge,
    /// Ending edge.
    pub to: Edge,
    /// River width in pixels.
    pub width: f64,
    /// Path style: "straight" (greedy walk) or "meandering" (sine wave).
    pub style: PathStyle,
    /// Step distance for walk.
    pub step_distance: f64,
    /// FOV angle for candidates (straight style only).
    pub fov: f64,
    /// Noise weight.
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
            width: 512.0,
            layer: 100,
            style: PathStyle::Straight,
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
            style: PathStyle::Meandering,
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
    generate_road_with_exclusions(noise_map, config, pixel_width, pixel_height, alloc, rng, &[])
}

/// Generate a road, avoiding exclusion zones (rooms).
pub fn generate_road_with_exclusions(
    noise_map: &NoiseMap,
    config: &RoadConfig,
    pixel_width: f64,
    pixel_height: f64,
    alloc: &NodeIdAllocator,
    rng: &mut impl Rng,
    exclusion_zones: &[ExclusionZone],
) -> Option<RoadResult> {
    let start = random_edge_point(config.from, pixel_width, pixel_height, config.margin, rng);
    let target = random_edge_point(config.to, pixel_width, pixel_height, config.margin, rng);

    let raw_waypoints = match config.style {
        PathStyle::Straight => greedy_walk(
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
            exclusion_zones,
        ),
        PathStyle::Meandering => generate_meander(
            start,
            target,
            pixel_width,
            pixel_height,
            noise_map,
            config.width,
            config.step_distance,
            rng,
            exclusion_zones,
        ),
    };

    if raw_waypoints.len() < 2 {
        return None;
    }

    // Smooth then clip to map bounds (meander extends beyond edges).
    let smoothed_raw = bezier_smooth(&raw_waypoints, config.smooth_density);
    let smoothed = clip_polyline_to_rect(&smoothed_raw, 0.0, 0.0, pixel_width, pixel_height);

    if smoothed.len() < 2 {
        return None;
    }

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
    generate_river_with_exclusions(noise_map, config, pixel_width, pixel_height, alloc, rng, &[])
}

/// Generate a river, avoiding exclusion zones (rooms).
pub fn generate_river_with_exclusions(
    noise_map: &NoiseMap,
    config: &RiverConfig,
    pixel_width: f64,
    pixel_height: f64,
    alloc: &NodeIdAllocator,
    rng: &mut impl Rng,
    exclusion_zones: &[ExclusionZone],
) -> Option<RiverResult> {
    let start = random_edge_point(config.from, pixel_width, pixel_height, config.margin, rng);
    let target = random_edge_point(config.to, pixel_width, pixel_height, config.margin, rng);

    let raw_waypoints = match config.style {
        PathStyle::Meandering => generate_meander(
            start, target, pixel_width, pixel_height,
            noise_map, config.width, config.step_distance,
            rng, exclusion_zones,
        ),
        PathStyle::Straight => greedy_walk(
            noise_map, start, target, pixel_width, pixel_height,
            config.step_distance, config.fov, config.noise_weight,
            false, rng, exclusion_zones,
        ),
    };

    if raw_waypoints.len() < 2 {
        return None;
    }

    let half_w = config.width / 2.0;

    // Smooth the full (extended) path, then offset to get banks.
    // Clip bank polylines to map rect for clean edge rendering.
    let smoothed_full = bezier_smooth(&raw_waypoints, config.smooth_density);
    let left_bank_full = offset_polyline(&smoothed_full, half_w);
    let right_bank_full = offset_polyline(&smoothed_full, -half_w);

    let left_bank = clip_polyline_to_rect(&left_bank_full, 0.0, 0.0, pixel_width, pixel_height);
    let right_bank = clip_polyline_to_rect(&right_bank_full, 0.0, 0.0, pixel_width, pixel_height);

    let left_vectors: Vec<Vector2> =
        left_bank.iter().map(|&(x, y)| Vector2::new(x, y)).collect();
    let right_vectors: Vec<Vector2> =
        right_bank.iter().map(|&(x, y)| Vector2::new(x, y)).collect();

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

    // Build water polygon from a decimated centerline to avoid self-intersecting
    // inner curves at S-bends. Target ~20 points per side.
    let water_center = {
        let target_pts = 20;
        if raw_waypoints.len() <= target_pts {
            raw_waypoints.clone()
        } else {
            let step = raw_waypoints.len() / target_pts;
            let mut pts: Vec<(f64, f64)> = raw_waypoints.iter().step_by(step).copied().collect();
            if let Some(&last) = raw_waypoints.last() {
                if pts.last() != Some(&last) {
                    pts.push(last);
                }
            }
            pts
        }
    };
    let water_left = offset_polyline(&water_center, half_w);
    let water_right = offset_polyline(&water_center, -half_w);

    // Assemble water polygon: left bank forward, right bank reversed.
    // No closing vertex — clip_polygon_to_rect uses modular indexing.
    let mut raw_polygon: Vec<(f64, f64)> = water_left.clone();
    raw_polygon.extend(water_right.into_iter().rev());

    // Clip to map rectangle — Sutherland-Hodgman gives clean rectangular cuts.
    let mut water_polygon = clip_polygon_to_rect(&raw_polygon, 0.0, 0.0, pixel_width, pixel_height);
    // Close the polygon for DD.
    if let Some(&first) = water_polygon.first() {
        water_polygon.push(first);
    }

    // Clip the centerline for corridor exclusion data.
    let smoothed = clip_polyline_to_rect(&smoothed_full, 0.0, 0.0, pixel_width, pixel_height);

    Some(RiverResult {
        bank_paths,
        corridor_points: smoothed,
        corridor_half_width: half_w,
        water_polygon,
    })
}

/// Generate a meandering centerline using sinusoidal displacement + noise.
///
/// The path is generated **longer than the map** (extended beyond both edges),
/// then callers clip the result to the map rectangle. This means the sine wave
/// is at full amplitude when it crosses the map boundary, producing clean
/// entry/exit with no edge artifacts.
fn generate_meander(
    start: (f64, f64),
    target: (f64, f64),
    pixel_width: f64,
    pixel_height: f64,
    noise_map: &NoiseMap,
    path_width: f64,
    step_distance: f64,
    rng: &mut impl Rng,
    exclusion_zones: &[ExclusionZone],
) -> Vec<(f64, f64)> {
    let dx = target.0 - start.0;
    let dy = target.1 - start.1;
    let total_dist = (dx * dx + dy * dy).sqrt();
    if total_dist < 1.0 {
        return vec![start, target];
    }

    // Unit vectors: forward along start→target, and perpendicular
    let fwd = (dx / total_dist, dy / total_dist);
    let perp = (-fwd.1, fwd.0);

    // Meander amplitude: scale with path width so curves are never tighter
    // than the path itself. Cap at ~6 grid squares max displacement.
    let amplitude = (path_width * 3.0).min(pixel_width.min(pixel_height) * 0.08);

    // Extend path beyond map edges so the sine is at full amplitude at boundaries.
    // One wavelength extension on each side ensures smooth entry/exit after clipping.
    let extension = total_dist * 0.4;
    let ext_start = (start.0 - fwd.0 * extension, start.1 - fwd.1 * extension);
    let ext_total = total_dist + extension * 2.0;

    // Single random phase offset
    let phase: f64 = rng.gen_range(0.0..std::f64::consts::TAU);

    let noise_scale_x = noise_map.width as f64 / pixel_width;
    let noise_scale_y = noise_map.height as f64 / pixel_height;

    let num_steps = (ext_total / step_distance).ceil() as usize;
    let num_steps = num_steps.max(20);
    let mut points = Vec::with_capacity(num_steps + 1);

    for i in 0..=num_steps {
        let t = i as f64 / num_steps as f64;

        // Base position along the extended straight line
        let base_x = ext_start.0 + fwd.0 * ext_total * t;
        let base_y = ext_start.1 + fwd.1 * ext_total * t;

        // Single gentle sine wave (1.5 periods over original distance) — no fade
        let sine_offset = amplitude *
            (2.0 * std::f64::consts::PI * t * 1.5 + phase).sin();

        // Subtle noise perturbation for irregularity (clamp sample coords to map)
        let sample_x = base_x.clamp(0.0, pixel_width) * noise_scale_x;
        let sample_y = base_y.clamp(0.0, pixel_height) * noise_scale_y;
        let noise_val = noise_map.sample(sample_x, sample_y);
        let noise_offset = amplitude * 0.15 * (noise_val - 0.5);

        let total_offset = sine_offset + noise_offset;
        let px = base_x + perp.0 * total_offset;
        let py = base_y + perp.1 * total_offset;

        points.push((px, py));
    }

    // Shift the entire path perpendicular until no points intersect exclusion zones.
    // This preserves curve shape instead of distorting individual points.
    if !exclusion_zones.is_empty() {
        let pad = path_width * 0.5 + 64.0;
        for _attempt in 0..20 {
            let mut best_shift: Option<f64> = None;
            for &(px, py) in &points {
                for zone in exclusion_zones {
                    if zone.contains(px, py) {
                        let pt_perp = (px - start.0) * perp.0 + (py - start.1) * perp.1;
                        let zone_center_perp =
                            ((zone.x + zone.width / 2.0) - start.0) * perp.0
                            + ((zone.y + zone.height / 2.0) - start.1) * perp.1;

                        let corners = [
                            (zone.x, zone.y),
                            (zone.x + zone.width, zone.y),
                            (zone.x, zone.y + zone.height),
                            (zone.x + zone.width, zone.y + zone.height),
                        ];
                        let projections: Vec<f64> = corners.iter()
                            .map(|&(cx, cy)| (cx - start.0) * perp.0 + (cy - start.1) * perp.1)
                            .collect();
                        let zone_min = projections.iter().cloned().fold(f64::INFINITY, f64::min);
                        let zone_max = projections.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

                        let shift = if pt_perp >= zone_center_perp {
                            (zone_max - pt_perp) + pad
                        } else {
                            (zone_min - pt_perp) - pad
                        };

                        match best_shift {
                            None => best_shift = Some(shift),
                            Some(prev) => {
                                if shift.abs() > prev.abs() {
                                    best_shift = Some(shift);
                                }
                            }
                        }
                    }
                }
            }

            let Some(shift) = best_shift else { break };
            if shift.abs() < 1.0 { break; }

            let dx = perp.0 * shift;
            let dy = perp.1 * shift;
            for pt in &mut points {
                pt.0 += dx;
                pt.1 += dy;
            }
        }
    }

    points
}

// ── Clipping ─────────────────────────────────────────────────────────────────

/// Clip a polyline to a rectangle, keeping only the interior portion.
/// For paths that enter and exit the rectangle once.
fn clip_polyline_to_rect(
    points: &[(f64, f64)],
    x_min: f64, y_min: f64,
    x_max: f64, y_max: f64,
) -> Vec<(f64, f64)> {
    fn inside(p: (f64, f64), x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> bool {
        p.0 >= x_min && p.0 <= x_max && p.1 >= y_min && p.1 <= y_max
    }

    let mut result = Vec::new();

    for window in points.windows(2) {
        let (a, b) = (window[0], window[1]);
        let a_in = inside(a, x_min, y_min, x_max, y_max);
        let b_in = inside(b, x_min, y_min, x_max, y_max);

        match (a_in, b_in) {
            (true, true) => {
                if result.is_empty() { result.push(a); }
                result.push(b);
            }
            (true, false) => {
                if result.is_empty() { result.push(a); }
                if let Some((_, exit)) = clip_segment(a, b, x_min, y_min, x_max, y_max) {
                    result.push(exit);
                }
            }
            (false, true) => {
                if let Some((entry, _)) = clip_segment(a, b, x_min, y_min, x_max, y_max) {
                    result.push(entry);
                }
                result.push(b);
            }
            (false, false) => {
                // Both outside — might still cross through for very long segments.
                if let Some((entry, exit)) = clip_segment(a, b, x_min, y_min, x_max, y_max) {
                    result.push(entry);
                    result.push(exit);
                }
            }
        }
    }

    result
}

/// Clip a closed polygon to a rectangle using Sutherland-Hodgman.
/// Input polygon should NOT have a repeated closing vertex.
fn clip_polygon_to_rect(
    polygon: &[(f64, f64)],
    x_min: f64, y_min: f64,
    x_max: f64, y_max: f64,
) -> Vec<(f64, f64)> {
    let mut output = polygon.to_vec();

    // Clip against each edge in turn
    output = sh_clip_edge(&output, |p| p.0 >= x_min, |a, b| {
        let t = (x_min - a.0) / (b.0 - a.0);
        (x_min, a.1 + t * (b.1 - a.1))
    });
    output = sh_clip_edge(&output, |p| p.0 <= x_max, |a, b| {
        let t = (x_max - a.0) / (b.0 - a.0);
        (x_max, a.1 + t * (b.1 - a.1))
    });
    output = sh_clip_edge(&output, |p| p.1 >= y_min, |a, b| {
        let t = (y_min - a.1) / (b.1 - a.1);
        (a.0 + t * (b.0 - a.0), y_min)
    });
    output = sh_clip_edge(&output, |p| p.1 <= y_max, |a, b| {
        let t = (y_max - a.1) / (b.1 - a.1);
        (a.0 + t * (b.0 - a.0), y_max)
    });

    output
}

/// One pass of Sutherland-Hodgman: clip polygon against a single edge.
fn sh_clip_edge(
    polygon: &[(f64, f64)],
    inside: impl Fn((f64, f64)) -> bool,
    intersect: impl Fn((f64, f64), (f64, f64)) -> (f64, f64),
) -> Vec<(f64, f64)> {
    if polygon.is_empty() { return vec![]; }

    let mut output = Vec::new();
    let n = polygon.len();

    for i in 0..n {
        let current = polygon[i];
        let next = polygon[(i + 1) % n];
        let cur_in = inside(current);
        let nxt_in = inside(next);

        match (cur_in, nxt_in) {
            (true, true) => output.push(next),
            (true, false) => output.push(intersect(current, next)),
            (false, true) => {
                output.push(intersect(current, next));
                output.push(next);
            }
            (false, false) => {}
        }
    }

    output
}

/// Liang-Barsky segment clip: returns clipped (entry, exit) points, or None.
fn clip_segment(
    a: (f64, f64), b: (f64, f64),
    x_min: f64, y_min: f64, x_max: f64, y_max: f64,
) -> Option<((f64, f64), (f64, f64))> {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let mut t_min = 0.0_f64;
    let mut t_max = 1.0_f64;

    let p = [-dx, dx, -dy, dy];
    let q = [a.0 - x_min, x_max - a.0, a.1 - y_min, y_max - a.1];

    for i in 0..4 {
        if p[i].abs() < 1e-10 {
            if q[i] < 0.0 { return None; }
        } else {
            let t = q[i] / p[i];
            if p[i] < 0.0 {
                t_min = t_min.max(t);
            } else {
                t_max = t_max.min(t);
            }
        }
    }

    if t_min > t_max { return None; }

    Some((
        (a.0 + dx * t_min, a.1 + dy * t_min),
        (a.0 + dx * t_max, a.1 + dy * t_max),
    ))
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
    exclusion_zones: &[ExclusionZone],
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

            // Skip candidates inside exclusion zones (rooms)
            if exclusion_zones.iter().any(|z| z.contains(nx, ny)) {
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
            &[],
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
    fn test_road_meandering_style() {
        let noise = test_noise();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let config = RoadConfig {
            from: Edge::Left,
            to: Edge::Right,
            style: PathStyle::Meandering,
            ..Default::default()
        };

        let result = generate_road(&noise, &config, 2560.0, 2560.0, &alloc, &mut rng);
        assert!(result.is_some());
        let road = result.unwrap();
        assert!(road.corridor_points.len() >= 2);
    }

    #[test]
    fn test_river_straight_style() {
        let noise = test_noise();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let config = RiverConfig {
            style: PathStyle::Straight,
            ..Default::default()
        };

        let result = generate_river(&noise, &config, 2560.0, 2560.0, &alloc, &mut rng);
        assert!(result.is_some());
        let river = result.unwrap();
        assert_eq!(river.bank_paths.len(), 2);
        assert!(river.water_polygon.len() >= 4);
    }

    #[test]
    fn test_path_style_serde() {
        let yaml = "straight";
        let style: PathStyle = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(style, PathStyle::Straight);

        let yaml = "meandering";
        let style: PathStyle = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(style, PathStyle::Meandering);
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
