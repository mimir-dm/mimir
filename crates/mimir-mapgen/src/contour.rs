//! Contour extraction via Marching Squares.
//!
//! Extracts contour polylines from 2D scalar fields at configurable thresholds.

use crate::noise_gen::NoiseMap;

/// A contour polyline — a sequence of 2D points.
pub type Contour = Vec<(f64, f64)>;

/// Extract contour lines from a noise map at the given threshold.
///
/// Uses the Marching Squares algorithm to find edges where the noise
/// field crosses the threshold value.
pub fn find_contours(noise_map: &NoiseMap, threshold: f64) -> Vec<Contour> {
    let h = noise_map.height;
    let w = noise_map.width;

    if w < 2 || h < 2 {
        return Vec::new();
    }

    // Track which cell edges have been visited
    let mut visited = vec![vec![false; w - 1]; h - 1];
    let mut contours = Vec::new();

    for y in 0..(h - 1) {
        for x in 0..(w - 1) {
            if visited[y][x] {
                continue;
            }

            let case = cell_case(noise_map, x, y, threshold);
            if case == 0 || case == 15 {
                // Fully inside or outside — no contour
                visited[y][x] = true;
                continue;
            }

            // Start tracing a contour from this cell
            if let Some(contour) = trace_contour(noise_map, &mut visited, x, y, threshold, w, h) {
                if contour.len() >= 2 {
                    contours.push(contour);
                }
            }
        }
    }

    contours
}

/// Filter contours by minimum length (number of points).
pub fn filter_contours(contours: Vec<Contour>, min_points: usize) -> Vec<Contour> {
    contours
        .into_iter()
        .filter(|c| c.len() >= min_points)
        .collect()
}

/// Smooth contours by averaging each point with its neighbors.
pub fn smooth_contours(contours: Vec<Contour>, iterations: usize) -> Vec<Contour> {
    contours
        .into_iter()
        .map(|c| smooth_polyline(&c, iterations))
        .collect()
}

fn smooth_polyline(points: &[(f64, f64)], iterations: usize) -> Contour {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut result = points.to_vec();
    for _ in 0..iterations {
        let mut smoothed = vec![(0.0, 0.0); result.len()];
        smoothed[0] = result[0];
        smoothed[result.len() - 1] = result[result.len() - 1];
        for i in 1..(result.len() - 1) {
            smoothed[i] = (
                (result[i - 1].0 + result[i].0 + result[i + 1].0) / 3.0,
                (result[i - 1].1 + result[i].1 + result[i + 1].1) / 3.0,
            );
        }
        result = smoothed;
    }
    result
}

/// Compute the Marching Squares case index (0-15) for a cell.
fn cell_case(noise_map: &NoiseMap, x: usize, y: usize, threshold: f64) -> u8 {
    let tl = if noise_map.get(x, y) >= threshold {
        1
    } else {
        0
    };
    let tr = if noise_map.get(x + 1, y) >= threshold {
        1
    } else {
        0
    };
    let br = if noise_map.get(x + 1, y + 1) >= threshold {
        1
    } else {
        0
    };
    let bl = if noise_map.get(x, y + 1) >= threshold {
        1
    } else {
        0
    };
    (tl << 3) | (tr << 2) | (br << 1) | bl
}

/// Interpolate position along an edge between two values crossing the threshold.
fn interpolate(v1: f64, v2: f64, threshold: f64) -> f64 {
    if (v2 - v1).abs() < f64::EPSILON {
        0.5
    } else {
        ((threshold - v1) / (v2 - v1)).clamp(0.0, 1.0)
    }
}

/// Get the edge crossing point(s) for a given marching squares case.
/// Returns pairs of (entry_point, exit_point) as edge segments.
fn cell_segments(
    noise_map: &NoiseMap,
    x: usize,
    y: usize,
    threshold: f64,
) -> Vec<((f64, f64), (f64, f64))> {
    let case = cell_case(noise_map, x, y, threshold);

    let tl = noise_map.get(x, y);
    let tr = noise_map.get(x + 1, y);
    let br = noise_map.get(x + 1, y + 1);
    let bl = noise_map.get(x, y + 1);

    let xf = x as f64;
    let yf = y as f64;

    // Edge midpoints with interpolation
    let top = (xf + interpolate(tl, tr, threshold), yf);
    let right = (xf + 1.0, yf + interpolate(tr, br, threshold));
    let bottom = (xf + interpolate(bl, br, threshold), yf + 1.0);
    let left = (xf, yf + interpolate(tl, bl, threshold));

    match case {
        0 | 15 => vec![],
        1 => vec![(left, bottom)],
        2 => vec![(bottom, right)],
        3 => vec![(left, right)],
        4 => vec![(right, top)],
        5 => vec![(left, top), (bottom, right)], // Saddle
        6 => vec![(bottom, top)],
        7 => vec![(left, top)],
        8 => vec![(top, left)],
        9 => vec![(top, bottom)],
        10 => vec![(top, right), (left, bottom)], // Saddle
        11 => vec![(top, right)],
        12 => vec![(right, left)],
        13 => vec![(right, bottom)],
        14 => vec![(bottom, left)],
        _ => vec![],
    }
}

/// Trace a contour starting from the given cell.
fn trace_contour(
    noise_map: &NoiseMap,
    visited: &mut [Vec<bool>],
    start_x: usize,
    start_y: usize,
    threshold: f64,
    w: usize,
    h: usize,
) -> Option<Contour> {
    let mut contour = Vec::new();
    let mut cx = start_x;
    let mut cy = start_y;

    let max_steps = w * h; // Prevent infinite loops
    for _ in 0..max_steps {
        if cx >= w - 1 || cy >= h - 1 {
            break;
        }
        if visited[cy][cx] {
            break;
        }

        let segments = cell_segments(noise_map, cx, cy, threshold);
        if segments.is_empty() {
            visited[cy][cx] = true;
            break;
        }

        visited[cy][cx] = true;

        // Take the first segment
        let (start, end) = segments[0];
        if contour.is_empty() {
            contour.push(start);
        }
        contour.push(end);

        // Determine next cell based on which edge the contour exits through
        let next = next_cell(cx, cy, end, w, h);
        match next {
            Some((nx, ny)) => {
                cx = nx;
                cy = ny;
            }
            None => break,
        }
    }

    if contour.len() >= 2 {
        Some(contour)
    } else {
        None
    }
}

/// Determine the next cell to visit based on the exit point.
fn next_cell(
    cx: usize,
    cy: usize,
    exit: (f64, f64),
    w: usize,
    h: usize,
) -> Option<(usize, usize)> {
    let xf = cx as f64;
    let yf = cy as f64;

    // Check which edge the exit point is on
    if (exit.1 - yf).abs() < f64::EPSILON && cy > 0 {
        // Top edge → go up
        Some((cx, cy - 1))
    } else if (exit.1 - (yf + 1.0)).abs() < f64::EPSILON && cy + 1 < h - 1 {
        // Bottom edge → go down
        Some((cx, cy + 1))
    } else if (exit.0 - xf).abs() < f64::EPSILON && cx > 0 {
        // Left edge → go left
        Some((cx - 1, cy))
    } else if (exit.0 - (xf + 1.0)).abs() < f64::EPSILON && cx + 1 < w - 1 {
        // Right edge → go right
        Some((cx + 1, cy))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noise_gen::{NoiseConfig, NoiseMap};

    #[test]
    fn test_find_contours_basic() {
        let noise = NoiseMap::generate(
            50,
            50,
            &NoiseConfig {
                seed: 42,
                ..Default::default()
            },
        );
        let contours = find_contours(&noise, 0.5);
        assert!(!contours.is_empty(), "Should find contours at threshold 0.5");
        for contour in &contours {
            assert!(contour.len() >= 2, "Contour should have at least 2 points");
        }
    }

    #[test]
    fn test_no_contours_at_extremes() {
        // A uniform map should have no contours
        let noise = NoiseMap {
            width: 10,
            height: 10,
            data: vec![vec![0.5; 10]; 10],
        };
        let contours = find_contours(&noise, 0.3);
        assert!(contours.is_empty(), "Uniform map should have no contours");
    }

    #[test]
    fn test_filter_contours() {
        let contours = vec![
            vec![(0.0, 0.0), (1.0, 1.0)],
            vec![(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0), (4.0, 4.0)],
        ];
        let filtered = filter_contours(contours, 3);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].len(), 5);
    }

    #[test]
    fn test_smooth_contours() {
        let contours = vec![vec![
            (0.0, 0.0),
            (1.0, 2.0),
            (2.0, 0.0),
            (3.0, 2.0),
            (4.0, 0.0),
        ]];
        let smoothed = smooth_contours(contours, 1);
        assert_eq!(smoothed.len(), 1);
        assert_eq!(smoothed[0].len(), 5);
        // Endpoints preserved
        assert_eq!(smoothed[0][0], (0.0, 0.0));
        assert_eq!(smoothed[0][4], (4.0, 0.0));
        // Middle points should be smoothed (closer to average)
        assert!((smoothed[0][2].1 - 0.0).abs() > 0.1); // No longer at 0, averaged with neighbors
    }

    #[test]
    fn test_cell_case() {
        // Create a map with a clear boundary
        let data = vec![
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0, 1.0],
        ];
        let noise = NoiseMap {
            width: 4,
            height: 4,
            data,
        };

        // Cell (0,0): all corners below 0.5 → case 0
        assert_eq!(cell_case(&noise, 0, 0, 0.5), 0);
        // Cell (2,0): all corners above 0.5 → case 15
        assert_eq!(cell_case(&noise, 2, 0, 0.5), 15);
        // Cell (1,0): right side above, left below → mixed case
        let case = cell_case(&noise, 1, 0, 0.5);
        assert!(case != 0 && case != 15);
    }
}
