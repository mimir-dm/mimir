//! Polygon-based layout system.
//!
//! Users define closed polygons in grid coordinates. All polygon edges are
//! collected, shared edges (overlap between any two polygons) are removed,
//! and the surviving segments are chained into closed loops for DD output.
//! Each polygon also registers a DD shape for interior fill.

use serde::{Deserialize, Serialize};

use crate::format::entities::{MapPortal, MapWall};
use crate::format::godot_types::Vector2;
use crate::format::NodeIdAllocator;
use crate::rooms::{PortalType, PIXELS_PER_GRID};

/// A closed polygon area defined in grid coordinates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonConfig {
    /// Unique identifier for this polygon.
    pub id: String,
    /// Vertices in grid coordinates, listed clockwise. The polygon is
    /// automatically closed (last vertex connects back to first).
    pub points: Vec<[f64; 2]>,
    /// Terrain texture slot to fill the interior (0–3).
    #[serde(default)]
    pub terrain_slot: Option<usize>,
    /// Wall texture override (default: stone).
    #[serde(default = "default_wall_texture")]
    pub wall_texture: String,
    /// Portal declarations on this polygon's edges.
    #[serde(default)]
    pub portals: Vec<PolygonPortalConfig>,
}

fn default_wall_texture() -> String {
    "res://textures/walls/stone.png".to_string()
}

/// A portal on a polygon edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonPortalConfig {
    /// Edge index (0-based, edge i connects point[i] to point[i+1]).
    pub edge: usize,
    /// Fractional position along the edge (0.0 = start, 1.0 = end, 0.5 = center).
    #[serde(default = "default_position")]
    pub position: f64,
    /// Portal type.
    #[serde(rename = "type")]
    pub portal_type: PortalType,
}

fn default_position() -> f64 {
    0.5
}

/// A terrain override that respects polygon boundaries (not just bounding box).
#[derive(Debug, Clone)]
pub struct PolygonTerrainOverride {
    /// Polygon vertices in grid coordinates.
    pub points: Vec<[f64; 2]>,
    /// Terrain slot to fill (0–3).
    pub slot: usize,
}

impl PolygonTerrainOverride {
    /// Apply this override to a terrain splat map, only filling cells
    /// whose center lies inside the polygon.
    pub fn apply(&self, splat_data: &mut [u8], map_cells_x: usize) {
        let min_x = self.points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
        let min_y = self.points.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
        let max_x = self.points.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
        let max_y = self.points.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);

        let cell_x0 = (min_x * 4.0).floor() as usize;
        let cell_y0 = (min_y * 4.0).floor() as usize;
        let cell_x1 = (max_x * 4.0).ceil() as usize;
        let cell_y1 = (max_y * 4.0).ceil() as usize;

        let mut weights = [0u8; 4];
        weights[self.slot] = 255;

        for cy in cell_y0..cell_y1 {
            for cx in cell_x0..cell_x1 {
                // Cell center in grid coordinates.
                let gx = (cx as f64 + 0.5) / 4.0;
                let gy = (cy as f64 + 0.5) / 4.0;
                if point_in_polygon(&[gx, gy], &self.points) {
                    let idx = (cy * map_cells_x + cx) * 4;
                    if idx + 4 <= splat_data.len() {
                        splat_data[idx..idx + 4].copy_from_slice(&weights);
                    }
                }
            }
        }
    }
}

/// Result of polygon layout generation.
#[derive(Debug, Clone)]
pub struct PolygonLayoutResult {
    pub walls: Vec<MapWall>,
    pub portals: Vec<MapPortal>,
    pub shape_wall_ids: Vec<serde_json::Value>,
    pub shape_polygons: Vec<serde_json::Value>,
    pub terrain_overrides: Vec<PolygonTerrainOverride>,
}

// ── Geometry primitives ─────────────────────────────────────────────────

/// A directed line segment with an integer key for its source polygon.
#[derive(Debug, Clone)]
struct Segment {
    start: [f64; 2],
    end: [f64; 2],
}

/// Tolerance for treating two coordinates as equal.
const EPS: f64 = 0.5;

fn pts_eq(a: &[f64; 2], b: &[f64; 2]) -> bool {
    (a[0] - b[0]).abs() <= EPS && (a[1] - b[1]).abs() <= EPS
}

// ── Step 1: Collect all edges ───────────────────────────────────────────

/// Extract directed edges from all polygons (in pixel coordinates).
fn collect_edges(polygons: &[PolygonConfig]) -> Vec<Segment> {
    let mut edges = Vec::new();
    for poly in polygons {
        let pts = &poly.points;
        let n = pts.len();
        for i in 0..n {
            let j = (i + 1) % n;
            edges.push(Segment {
                start: [pts[i][0] * PIXELS_PER_GRID, pts[i][1] * PIXELS_PER_GRID],
                end: [pts[j][0] * PIXELS_PER_GRID, pts[j][1] * PIXELS_PER_GRID],
            });
        }
    }
    edges
}

// ── Step 2: Remove shared edges ─────────────────────────────────────────

/// Two segments overlap if they are collinear and share any interval.
/// Returns the non-overlapping remainders of segment `a` after subtracting
/// the overlap with segment `b`. Returns None if no overlap.
fn subtract_overlap(a: &Segment, b: &Segment) -> Option<Vec<Segment>> {
    let ax = a.end[0] - a.start[0];
    let ay = a.end[1] - a.start[1];
    let len_a = (ax * ax + ay * ay).sqrt();
    if len_a < EPS {
        return None;
    }

    let bx = b.end[0] - b.start[0];
    let by = b.end[1] - b.start[1];
    let len_b = (bx * bx + by * by).sqrt();
    if len_b < EPS {
        return None;
    }

    // Check parallel: normalized cross product ≈ 0.
    let cross = ax * by - ay * bx;
    if (cross / (len_a * len_b)).abs() > 0.01 {
        return None;
    }

    // Check collinear: b.start lies on a's line.
    let to_b_x = b.start[0] - a.start[0];
    let to_b_y = b.start[1] - a.start[1];
    let perp = (ax * to_b_y - ay * to_b_x).abs() / len_a;
    if perp > EPS {
        return None;
    }

    // Project b's endpoints onto a's parametric line [0, 1].
    let t0 = (to_b_x * ax + to_b_y * ay) / (len_a * len_a);
    let t1 = {
        let dx = b.end[0] - a.start[0];
        let dy = b.end[1] - a.start[1];
        (dx * ax + dy * ay) / (len_a * len_a)
    };

    let t_min = t0.min(t1).max(0.0);
    let t_max = t0.max(t1).min(1.0);

    if t_max - t_min < 1e-6 {
        return None; // No overlap.
    }

    // Compute remainders of a after removing [t_min, t_max].
    let mut remainders = Vec::new();
    if t_min > 1e-6 {
        remainders.push(Segment {
            start: a.start,
            end: [a.start[0] + ax * t_min, a.start[1] + ay * t_min],
        });
    }
    if t_max < 1.0 - 1e-6 {
        remainders.push(Segment {
            start: [a.start[0] + ax * t_max, a.start[1] + ay * t_max],
            end: a.end,
        });
    }

    Some(remainders)
}

/// Remove all shared (overlapping) portions between edges.
/// Returns only the non-shared segments.
fn remove_shared_edges(edges: Vec<Segment>) -> Vec<Segment> {
    // For each edge, subtract all overlapping portions from other edges.
    let mut result: Vec<Segment> = Vec::new();

    for (i, edge) in edges.iter().enumerate() {
        // Start with the full edge, then whittle it down.
        let mut current = vec![edge.clone()];

        for (j, other) in edges.iter().enumerate() {
            if i == j {
                continue;
            }

            let mut next = Vec::new();
            for seg in &current {
                match subtract_overlap(seg, other) {
                    Some(remainders) => next.extend(remainders),
                    None => next.push(seg.clone()),
                }
            }
            current = next;
        }

        // Keep only non-degenerate segments.
        for seg in current {
            let dx = seg.end[0] - seg.start[0];
            let dy = seg.end[1] - seg.start[1];
            if dx * dx + dy * dy > EPS * EPS {
                result.push(seg);
            }
        }
    }

    result
}

// ── Step 2b: Split edges at crossing intersections ─────────────────────

/// Find the crossing point of two non-collinear line segments.
/// Returns (intersection_point, t_on_a, u_on_b), or None.
fn segment_crossing(a: &Segment, b: &Segment) -> Option<([f64; 2], f64, f64)> {
    let ax = a.end[0] - a.start[0];
    let ay = a.end[1] - a.start[1];
    let bx = b.end[0] - b.start[0];
    let by = b.end[1] - b.start[1];

    let denom = ax * by - ay * bx;
    if denom.abs() < 1e-10 {
        return None; // Parallel or collinear.
    }

    let cx = b.start[0] - a.start[0];
    let cy = b.start[1] - a.start[1];

    let t = (cx * by - cy * bx) / denom;
    let u = (cx * ay - cy * ax) / denom;

    // Must be interior to both segments (not at endpoints).
    let margin = 1e-6;
    if t > margin && t < 1.0 - margin && u > margin && u < 1.0 - margin {
        let point = [a.start[0] + ax * t, a.start[1] + ay * t];
        Some((point, t, u))
    } else {
        None
    }
}

/// Split all edges at crossing intersection points.
fn split_at_crossings(edges: Vec<Segment>) -> Vec<Segment> {
    let n = edges.len();
    let mut splits: Vec<Vec<f64>> = vec![vec![]; n];

    for i in 0..n {
        for j in (i + 1)..n {
            if let Some((_, t, u)) = segment_crossing(&edges[i], &edges[j]) {
                splits[i].push(t);
                splits[j].push(u);
            }
        }
    }

    let mut result = Vec::new();
    for (i, edge) in edges.iter().enumerate() {
        let mut ts = splits[i].clone();
        if ts.is_empty() {
            result.push(edge.clone());
            continue;
        }

        ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ts.dedup_by(|a, b| (*a - *b).abs() < 1e-6);

        let dx = edge.end[0] - edge.start[0];
        let dy = edge.end[1] - edge.start[1];
        let mut prev_t = 0.0;

        for t in ts {
            if t - prev_t > 1e-6 {
                result.push(Segment {
                    start: [edge.start[0] + dx * prev_t, edge.start[1] + dy * prev_t],
                    end: [edge.start[0] + dx * t, edge.start[1] + dy * t],
                });
            }
            prev_t = t;
        }
        if 1.0 - prev_t > 1e-6 {
            result.push(Segment {
                start: [edge.start[0] + dx * prev_t, edge.start[1] + dy * prev_t],
                end: edge.end,
            });
        }
    }

    result
}

// ── Step 3: Walk CW to form union boundary ────────────────────────────

/// Signed turn angle from direction d_fwd to d_out.
/// In y-down coords: negative = left turn (CCW), positive = right turn (CW).
fn turn_angle(d_fwd: &[f64; 2], d_out: &[f64; 2]) -> f64 {
    let cross = d_fwd[0] * d_out[1] - d_fwd[1] * d_out[0];
    let dot = d_fwd[0] * d_out[0] + d_fwd[1] * d_out[1];
    cross.atan2(dot)
}

/// Walk the directed segment graph clockwise, using the leftmost-turn rule
/// at junctions to trace union boundaries.
///
/// At each junction (where multiple outgoing edges exist), picks the
/// most-left (most CCW / most negative turn angle in y-down) outgoing edge.
/// This traces the outer boundary of overlapping polygon unions.
fn walk_cw_loops(segments: Vec<Segment>) -> Vec<Vec<[f64; 2]>> {
    if segments.is_empty() {
        return vec![];
    }

    let mut used = vec![false; segments.len()];
    let mut loops = Vec::new();

    loop {
        // Find the topmost-leftmost unused segment start, then pick
        // the most-rightward (smallest atan2) outgoing edge from it.
        let start_idx = {
            let mut best_point: Option<[f64; 2]> = None;
            for (i, seg) in segments.iter().enumerate() {
                if used[i] {
                    continue;
                }
                let p = seg.start;
                if let Some(bp) = best_point {
                    if p[1] < bp[1] - EPS || ((p[1] - bp[1]).abs() < EPS && p[0] < bp[0] - EPS)
                    {
                        best_point = Some(p);
                    }
                } else {
                    best_point = Some(p);
                }
            }

            match best_point {
                None => break,
                Some(bp) => {
                    // Among unused segments starting at bp, pick most rightward.
                    let mut best: Option<(usize, f64)> = None;
                    for (i, seg) in segments.iter().enumerate() {
                        if used[i] || !pts_eq(&seg.start, &bp) {
                            continue;
                        }
                        let dx = seg.end[0] - seg.start[0];
                        let dy = seg.end[1] - seg.start[1];
                        let angle = dy.atan2(dx);
                        if best.is_none() || angle < best.unwrap().1 {
                            best = Some((i, angle));
                        }
                    }
                    match best {
                        Some((idx, _)) => idx,
                        None => break,
                    }
                }
            }
        };

        let mut chain = vec![segments[start_idx].start];
        used[start_idx] = true;
        chain.push(segments[start_idx].end);

        let max_steps = segments.len() + 1;
        for _ in 0..max_steps {
            let tail = *chain.last().unwrap();

            if pts_eq(&tail, &chain[0]) && chain.len() > 2 {
                chain.pop(); // Remove duplicate closing point.
                break;
            }

            // Incoming direction.
            let prev = chain[chain.len() - 2];
            let d_fwd = [tail[0] - prev[0], tail[1] - prev[1]];

            // Find unused segments starting at tail, pick leftmost turn.
            let mut best: Option<(usize, f64)> = None;
            for (i, seg) in segments.iter().enumerate() {
                if used[i] || !pts_eq(&seg.start, &tail) {
                    continue;
                }
                let d_out = [seg.end[0] - seg.start[0], seg.end[1] - seg.start[1]];
                let angle = turn_angle(&d_fwd, &d_out);
                if best.is_none() || angle < best.unwrap().1 {
                    best = Some((i, angle));
                }
            }

            match best {
                Some((idx, _)) => {
                    used[idx] = true;
                    chain.push(segments[idx].end);
                }
                None => break, // Dead end.
            }
        }

        if chain.len() >= 3 {
            loops.push(remove_collinear_points(chain));
        }
    }

    // Filter out interior loops (contained within a larger loop).
    filter_interior_loops(loops)
}

/// Remove collinear midpoints from a closed polygon loop.
/// If three consecutive points are collinear, the middle one is redundant.
fn remove_collinear_points(pts: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    if pts.len() < 3 {
        return pts;
    }
    let n = pts.len();
    let mut keep = Vec::with_capacity(n);
    for i in 0..n {
        let prev = pts[(i + n - 1) % n];
        let curr = pts[i];
        let next = pts[(i + 1) % n];
        // Cross product of (prev→curr) × (curr→next). Zero means collinear.
        let cross = (curr[0] - prev[0]) * (next[1] - curr[1])
            - (curr[1] - prev[1]) * (next[0] - curr[0]);
        if cross.abs() > 0.1 {
            keep.push(curr);
        }
    }
    keep
}

/// Remove loops whose centroid lies inside a larger loop.
fn filter_interior_loops(loops: Vec<Vec<[f64; 2]>>) -> Vec<Vec<[f64; 2]>> {
    if loops.len() <= 1 {
        return loops;
    }

    let areas: Vec<f64> = loops.iter().map(|l| polygon_area(l)).collect();
    let mut keep = vec![true; loops.len()];

    for j in 0..loops.len() {
        if !keep[j] {
            continue;
        }
        let c = polygon_centroid(&loops[j]);
        for i in 0..loops.len() {
            if i == j || !keep[i] {
                continue;
            }
            if areas[i] > areas[j] && point_in_polygon(&c, &loops[i]) {
                keep[j] = false;
                break;
            }
        }
    }

    loops
        .into_iter()
        .zip(keep)
        .filter(|(_, k)| *k)
        .map(|(l, _)| l)
        .collect()
}

/// Absolute area of a polygon (shoelace formula).
fn polygon_area(pts: &[[f64; 2]]) -> f64 {
    let n = pts.len();
    let mut sum = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        sum += pts[i][0] * pts[j][1] - pts[j][0] * pts[i][1];
    }
    sum.abs() / 2.0
}

/// Centroid of a polygon (average of vertices).
fn polygon_centroid(pts: &[[f64; 2]]) -> [f64; 2] {
    let n = pts.len() as f64;
    let sx: f64 = pts.iter().map(|p| p[0]).sum();
    let sy: f64 = pts.iter().map(|p| p[1]).sum();
    [sx / n, sy / n]
}

/// Point-in-polygon test using ray casting.
fn point_in_polygon(point: &[f64; 2], polygon: &[[f64; 2]]) -> bool {
    let n = polygon.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let yi = polygon[i][1];
        let yj = polygon[j][1];
        if (yi > point[1]) != (yj > point[1]) {
            let x_int = polygon[i][0]
                + (point[1] - yi) * (polygon[j][0] - polygon[i][0]) / (yj - yi);
            if point[0] < x_int {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

// ── Public API ──────────────────────────────────────────────────────────

/// Generate the polygon-based layout.
///
/// 1. Collect all edges from all polygons.
/// 2. Remove shared edges (any segment that overlaps with another).
/// 3. Chain surviving segments into closed loops.
/// 4. Emit each loop as a closed-loop DD wall.
/// 5. Register each input polygon as a DD shape for interior fill.
pub fn generate_polygon_layout(
    polygons: &[PolygonConfig],
    alloc: &NodeIdAllocator,
) -> PolygonLayoutResult {
    let mut result = PolygonLayoutResult {
        walls: Vec::new(),
        portals: Vec::new(),
        shape_wall_ids: Vec::new(),
        shape_polygons: Vec::new(),
        terrain_overrides: Vec::new(),
    };

    if polygons.is_empty() {
        return result;
    }

    // Step 1: Collect edges, split at crossings, remove collinear overlaps, walk CW.
    let edges = collect_edges(polygons);
    let split = split_at_crossings(edges);
    let surviving = remove_shared_edges(split);
    let loops = walk_cw_loops(surviving);

    // Step 4: each loop becomes one wall + one shape. Track wall IDs for portals.
    let mut wall_ids: Vec<String> = Vec::new();
    for chain in &loops {
        let wall_id = alloc.next();
        wall_ids.push(wall_id.clone());
        let pts: Vec<Vector2> = chain.iter().map(|p| Vector2::new(p[0], p[1])).collect();
        let texture = &polygons[0].wall_texture;
        let wall = MapWall::new_room(pts.clone(), texture, &wall_id);
        result.walls.push(wall);

        // Shape: same points as the wall, references this wall.
        let wall_id_decimal: i64 = i64::from_str_radix(&wall_id, 16).unwrap_or(0);
        result
            .shape_wall_ids
            .push(serde_json::Value::Number(serde_json::Number::from(
                wall_id_decimal,
            )));

        let polygon_str = format!(
            "PoolVector2Array( {} )",
            pts.iter()
                .map(|p| format!("{}, {}", p.x, p.y))
                .collect::<Vec<_>>()
                .join(", ")
        );
        result
            .shape_polygons
            .push(serde_json::Value::String(polygon_str));
    }

    // Step 5: generate portals on polygon edges.
    let (freestanding, anchored) = generate_portals(polygons, &loops, &wall_ids, alloc);
    result.portals = freestanding;
    for (loop_idx, portal) in anchored {
        result.walls[loop_idx].portals.push(portal);
    }

    // Terrain overrides from input polygons (per-cell point-in-polygon fill).
    for poly in polygons {
        if let Some(slot) = poly.terrain_slot {
            result.terrain_overrides.push(PolygonTerrainOverride {
                points: poly.points.clone(),
                slot,
            });
        }
    }

    result
}

// ── Portal generation ──────────────────────────────────────────────────

/// Check if a point lies on a line segment. Returns the parametric t value
/// (0.0 = start, 1.0 = end) if the point is on the segment, None otherwise.
fn point_on_segment(point: &[f64; 2], seg_start: &[f64; 2], seg_end: &[f64; 2]) -> Option<f64> {
    let dx = seg_end[0] - seg_start[0];
    let dy = seg_end[1] - seg_start[1];
    let len_sq = dx * dx + dy * dy;
    if len_sq < EPS * EPS {
        return None;
    }

    // Project point onto segment line.
    let px = point[0] - seg_start[0];
    let py = point[1] - seg_start[1];

    // Perpendicular distance from line.
    let len = len_sq.sqrt();
    let perp = (dx * py - dy * px).abs() / len;
    if perp > EPS {
        return None;
    }

    // Parametric position along segment.
    let t = (px * dx + py * dy) / len_sq;
    if t >= -0.01 && t <= 1.01 {
        Some(t.clamp(0.0, 1.0))
    } else {
        None
    }
}

/// Compute portal rotation and direction for an edge.
///
/// DD portals use `rotation` as the angle of the wall edge (not the normal).
/// The `direction` vector is `(cos(rotation), sin(rotation))`.
/// - Horizontal wall (E-W): rotation = 0, direction = (1, 0)
/// - Vertical wall (N-S): rotation = π/2, direction = (0, 1)
fn edge_orientation(edge_start: &[f64; 2], edge_end: &[f64; 2]) -> (Vector2, f64) {
    let dx = edge_end[0] - edge_start[0];
    let dy = edge_end[1] - edge_start[1];
    let len = (dx * dx + dy * dy).sqrt();
    if len < EPS {
        return (Vector2::new(1.0, 0.0), 0.0);
    }
    let rotation = dy.atan2(dx);
    let dir = Vector2::new(rotation.cos(), rotation.sin());
    (dir, rotation)
}

/// Generate portals for all polygons after wall loop computation.
///
/// For each portal config on a polygon:
/// 1. Compute its pixel position on the original edge.
/// 2. Search the merged loops for a segment containing that point.
/// 3. If found → wall-anchored portal (embedded in the wall).
/// 4. If not found → freestanding portal (shared edge was removed).
fn generate_portals(
    polygons: &[PolygonConfig],
    loops: &[Vec<[f64; 2]>],
    wall_ids: &[String],
    alloc: &NodeIdAllocator,
) -> (Vec<MapPortal>, Vec<(usize, MapPortal)>) {
    // Returns: (freestanding, vec of (loop_index, wall-anchored portal))
    let mut freestanding = Vec::new();
    let mut anchored = Vec::new();

    for poly in polygons {
        let n = poly.points.len();
        for portal_cfg in &poly.portals {
            if portal_cfg.edge >= n {
                continue; // Invalid edge index, skip.
            }

            // Original edge in pixel coordinates.
            let edge_start = [
                poly.points[portal_cfg.edge][0] * PIXELS_PER_GRID,
                poly.points[portal_cfg.edge][1] * PIXELS_PER_GRID,
            ];
            let j = (portal_cfg.edge + 1) % n;
            let edge_end = [
                poly.points[j][0] * PIXELS_PER_GRID,
                poly.points[j][1] * PIXELS_PER_GRID,
            ];

            // Portal position on the original edge.
            let t = portal_cfg.position;
            let portal_pos = [
                edge_start[0] + (edge_end[0] - edge_start[0]) * t,
                edge_start[1] + (edge_end[1] - edge_start[1]) * t,
            ];

            // Perpendicular direction from the original edge.
            let (direction, rotation) = edge_orientation(&edge_start, &edge_end);
            let radius = portal_cfg.portal_type.default_radius();
            let texture = portal_cfg.portal_type.default_texture();
            let pos = Vector2::new(portal_pos[0], portal_pos[1]);

            // Search loops for a segment containing this point.
            let mut found = false;
            for (loop_idx, chain) in loops.iter().enumerate() {
                let seg_count = chain.len();
                for seg_i in 0..seg_count {
                    let seg_start = &chain[seg_i];
                    let seg_end = &chain[(seg_i + 1) % seg_count];

                    if let Some(seg_t) = point_on_segment(&portal_pos, seg_start, seg_end) {
                        // Wall-anchored: point_index = segment index in the loop,
                        // wall_distance = segment_index + fractional position along
                        // that segment (DD uses absolute perimeter offset).
                        let wall_distance = seg_i as f64 + seg_t;
                        let portal = MapPortal::new(
                            pos,
                            rotation,
                            direction,
                            texture,
                            radius,
                            seg_i as i32,
                            &wall_ids[loop_idx],
                            wall_distance,
                            &alloc.next(),
                        );
                        anchored.push((loop_idx, portal));
                        found = true;
                        break;
                    }
                }
                if found {
                    break;
                }
            }

            if !found {
                // Freestanding: shared edge was removed, portal floats in space.
                let portal = MapPortal::new_freestanding(
                    pos,
                    rotation,
                    direction,
                    texture,
                    radius,
                    &alloc.next(),
                );
                freestanding.push(portal);
            }
        }
    }

    (freestanding, anchored)
}

/// Build exclusion zones from polygon configs.
pub fn build_polygon_exclusion_zones(
    polygons: &[PolygonConfig],
) -> Vec<crate::rooms::ExclusionZone> {
    let padding = PIXELS_PER_GRID;

    polygons
        .iter()
        .map(|poly| {
            let min_x = poly
                .points
                .iter()
                .map(|p| p[0])
                .fold(f64::INFINITY, f64::min);
            let min_y = poly
                .points
                .iter()
                .map(|p| p[1])
                .fold(f64::INFINITY, f64::min);
            let max_x = poly
                .points
                .iter()
                .map(|p| p[0])
                .fold(f64::NEG_INFINITY, f64::max);
            let max_y = poly
                .points
                .iter()
                .map(|p| p[1])
                .fold(f64::NEG_INFINITY, f64::max);

            crate::rooms::ExclusionZone {
                x: min_x * PIXELS_PER_GRID - padding,
                y: min_y * PIXELS_PER_GRID - padding,
                width: (max_x - min_x) * PIXELS_PER_GRID + padding * 2.0,
                height: (max_y - min_y) * PIXELS_PER_GRID + padding * 2.0,
            }
        })
        .collect()
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── remove_shared_edges ─────────────────────────────────────────

    #[test]
    fn single_square_no_removal() {
        // One square: 4 edges, nothing shared → 4 segments survive.
        let edges = vec![
            Segment { start: [0.0, 0.0], end: [10.0, 0.0] },
            Segment { start: [10.0, 0.0], end: [10.0, 10.0] },
            Segment { start: [10.0, 10.0], end: [0.0, 10.0] },
            Segment { start: [0.0, 10.0], end: [0.0, 0.0] },
        ];
        let result = remove_shared_edges(edges);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn two_adjacent_squares_shared_edge_removed() {
        // Left square: (0,0)-(10,0)-(10,10)-(0,10)
        // Right square: (10,0)-(20,0)-(20,10)-(10,10)
        // Shared edge: left's east (10,0)→(10,10) and right's west (10,10)→(10,0)
        let edges = vec![
            // Left
            Segment { start: [0.0, 0.0], end: [10.0, 0.0] },
            Segment { start: [10.0, 0.0], end: [10.0, 10.0] },  // shared
            Segment { start: [10.0, 10.0], end: [0.0, 10.0] },
            Segment { start: [0.0, 10.0], end: [0.0, 0.0] },
            // Right
            Segment { start: [10.0, 0.0], end: [20.0, 0.0] },
            Segment { start: [20.0, 0.0], end: [20.0, 10.0] },
            Segment { start: [20.0, 10.0], end: [10.0, 10.0] },
            Segment { start: [10.0, 10.0], end: [10.0, 0.0] },  // shared
        ];
        let result = remove_shared_edges(edges);
        // 8 edges - 2 shared = 6 surviving
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn partial_overlap_splits_edge() {
        // Long edge (0,0)→(20,0) partially overlapped by (5,0)→(15,0)
        let edges = vec![
            Segment { start: [0.0, 0.0], end: [20.0, 0.0] },
            Segment { start: [15.0, 0.0], end: [5.0, 0.0] },  // reverse direction overlap
        ];
        let result = remove_shared_edges(edges);
        // Long edge splits into (0,0)→(5,0) and (15,0)→(20,0) = 2 segments
        // Short edge completely removed = 0 segments
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn three_squares_in_a_row() {
        // Three squares side by side: 12 edges, 4 shared → 8 survive
        let edges = vec![
            // Square 0: (0,0)-(10,0)-(10,10)-(0,10)
            Segment { start: [0.0, 0.0], end: [10.0, 0.0] },
            Segment { start: [10.0, 0.0], end: [10.0, 10.0] },
            Segment { start: [10.0, 10.0], end: [0.0, 10.0] },
            Segment { start: [0.0, 10.0], end: [0.0, 0.0] },
            // Square 1: (10,0)-(20,0)-(20,10)-(10,10)
            Segment { start: [10.0, 0.0], end: [20.0, 0.0] },
            Segment { start: [20.0, 0.0], end: [20.0, 10.0] },
            Segment { start: [20.0, 10.0], end: [10.0, 10.0] },
            Segment { start: [10.0, 10.0], end: [10.0, 0.0] },
            // Square 2: (20,0)-(30,0)-(30,10)-(20,10)
            Segment { start: [20.0, 0.0], end: [30.0, 0.0] },
            Segment { start: [30.0, 0.0], end: [30.0, 10.0] },
            Segment { start: [30.0, 10.0], end: [20.0, 10.0] },
            Segment { start: [20.0, 10.0], end: [20.0, 0.0] },
        ];
        let result = remove_shared_edges(edges);
        assert_eq!(result.len(), 8);
    }

    // ── walk_cw_loops ────────────────────────────────────────────

    #[test]
    fn chain_simple_square() {
        let segments = vec![
            Segment { start: [0.0, 0.0], end: [10.0, 0.0] },
            Segment { start: [10.0, 0.0], end: [10.0, 10.0] },
            Segment { start: [10.0, 10.0], end: [0.0, 10.0] },
            Segment { start: [0.0, 10.0], end: [0.0, 0.0] },
        ];
        let loops = walk_cw_loops(segments);
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].len(), 4);
    }

    #[test]
    fn chain_two_adjacent_squares_forms_one_loop() {
        // After removing shared edges from two adjacent squares,
        // the 6 surviving segments should form one closed loop.
        let edges = vec![
            // Left
            Segment { start: [0.0, 0.0], end: [10.0, 0.0] },
            Segment { start: [10.0, 0.0], end: [10.0, 10.0] },
            Segment { start: [10.0, 10.0], end: [0.0, 10.0] },
            Segment { start: [0.0, 10.0], end: [0.0, 0.0] },
            // Right
            Segment { start: [10.0, 0.0], end: [20.0, 0.0] },
            Segment { start: [20.0, 0.0], end: [20.0, 10.0] },
            Segment { start: [20.0, 10.0], end: [10.0, 10.0] },
            Segment { start: [10.0, 10.0], end: [10.0, 0.0] },
        ];
        let surviving = remove_shared_edges(edges);
        let loops = walk_cw_loops(surviving);
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].len(), 4); // clean rectangle, collinear midpoints collapsed
    }

    #[test]
    fn chain_l_shape() {
        // Room (0,0)-(10,0)-(10,10)-(0,10) + corridor (3,10)-(7,10)-(7,15)-(3,15)
        // Shared: part of room's south edge overlaps corridor's north edge
        let edges = vec![
            // Room
            Segment { start: [0.0, 0.0], end: [10.0, 0.0] },
            Segment { start: [10.0, 0.0], end: [10.0, 10.0] },
            Segment { start: [10.0, 10.0], end: [0.0, 10.0] },
            Segment { start: [0.0, 10.0], end: [0.0, 0.0] },
            // Corridor
            Segment { start: [3.0, 10.0], end: [7.0, 10.0] },
            Segment { start: [7.0, 10.0], end: [7.0, 15.0] },
            Segment { start: [7.0, 15.0], end: [3.0, 15.0] },
            Segment { start: [3.0, 15.0], end: [3.0, 10.0] },
        ];
        let surviving = remove_shared_edges(edges);
        let loops = walk_cw_loops(surviving);
        assert_eq!(loops.len(), 1);
        // L-shape: 8 vertices
        assert_eq!(loops[0].len(), 8);
    }

    // ── Full pipeline via generate_polygon_layout ───────────────────

    #[test]
    fn single_polygon_one_closed_wall() {
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![PolygonConfig {
            id: "room".to_string(),
            points: vec![[2.0, 2.0], [6.0, 2.0], [6.0, 5.0], [2.0, 5.0]],
            terrain_slot: Some(3),
            wall_texture: default_wall_texture(),
            portals: vec![],
        }];

        let result = generate_polygon_layout(&polygons, &alloc);

        assert_eq!(result.walls.len(), 1);
        assert!(result.walls[0].is_loop);
        assert_eq!(result.shape_wall_ids.len(), 1);
        assert_eq!(result.shape_polygons.len(), 1);
        assert_eq!(result.terrain_overrides.len(), 1);
    }

    #[test]
    fn two_adjacent_polygons_one_closed_wall() {
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "left".to_string(),
                points: vec![[2.0, 2.0], [6.0, 2.0], [6.0, 5.0], [2.0, 5.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "right".to_string(),
                points: vec![[6.0, 2.0], [10.0, 2.0], [10.0, 5.0], [6.0, 5.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        // Shared edge removed → one merged loop = one wall, one shape.
        assert_eq!(result.walls.len(), 1);
        assert!(result.walls[0].is_loop);
        assert_eq!(result.shape_wall_ids.len(), 1);
        assert_eq!(result.shape_polygons.len(), 1);
    }

    #[test]
    fn exclusion_zones() {
        let polygons = vec![PolygonConfig {
            id: "room".to_string(),
            points: vec![[4.0, 4.0], [8.0, 4.0], [8.0, 8.0], [4.0, 8.0]],
            terrain_slot: None,
            wall_texture: default_wall_texture(),
            portals: vec![],
        }];

        let zones = build_polygon_exclusion_zones(&polygons);
        assert_eq!(zones.len(), 1);

        let z = &zones[0];
        assert_eq!(z.x, 4.0 * 256.0 - 256.0);
        assert_eq!(z.y, 4.0 * 256.0 - 256.0);
        assert_eq!(z.width, 4.0 * 256.0 + 512.0);
        assert_eq!(z.height, 4.0 * 256.0 + 512.0);
    }

    // ── Portal generation ────────────────────────────────────────────

    #[test]
    fn portal_on_perimeter_wall_anchored() {
        // Single polygon with a door on edge 0 (north wall).
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![PolygonConfig {
            id: "room".to_string(),
            points: vec![[2.0, 2.0], [6.0, 2.0], [6.0, 5.0], [2.0, 5.0]],
            terrain_slot: Some(3),
            wall_texture: default_wall_texture(),
            portals: vec![PolygonPortalConfig {
                edge: 0,
                position: 0.5,
                portal_type: PortalType::Door,
            }],
        }];

        let result = generate_polygon_layout(&polygons, &alloc);

        // One wall, door should be wall-anchored (embedded in wall.portals).
        assert_eq!(result.walls.len(), 1);
        assert_eq!(result.walls[0].portals.len(), 1);
        assert_eq!(result.portals.len(), 0); // No freestanding portals.

        let portal = &result.walls[0].portals[0];
        assert_ne!(portal.wall_id, "ffffffff");
        // wall_distance = segment_index + fractional_position.
        // The exact segment index depends on loop chaining order,
        // but the fractional part should be 0.5.
        assert!((portal.wall_distance.fract() - 0.5).abs() < 0.1);
    }

    #[test]
    fn portal_on_shared_edge_freestanding() {
        // Two adjacent polygons sharing an edge. Portal on the shared edge
        // should become freestanding since the edge is removed.
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "left".to_string(),
                points: vec![[2.0, 2.0], [6.0, 2.0], [6.0, 5.0], [2.0, 5.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![PolygonPortalConfig {
                    edge: 1, // East wall (shared with right room).
                    position: 0.5,
                    portal_type: PortalType::Door,
                }],
            },
            PolygonConfig {
                id: "right".to_string(),
                points: vec![[6.0, 2.0], [10.0, 2.0], [10.0, 5.0], [6.0, 5.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        // One merged wall (shared edge removed).
        assert_eq!(result.walls.len(), 1);
        // The door on the shared edge should be freestanding.
        assert_eq!(result.portals.len(), 1);
        assert_eq!(result.portals[0].wall_id, "ffffffff");
        // No wall-anchored portals.
        assert_eq!(result.walls[0].portals.len(), 0);
    }

    #[test]
    fn portal_position_pixels() {
        // Verify portal pixel position is correct.
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![PolygonConfig {
            id: "room".to_string(),
            points: vec![[2.0, 2.0], [6.0, 2.0], [6.0, 5.0], [2.0, 5.0]],
            terrain_slot: None,
            wall_texture: default_wall_texture(),
            portals: vec![PolygonPortalConfig {
                edge: 0, // North wall: (2,2)→(6,2), midpoint = (4,2)
                position: 0.5,
                portal_type: PortalType::Door,
            }],
        }];

        let result = generate_polygon_layout(&polygons, &alloc);
        let portal = &result.walls[0].portals[0];

        // Midpoint of edge 0 in pixels: (4*256, 2*256) = (1024, 512).
        assert!((portal.position.x - 1024.0).abs() < 1.0);
        assert!((portal.position.y - 512.0).abs() < 1.0);
    }

    #[test]
    fn multiple_portals_on_different_edges() {
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![PolygonConfig {
            id: "room".to_string(),
            points: vec![[2.0, 2.0], [6.0, 2.0], [6.0, 5.0], [2.0, 5.0]],
            terrain_slot: None,
            wall_texture: default_wall_texture(),
            portals: vec![
                PolygonPortalConfig {
                    edge: 0, // North
                    position: 0.5,
                    portal_type: PortalType::Door,
                },
                PolygonPortalConfig {
                    edge: 2, // South
                    position: 0.5,
                    portal_type: PortalType::Window,
                },
            ],
        }];

        let result = generate_polygon_layout(&polygons, &alloc);
        assert_eq!(result.walls.len(), 1);
        assert_eq!(result.walls[0].portals.len(), 2);
        assert_eq!(result.portals.len(), 0);
    }

    #[test]
    fn three_rooms_corridor_portal_freestanding() {
        // Mirrors the test-three-rooms.yaml layout: rooms connected by corridors.
        // A portal on a corridor edge that's shared with a room → freestanding.
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "room_a".to_string(),
                points: vec![[3.0, 6.0], [9.0, 6.0], [9.0, 14.0], [3.0, 14.0]],
                terrain_slot: Some(3),
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "corridor_ab".to_string(),
                points: vec![[9.0, 10.0], [13.0, 10.0], [13.0, 11.0], [9.0, 11.0]],
                terrain_slot: Some(3),
                wall_texture: default_wall_texture(),
                portals: vec![PolygonPortalConfig {
                    edge: 3, // West wall of corridor (shared with room_a east).
                    position: 0.5,
                    portal_type: PortalType::Door,
                }],
            },
            PolygonConfig {
                id: "room_b".to_string(),
                points: vec![[13.0, 6.0], [19.0, 6.0], [19.0, 14.0], [13.0, 14.0]],
                terrain_slot: Some(3),
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        // Corridor's west edge is shared with room_a → portal is freestanding.
        assert_eq!(result.portals.len(), 1);
        assert_eq!(result.portals[0].wall_id, "ffffffff");
    }

    // ── Polygon union (overlapping polygons) ─────────────────────────

    #[test]
    fn overlapping_ovals_merge_to_one_wall() {
        // Two 8-sided "ovals" sharing vertices at (16,7) and (16,13).
        // Their overlap region should be merged into one outer boundary.
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "oval_left".to_string(),
                points: vec![
                    [8.0, 10.0], [10.0, 7.0], [13.0, 6.0], [16.0, 7.0],
                    [18.0, 10.0], [16.0, 13.0], [13.0, 14.0], [10.0, 13.0],
                ],
                terrain_slot: Some(3),
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "oval_right".to_string(),
                points: vec![
                    [14.0, 10.0], [16.0, 7.0], [19.0, 6.0], [22.0, 7.0],
                    [24.0, 10.0], [22.0, 13.0], [19.0, 14.0], [16.0, 13.0],
                ],
                terrain_slot: Some(3),
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        // Union: one merged wall with 12 vertices (outer boundary).
        // Inner diamond (4 interior edges) filtered out.
        assert_eq!(result.walls.len(), 1);
        assert!(result.walls[0].is_loop);
        assert_eq!(result.shape_wall_ids.len(), 1);
        assert_eq!(result.shape_polygons.len(), 1);

        // Verify 12 vertices in the merged boundary.
        assert_eq!(result.walls[0].points.0.len(), 12);
    }

    #[test]
    fn overlapping_ovals_with_portals() {
        // Portals on the outer perimeter of overlapping ovals → wall-anchored.
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "oval_left".to_string(),
                points: vec![
                    [8.0, 10.0], [10.0, 7.0], [13.0, 6.0], [16.0, 7.0],
                    [18.0, 10.0], [16.0, 13.0], [13.0, 14.0], [10.0, 13.0],
                ],
                terrain_slot: Some(3),
                wall_texture: default_wall_texture(),
                portals: vec![PolygonPortalConfig {
                    edge: 0, // West side: (8,10)→(10,7) — on outer perimeter.
                    position: 0.5,
                    portal_type: PortalType::Door,
                }],
            },
            PolygonConfig {
                id: "oval_right".to_string(),
                points: vec![
                    [14.0, 10.0], [16.0, 7.0], [19.0, 6.0], [22.0, 7.0],
                    [24.0, 10.0], [22.0, 13.0], [19.0, 14.0], [16.0, 13.0],
                ],
                terrain_slot: Some(3),
                wall_texture: default_wall_texture(),
                portals: vec![PolygonPortalConfig {
                    edge: 3, // East side: (22,7)→(24,10) — on outer perimeter.
                    position: 0.5,
                    portal_type: PortalType::Door,
                }],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        assert_eq!(result.walls.len(), 1);
        // Both portals on outer perimeter → wall-anchored.
        assert_eq!(result.walls[0].portals.len(), 2);
        assert_eq!(result.portals.len(), 0);
    }

    #[test]
    fn overlapping_ovals_interior_portal_freestanding() {
        // Portal on an interior edge (overlap boundary) → freestanding.
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "oval_left".to_string(),
                points: vec![
                    [8.0, 10.0], [10.0, 7.0], [13.0, 6.0], [16.0, 7.0],
                    [18.0, 10.0], [16.0, 13.0], [13.0, 14.0], [10.0, 13.0],
                ],
                terrain_slot: Some(3),
                wall_texture: default_wall_texture(),
                portals: vec![PolygonPortalConfig {
                    edge: 3, // (16,7)→(18,10) — interior edge (overlap).
                    position: 0.5,
                    portal_type: PortalType::Door,
                }],
            },
            PolygonConfig {
                id: "oval_right".to_string(),
                points: vec![
                    [14.0, 10.0], [16.0, 7.0], [19.0, 6.0], [22.0, 7.0],
                    [24.0, 10.0], [22.0, 13.0], [19.0, 14.0], [16.0, 13.0],
                ],
                terrain_slot: Some(3),
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        assert_eq!(result.walls.len(), 1);
        // Portal on interior edge → freestanding.
        assert_eq!(result.portals.len(), 1);
        assert_eq!(result.portals[0].wall_id, "ffffffff");
        assert_eq!(result.walls[0].portals.len(), 0);
    }

    // ── Butting-up adjacency tests ────────────────────────────────────
    //
    // These verify that shared walls between adjacent polygons are
    // properly removed, producing a single merged outer boundary.

    /// Helper: count total wall vertex points across all walls in the result.
    fn total_wall_points(result: &PolygonLayoutResult) -> usize {
        result.walls.iter().map(|w| w.points.0.len()).sum()
    }

    #[test]
    fn butting_side_by_side_horizontal() {
        // Two rectangles sharing a vertical edge:
        //  ┌───┬───┐
        //  │ A │ B │
        //  └───┴───┘
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "a".to_string(),
                points: vec![[0.0, 0.0], [5.0, 0.0], [5.0, 4.0], [0.0, 4.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "b".to_string(),
                points: vec![[5.0, 0.0], [10.0, 0.0], [10.0, 4.0], [5.0, 4.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        // Shared vertical edge at x=5 should be removed → one merged wall
        assert_eq!(result.walls.len(), 1, "Should produce 1 merged wall, got {}", result.walls.len());
        assert!(result.walls[0].is_loop, "Merged wall should be a closed loop");
        // Clean rectangle after collinear point collapse
        assert_eq!(total_wall_points(&result), 4,
            "Expected 4 vertices on merged boundary, got {}", total_wall_points(&result));
    }

    #[test]
    fn butting_stacked_vertical() {
        // Two rectangles sharing a horizontal edge:
        //  ┌───┐
        //  │ A │
        //  ├───┤
        //  │ B │
        //  └───┘
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "top".to_string(),
                points: vec![[2.0, 2.0], [8.0, 2.0], [8.0, 6.0], [2.0, 6.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "bottom".to_string(),
                points: vec![[2.0, 6.0], [8.0, 6.0], [8.0, 10.0], [2.0, 10.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        assert_eq!(result.walls.len(), 1, "Stacked rooms should merge to 1 wall, got {}", result.walls.len());
        assert!(result.walls[0].is_loop);
        assert_eq!(total_wall_points(&result), 4,
            "Expected 4 vertices after collinear collapse, got {}", total_wall_points(&result));
    }

    #[test]
    fn butting_three_in_a_row() {
        // Three rooms in a horizontal row, sharing two vertical edges:
        //  ┌───┬───┬───┐
        //  │ A │ B │ C │
        //  └───┴───┴───┘
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "a".to_string(),
                points: vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "b".to_string(),
                points: vec![[4.0, 0.0], [8.0, 0.0], [8.0, 4.0], [4.0, 4.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "c".to_string(),
                points: vec![[8.0, 0.0], [12.0, 0.0], [12.0, 4.0], [8.0, 4.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        assert_eq!(result.walls.len(), 1, "Three adjacent rooms should merge to 1 wall, got {}", result.walls.len());
        assert_eq!(total_wall_points(&result), 4,
            "Expected 4 vertices (clean rectangle), got {}", total_wall_points(&result));
    }

    #[test]
    fn butting_l_shaped_two_rects() {
        // Two rectangles forming an L-shape, sharing a partial edge:
        //  ┌─────┐
        //  │  A  │
        //  │  ┌──┘
        //  │  │B
        //  └──┘
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "top".to_string(),
                points: vec![[0.0, 0.0], [6.0, 0.0], [6.0, 4.0], [0.0, 4.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "leg".to_string(),
                points: vec![[0.0, 4.0], [3.0, 4.0], [3.0, 8.0], [0.0, 8.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        assert_eq!(result.walls.len(), 1, "L-shape should merge to 1 wall, got {}", result.walls.len());
        // L-shape boundary: [0,0]→[6,0]→[6,4]→[3,4]→[3,8]→[0,8]→back
        // 6 vertices — [0,4] collapsed because [0,8]→[0,4]→[0,0] is collinear (all x=0).
        assert_eq!(total_wall_points(&result), 6,
            "Expected 6 vertices for L-shape, got {}", total_wall_points(&result));
    }

    #[test]
    fn butting_t_junction() {
        // A corridor meets a room on one side (T-junction):
        //  ┌──────────┐
        //  │   room   │
        //  └──┬────┬──┘
        //     │corr│
        //     └────┘
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "room".to_string(),
                points: vec![[0.0, 0.0], [10.0, 0.0], [10.0, 5.0], [0.0, 5.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "corridor".to_string(),
                points: vec![[3.0, 5.0], [7.0, 5.0], [7.0, 9.0], [3.0, 9.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        assert_eq!(result.walls.len(), 1, "T-junction should merge to 1 wall, got {}", result.walls.len());
        // T-shape: 8 vertices
        assert_eq!(total_wall_points(&result), 8,
            "Expected 8 vertices for T-shape, got {}", total_wall_points(&result));
    }

    #[test]
    fn butting_four_rooms_grid() {
        // 2x2 grid of rooms sharing edges:
        //  ┌───┬───┐
        //  │ A │ B │
        //  ├───┼───┤
        //  │ C │ D │
        //  └───┴───┘
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "a".to_string(),
                points: vec![[0.0, 0.0], [5.0, 0.0], [5.0, 5.0], [0.0, 5.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "b".to_string(),
                points: vec![[5.0, 0.0], [10.0, 0.0], [10.0, 5.0], [5.0, 5.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "c".to_string(),
                points: vec![[0.0, 5.0], [5.0, 5.0], [5.0, 10.0], [0.0, 10.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "d".to_string(),
                points: vec![[5.0, 5.0], [10.0, 5.0], [10.0, 10.0], [5.0, 10.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        // All 4 interior edges removed → single outer rectangle
        assert_eq!(result.walls.len(), 1,
            "2x2 grid should merge to 1 wall, got {} walls", result.walls.len());
        // Clean rectangle: 4 corners, collinear midpoints collapsed
        assert_eq!(total_wall_points(&result), 4,
            "Expected 4 vertices for 2x2 grid outer rectangle, got {}", total_wall_points(&result));
    }

    #[test]
    fn butting_does_not_merge_disjoint_rooms() {
        // Two rooms with a gap between them — should NOT merge
        let alloc = NodeIdAllocator::new(1);
        let polygons = vec![
            PolygonConfig {
                id: "left".to_string(),
                points: vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "right".to_string(),
                points: vec![[6.0, 0.0], [10.0, 0.0], [10.0, 4.0], [6.0, 4.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ];

        let result = generate_polygon_layout(&polygons, &alloc);

        // No shared edge → two separate walls
        assert_eq!(result.walls.len(), 2,
            "Disjoint rooms should produce 2 walls, got {}", result.walls.len());
    }

    #[test]
    fn butting_shared_wall_has_no_double_segments() {
        // Verify at the segment level that shared edges are fully removed,
        // not just masked by the walk.
        let edges = collect_edges(&[
            PolygonConfig {
                id: "a".to_string(),
                points: vec![[0.0, 0.0], [5.0, 0.0], [5.0, 4.0], [0.0, 4.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
            PolygonConfig {
                id: "b".to_string(),
                points: vec![[5.0, 0.0], [10.0, 0.0], [10.0, 4.0], [5.0, 4.0]],
                terrain_slot: None,
                wall_texture: default_wall_texture(),
                portals: vec![],
            },
        ]);

        // 8 edges total (4 per polygon)
        assert_eq!(edges.len(), 8);

        let split = split_at_crossings(edges);
        let surviving = remove_shared_edges(split);

        // Shared edge: A's [5,0]→[5,4] and B's [5,4]→[5,0] should BOTH be removed
        // Leaving 6 edges (3 per polygon minus shared)
        assert_eq!(surviving.len(), 6,
            "Expected 6 surviving edges after removing shared edge, got {}.\nEdges: {:?}",
            surviving.len(),
            surviving.iter().map(|s| format!("[{:.0},{:.0}]→[{:.0},{:.0}]", s.start[0], s.start[1], s.end[0], s.end[1])).collect::<Vec<_>>());

        // Verify no segment has both endpoints on x=5*256=1280 (the shared edge line)
        let shared_x = 5.0 * PIXELS_PER_GRID;
        for seg in &surviving {
            let on_shared = (seg.start[0] - shared_x).abs() < 1.0
                && (seg.end[0] - shared_x).abs() < 1.0;
            assert!(!on_shared,
                "Found surviving segment on shared edge: [{:.0},{:.0}]→[{:.0},{:.0}]",
                seg.start[0], seg.start[1], seg.end[0], seg.end[1]);
        }
    }
}
