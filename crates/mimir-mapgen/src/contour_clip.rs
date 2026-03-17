//! Contour clipping against road/river corridors.
//!
//! After contours and roads/rivers are both generated, this module clips
//! contour polylines to remove segments that fall within corridor bounds.

use crate::format::entities::MapPath;
use crate::format::godot_types::Vector2;
use crate::format::NodeIdAllocator;

/// A corridor defined by a centerline polyline and half-width.
pub struct Corridor {
    pub points: Vec<(f64, f64)>,
    pub half_width: f64,
}

/// Clip a set of MapPath contour lines against corridors.
///
/// For each contour, removes segments that fall within any corridor's bounds.
/// Returns new MapPath entries with the surviving segments.
pub fn clip_contours_against_corridors(
    contours: Vec<MapPath>,
    corridors: &[Corridor],
    alloc: &NodeIdAllocator,
) -> Vec<MapPath> {
    if corridors.is_empty() {
        return contours;
    }

    let mut result = Vec::new();

    for contour in contours {
        // Convert edit_points to absolute coordinates
        let abs_points: Vec<(f64, f64)> = contour.edit_points.0.iter()
            .map(|p| (contour.position.x + p.x, contour.position.y + p.y))
            .collect();

        if abs_points.len() < 2 {
            continue;
        }

        // Walk the polyline, tracking whether we're inside a corridor
        let segments = split_polyline_by_corridors(&abs_points, corridors);

        for segment in segments {
            if segment.len() >= 2 {
                let vectors: Vec<Vector2> = segment.iter()
                    .map(|&(x, y)| Vector2::new(x, y))
                    .collect();
                let path = MapPath::new(&contour.texture, vectors, contour.width, &alloc.next())
                    .with_layer(contour.layer);
                result.push(path);
            }
        }
    }

    result
}

/// Split a polyline into segments that are outside all corridors.
fn split_polyline_by_corridors(
    points: &[(f64, f64)],
    corridors: &[Corridor],
) -> Vec<Vec<(f64, f64)>> {
    let mut segments: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut current_segment: Vec<(f64, f64)> = Vec::new();

    for &point in points {
        if is_inside_any_corridor(point, corridors) {
            // Point is inside a corridor — end current segment
            if current_segment.len() >= 2 {
                segments.push(std::mem::take(&mut current_segment));
            } else {
                current_segment.clear();
            }
        } else {
            // Point is outside all corridors — add to current segment
            current_segment.push(point);
        }
    }

    // Flush final segment
    if current_segment.len() >= 2 {
        segments.push(current_segment);
    }

    segments
}

/// Check if a point is inside any corridor.
fn is_inside_any_corridor(point: (f64, f64), corridors: &[Corridor]) -> bool {
    for corridor in corridors {
        if distance_to_polyline(point.0, point.1, &corridor.points) <= corridor.half_width {
            return true;
        }
    }
    false
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

    #[test]
    fn test_no_corridors_returns_unchanged() {
        let alloc = NodeIdAllocator::new(1);
        let path = MapPath::new("cliff.png", vec![
            Vector2::new(100.0, 100.0),
            Vector2::new(500.0, 100.0),
            Vector2::new(900.0, 100.0),
        ], 20.0, &alloc.next());
        let result = clip_contours_against_corridors(vec![path], &[], &alloc);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_corridor_clips_middle() {
        let alloc = NodeIdAllocator::new(1);
        // Contour runs horizontally from 0 to 1000
        let points: Vec<Vector2> = (0..=10).map(|i| Vector2::new(i as f64 * 100.0, 500.0)).collect();
        let path = MapPath::new("cliff.png", points, 20.0, &alloc.next());

        // Corridor runs vertically through the middle at x=500
        let corridor = Corridor {
            points: vec![(500.0, 0.0), (500.0, 1000.0)],
            half_width: 100.0,
        };

        let result = clip_contours_against_corridors(vec![path], &[corridor], &alloc);
        // Should produce 2 segments: left of corridor and right of corridor
        assert!(result.len() >= 2, "Expected 2+ segments, got {}", result.len());
    }

    #[test]
    fn test_corridor_fully_covers_contour() {
        let alloc = NodeIdAllocator::new(1);
        // Short contour at y=500
        let path = MapPath::new("cliff.png", vec![
            Vector2::new(480.0, 500.0),
            Vector2::new(520.0, 500.0),
        ], 20.0, &alloc.next());

        // Wide corridor at y=500
        let corridor = Corridor {
            points: vec![(0.0, 500.0), (1000.0, 500.0)],
            half_width: 200.0,
        };

        let result = clip_contours_against_corridors(vec![path], &[corridor], &alloc);
        assert!(result.is_empty(), "Fully covered contour should be removed");
    }

    #[test]
    fn test_distance_to_polyline() {
        let line = vec![(0.0, 0.0), (100.0, 0.0)];
        assert!((distance_to_polyline(50.0, 10.0, &line) - 10.0).abs() < 0.01);
        assert!((distance_to_polyline(50.0, 0.0, &line) - 0.0).abs() < 0.01);
        assert!((distance_to_polyline(150.0, 0.0, &line) - 50.0).abs() < 0.01);
    }
}
