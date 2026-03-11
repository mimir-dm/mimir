//! Spatial distribution algorithms.
//!
//! Poisson Disc sampling (Bridson's algorithm) with noise-gated variants.

use rand::Rng;
use std::f64::consts::PI;

use crate::noise_gen::NoiseMap;

/// A point in 2D space.
pub type Point = (f64, f64);

/// Poisson Disc sampler using Bridson's algorithm.
///
/// Generates a set of points where no two are closer than `min_distance`.
pub struct PoissonDisc;

impl PoissonDisc {
    /// Generate uniformly distributed points with minimum spacing.
    ///
    /// # Arguments
    /// * `width` - Area width
    /// * `height` - Area height
    /// * `min_distance` - Minimum distance between any two points
    /// * `rng` - Random number generator (for reproducibility)
    /// * `k` - Number of candidate attempts per active point (default 30)
    pub fn sample(
        width: f64,
        height: f64,
        min_distance: f64,
        rng: &mut impl Rng,
        k: u32,
    ) -> Vec<Point> {
        if min_distance <= 0.0 || width <= 0.0 || height <= 0.0 {
            return Vec::new();
        }

        let cell_size = min_distance / std::f64::consts::SQRT_2;
        let grid_w = (width / cell_size).ceil() as usize + 1;
        let grid_h = (height / cell_size).ceil() as usize + 1;

        let mut grid: Vec<Vec<Option<usize>>> = vec![vec![None; grid_w]; grid_h];
        let mut points: Vec<Point> = Vec::new();
        let mut active: Vec<usize> = Vec::new();

        // Start with a random initial point
        let initial = (rng.gen_range(0.0..width), rng.gen_range(0.0..height));
        let gi = (initial.0 / cell_size) as usize;
        let gj = (initial.1 / cell_size) as usize;
        grid[gj][gi] = Some(0);
        points.push(initial);
        active.push(0);

        while !active.is_empty() {
            let active_idx = rng.gen_range(0..active.len());
            let point_idx = active[active_idx];
            let (px, py) = points[point_idx];

            let mut found = false;
            for _ in 0..k {
                let angle = rng.gen_range(0.0..2.0 * PI);
                let dist = rng.gen_range(min_distance..2.0 * min_distance);
                let nx = px + angle.cos() * dist;
                let ny = py + angle.sin() * dist;

                if nx < 0.0 || nx >= width || ny < 0.0 || ny >= height {
                    continue;
                }

                let gi = (nx / cell_size) as usize;
                let gj = (ny / cell_size) as usize;

                if self::is_valid(&grid, &points, nx, ny, gi, gj, grid_w, grid_h, min_distance) {
                    let new_idx = points.len();
                    grid[gj][gi] = Some(new_idx);
                    points.push((nx, ny));
                    active.push(new_idx);
                    found = true;
                    break;
                }
            }

            if !found {
                active.swap_remove(active_idx);
            }
        }

        points
    }

    /// Generate noise-gated points — only places points where the noise value
    /// at that position falls within `[lower, upper]`.
    ///
    /// # Arguments
    /// * `noise_map` - 2D noise field to gate against
    /// * `lower` - Minimum noise value (0.0–1.0) to accept a point
    /// * `upper` - Maximum noise value (0.0–1.0) to accept a point
    /// * `min_distance` - Minimum spacing between points
    /// * `rng` - Random number generator
    /// * `probability` - Chance (0.0–1.0) of keeping each valid point
    pub fn sample_gated(
        noise_map: &NoiseMap,
        lower: f64,
        upper: f64,
        min_distance: f64,
        rng: &mut impl Rng,
        probability: f64,
    ) -> Vec<Point> {
        let width = noise_map.width as f64;
        let height = noise_map.height as f64;

        let all_points = Self::sample(width, height, min_distance, rng, 30);

        all_points
            .into_iter()
            .filter(|&(x, y)| {
                let noise_val = noise_map.sample(x, y);
                noise_val >= lower && noise_val <= upper && rng.gen::<f64>() < probability
            })
            .collect()
    }
}

fn is_valid(
    grid: &[Vec<Option<usize>>],
    points: &[Point],
    x: f64,
    y: f64,
    gi: usize,
    gj: usize,
    grid_w: usize,
    grid_h: usize,
    min_distance: f64,
) -> bool {
    let min_dist_sq = min_distance * min_distance;

    let search_radius = 2i32;
    for dy in -search_radius..=search_radius {
        for dx in -search_radius..=search_radius {
            let ni = gi as i32 + dx;
            let nj = gj as i32 + dy;
            if ni < 0 || ni >= grid_w as i32 || nj < 0 || nj >= grid_h as i32 {
                continue;
            }
            if let Some(idx) = grid[nj as usize][ni as usize] {
                let (px, py) = points[idx];
                let dx = x - px;
                let dy = y - py;
                if dx * dx + dy * dy < min_dist_sq {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_poisson_disc_basic() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let points = PoissonDisc::sample(100.0, 100.0, 10.0, &mut rng, 30);

        assert!(!points.is_empty());
        // All points within bounds
        for &(x, y) in &points {
            assert!(x >= 0.0 && x < 100.0);
            assert!(y >= 0.0 && y < 100.0);
        }
    }

    #[test]
    fn test_poisson_disc_min_distance() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let min_dist = 15.0;
        let points = PoissonDisc::sample(100.0, 100.0, min_dist, &mut rng, 30);

        // Check all pairs maintain minimum distance
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                let dx = points[i].0 - points[j].0;
                let dy = points[i].1 - points[j].1;
                let dist = (dx * dx + dy * dy).sqrt();
                assert!(
                    dist >= min_dist - 1e-10,
                    "Points too close: {} (min {})",
                    dist,
                    min_dist
                );
            }
        }
    }

    #[test]
    fn test_poisson_disc_deterministic() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(123);
        let mut rng2 = ChaCha8Rng::seed_from_u64(123);
        let p1 = PoissonDisc::sample(50.0, 50.0, 5.0, &mut rng1, 30);
        let p2 = PoissonDisc::sample(50.0, 50.0, 5.0, &mut rng2, 30);
        assert_eq!(p1.len(), p2.len());
        for (a, b) in p1.iter().zip(p2.iter()) {
            assert_eq!(a.0, b.0);
            assert_eq!(a.1, b.1);
        }
    }

    #[test]
    fn test_poisson_disc_gated() {
        use crate::noise_gen::{NoiseConfig, NoiseMap};

        let noise = NoiseMap::generate(
            100,
            100,
            &NoiseConfig {
                seed: 42,
                ..Default::default()
            },
        );
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let points = PoissonDisc::sample_gated(&noise, 0.3, 0.7, 5.0, &mut rng, 1.0);

        // Should have fewer points than ungated
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let all_points = PoissonDisc::sample(100.0, 100.0, 5.0, &mut rng2, 30);

        assert!(points.len() < all_points.len());
        // All gated points should be in noise range
        for &(x, y) in &points {
            let val = noise.sample(x, y);
            assert!(val >= 0.3 && val <= 0.7, "Noise value out of range: {}", val);
        }
    }

    #[test]
    fn test_empty_area() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let points = PoissonDisc::sample(0.0, 0.0, 10.0, &mut rng, 30);
        assert!(points.is_empty());
    }
}
