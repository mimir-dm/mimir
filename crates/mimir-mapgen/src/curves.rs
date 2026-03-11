//! Curve smoothing algorithms.
//!
//! Cubic Bezier evaluation for smoothing polylines into natural curves.

/// Evaluate a cubic Bezier curve at parameter `t` (0.0–1.0).
///
/// Given 4 control points (p0, p1, p2, p3), returns the point on the curve at `t`.
pub fn cubic_bezier(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    t: f64,
) -> (f64, f64) {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    (
        mt3 * p0.0 + 3.0 * mt2 * t * p1.0 + 3.0 * mt * t2 * p2.0 + t3 * p3.0,
        mt3 * p0.1 + 3.0 * mt2 * t * p1.1 + 3.0 * mt * t2 * p2.1 + t3 * p3.1,
    )
}

/// Smooth a polyline using cubic Bezier interpolation.
///
/// Converts a sequence of waypoints into a smooth curve by generating
/// control points and evaluating the Bezier at `density` intermediate
/// points per segment.
///
/// # Arguments
/// * `points` - Input waypoints
/// * `density` - Number of interpolated points per segment (higher = smoother)
pub fn bezier_smooth(points: &[(f64, f64)], density: usize) -> Vec<(f64, f64)> {
    if points.len() < 2 {
        return points.to_vec();
    }

    if points.len() == 2 {
        return points.to_vec();
    }

    let density = density.max(1);
    let mut result = Vec::with_capacity(points.len() * density);

    for i in 0..(points.len() - 1) {
        let p0 = points[i];
        let p3 = points[i + 1];

        // Generate control points using Catmull-Rom style
        let p_prev = if i > 0 { points[i - 1] } else { p0 };
        let p_next = if i + 2 < points.len() {
            points[i + 2]
        } else {
            p3
        };

        // Control points at 1/3 intervals, influenced by neighbors
        let tension = 0.3;
        let p1 = (
            p0.0 + (p3.0 - p_prev.0) * tension,
            p0.1 + (p3.1 - p_prev.1) * tension,
        );
        let p2 = (
            p3.0 - (p_next.0 - p0.0) * tension,
            p3.1 - (p_next.1 - p0.1) * tension,
        );

        // Evaluate the Bezier curve
        for step in 0..density {
            let t = step as f64 / density as f64;
            result.push(cubic_bezier(p0, p1, p2, p3, t));
        }
    }

    // Add the last point
    result.push(*points.last().unwrap());

    result
}

/// Compute the total length of a polyline.
pub fn polyline_length(points: &[(f64, f64)]) -> f64 {
    points
        .windows(2)
        .map(|w| {
            let dx = w[1].0 - w[0].0;
            let dy = w[1].1 - w[0].1;
            (dx * dx + dy * dy).sqrt()
        })
        .sum()
}

/// Compute the perpendicular offset of a polyline.
///
/// Returns a new polyline offset by `distance` to the left (positive)
/// or right (negative) of the original.
pub fn offset_polyline(points: &[(f64, f64)], distance: f64) -> Vec<(f64, f64)> {
    if points.len() < 2 {
        return points.to_vec();
    }

    let mut result = Vec::with_capacity(points.len());

    for i in 0..points.len() {
        let (nx, ny) = if i == 0 {
            segment_normal(points[0], points[1])
        } else if i == points.len() - 1 {
            segment_normal(points[i - 1], points[i])
        } else {
            // Average normals of adjacent segments
            let n1 = segment_normal(points[i - 1], points[i]);
            let n2 = segment_normal(points[i], points[i + 1]);
            let avg = ((n1.0 + n2.0) / 2.0, (n1.1 + n2.1) / 2.0);
            let len = (avg.0 * avg.0 + avg.1 * avg.1).sqrt();
            if len > f64::EPSILON {
                (avg.0 / len, avg.1 / len)
            } else {
                n1
            }
        };

        result.push((points[i].0 + nx * distance, points[i].1 + ny * distance));
    }

    result
}

/// Compute the unit normal (perpendicular) of a line segment.
fn segment_normal(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < f64::EPSILON {
        (0.0, 0.0)
    } else {
        (-dy / len, dx / len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubic_bezier_endpoints() {
        let p0 = (0.0, 0.0);
        let p1 = (1.0, 2.0);
        let p2 = (3.0, 2.0);
        let p3 = (4.0, 0.0);

        let start = cubic_bezier(p0, p1, p2, p3, 0.0);
        assert!((start.0 - p0.0).abs() < 1e-10);
        assert!((start.1 - p0.1).abs() < 1e-10);

        let end = cubic_bezier(p0, p1, p2, p3, 1.0);
        assert!((end.0 - p3.0).abs() < 1e-10);
        assert!((end.1 - p3.1).abs() < 1e-10);
    }

    #[test]
    fn test_bezier_smooth_preserves_endpoints() {
        let points = vec![(0.0, 0.0), (5.0, 10.0), (10.0, 5.0), (15.0, 0.0)];
        let smoothed = bezier_smooth(&points, 10);

        assert_eq!(smoothed[0], points[0]);
        assert_eq!(*smoothed.last().unwrap(), *points.last().unwrap());
        assert!(smoothed.len() > points.len());
    }

    #[test]
    fn test_bezier_smooth_two_points() {
        let points = vec![(0.0, 0.0), (10.0, 10.0)];
        let smoothed = bezier_smooth(&points, 10);
        assert_eq!(smoothed.len(), 2);
    }

    #[test]
    fn test_polyline_length() {
        let points = vec![(0.0, 0.0), (3.0, 0.0), (3.0, 4.0)];
        let length = polyline_length(&points);
        assert!((length - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_offset_polyline() {
        // A horizontal line: normal is (-dy, dx) = (0, 1) → positive y direction
        let points = vec![(0.0, 0.0), (10.0, 0.0)];
        let offset = offset_polyline(&points, 5.0);
        assert_eq!(offset.len(), 2);
        assert!((offset[0].1 - 5.0).abs() < 1e-10);
        assert!((offset[1].1 - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_segment_normal() {
        let n = segment_normal((0.0, 0.0), (10.0, 0.0));
        // Normal to rightward segment: (-dy/len, dx/len) = (0, 1)
        assert!((n.0).abs() < 1e-10);
        assert!((n.1 - 1.0).abs() < 1e-10);
    }
}
