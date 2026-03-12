//! Noise generation wrapper around the `noise` crate.
//!
//! Provides configurable multi-octave Perlin noise with island/canyon modifiers.

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use serde::{Deserialize, Serialize};

/// Configuration for noise map generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NoiseConfig {
    /// Seed for deterministic generation.
    pub seed: u32,
    /// Number of fractal octaves (more = more detail, default 6).
    pub octaves: usize,
    /// How quickly amplitude decreases per octave (default 0.5).
    pub persistence: f64,
    /// How quickly frequency increases per octave (default 2.0).
    pub lacunarity: f64,
    /// Scale factor — larger values = more zoomed out (default 0.01).
    pub scale: f64,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            octaves: 6,
            persistence: 0.5,
            lacunarity: 2.0,
            scale: 0.01,
        }
    }
}

/// A 2D noise map with values normalized to 0.0–1.0.
#[derive(Debug, Clone)]
pub struct NoiseMap {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Vec<f64>>,
}

impl NoiseMap {
    /// Generate a noise map with the given dimensions and config.
    pub fn generate(width: usize, height: usize, config: &NoiseConfig) -> Self {
        let fbm = Fbm::<Perlin>::new(config.seed)
            .set_octaves(config.octaves)
            .set_persistence(config.persistence)
            .set_lacunarity(config.lacunarity);

        let mut data = vec![vec![0.0; width]; height];

        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 * config.scale;
                let ny = y as f64 * config.scale;
                // Fbm output is roughly in [-1, 1], normalize to [0, 1]
                let raw = fbm.get([nx, ny]);
                data[y][x] = (raw + 1.0) * 0.5;
            }
        }

        // Normalize to actual [0, 1] range
        let mut min_val = f64::MAX;
        let mut max_val = f64::MIN;
        for row in &data {
            for &val in row {
                min_val = min_val.min(val);
                max_val = max_val.max(val);
            }
        }

        let range = max_val - min_val;
        if range > f64::EPSILON {
            for row in &mut data {
                for val in row {
                    *val = (*val - min_val) / range;
                }
            }
        }

        Self {
            width,
            height,
            data,
        }
    }

    /// Get the value at (x, y), clamped to bounds.
    pub fn get(&self, x: usize, y: usize) -> f64 {
        let x = x.min(self.width.saturating_sub(1));
        let y = y.min(self.height.saturating_sub(1));
        self.data[y][x]
    }

    /// Get a bilinearly interpolated value at fractional coordinates.
    pub fn sample(&self, x: f64, y: f64) -> f64 {
        let x = x.clamp(0.0, (self.width - 1) as f64);
        let y = y.clamp(0.0, (self.height - 1) as f64);

        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = x - x0 as f64;
        let fy = y - y0 as f64;

        let v00 = self.data[y0][x0];
        let v10 = self.data[y0][x1];
        let v01 = self.data[y1][x0];
        let v11 = self.data[y1][x1];

        let top = v00 * (1.0 - fx) + v10 * fx;
        let bottom = v01 * (1.0 - fx) + v11 * fx;
        top * (1.0 - fy) + bottom * fy
    }

    /// Apply island mode: push values at edges toward 1.0 (high = water for terrain).
    /// This creates a landmass in the center with water around the edges.
    pub fn apply_island_mode(&mut self, falloff_strength: f64) {
        let cx = self.width as f64 / 2.0;
        let cy = self.height as f64 / 2.0;
        let max_dist = (cx * cx + cy * cy).sqrt();

        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let dist = (dx * dx + dy * dy).sqrt() / max_dist;
                // Quadratic falloff
                let falloff = (dist * falloff_strength).powi(2).min(1.0);
                self.data[y][x] = (self.data[y][x] + falloff).min(1.0);
            }
        }
    }

    /// Apply canyon mode: force values below each threshold down to create sharp level boundaries.
    pub fn apply_canyon_mode(&mut self, levels: &[f64]) {
        for row in &mut self.data {
            for val in row {
                for &threshold in levels {
                    if *val < threshold {
                        *val *= 0.7; // Push down values below threshold
                        break;
                    }
                }
            }
        }
    }

    /// Convert to a 0-255 byte map for compatibility with reference impl conventions.
    pub fn to_byte_map(&self) -> Vec<Vec<u8>> {
        self.data
            .iter()
            .map(|row| row.iter().map(|&v| (v * 255.0).round() as u8).collect())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_map_dimensions() {
        let config = NoiseConfig::default();
        let map = NoiseMap::generate(100, 80, &config);
        assert_eq!(map.width, 100);
        assert_eq!(map.height, 80);
        assert_eq!(map.data.len(), 80);
        assert_eq!(map.data[0].len(), 100);
    }

    #[test]
    fn test_noise_map_normalized() {
        let config = NoiseConfig::default();
        let map = NoiseMap::generate(50, 50, &config);

        let mut min_val = f64::MAX;
        let mut max_val = f64::MIN;
        for row in &map.data {
            for &val in row {
                min_val = min_val.min(val);
                max_val = max_val.max(val);
                assert!((0.0..=1.0).contains(&val), "Value out of range: {}", val);
            }
        }
        // Should span most of the range
        assert!(min_val < 0.1, "Min value too high: {}", min_val);
        assert!(max_val > 0.9, "Max value too low: {}", max_val);
    }

    #[test]
    fn test_noise_map_deterministic() {
        let config = NoiseConfig {
            seed: 42,
            ..Default::default()
        };
        let map1 = NoiseMap::generate(20, 20, &config);
        let map2 = NoiseMap::generate(20, 20, &config);
        assert_eq!(map1.data, map2.data);
    }

    #[test]
    fn test_noise_map_different_seeds() {
        let map1 = NoiseMap::generate(
            20,
            20,
            &NoiseConfig {
                seed: 1,
                ..Default::default()
            },
        );
        let map2 = NoiseMap::generate(
            20,
            20,
            &NoiseConfig {
                seed: 2,
                ..Default::default()
            },
        );
        assert_ne!(map1.data, map2.data);
    }

    #[test]
    fn test_bilinear_sample() {
        let config = NoiseConfig::default();
        let map = NoiseMap::generate(10, 10, &config);
        // Sample at integer coords should match get
        let val = map.sample(5.0, 5.0);
        assert!((val - map.get(5, 5)).abs() < 1e-10);
    }

    #[test]
    fn test_island_mode() {
        let config = NoiseConfig::default();
        let mut map = NoiseMap::generate(50, 50, &config);
        let edge_before = map.get(0, 0);

        map.apply_island_mode(1.5);

        // Edge values should be pushed up (toward water)
        let edge_after = map.get(0, 0);
        assert!(
            edge_after >= edge_before,
            "Edge should increase: {} -> {}",
            edge_before,
            edge_after
        );
    }

    #[test]
    fn test_byte_map() {
        let config = NoiseConfig::default();
        let map = NoiseMap::generate(10, 10, &config);
        let bytes = map.to_byte_map();
        assert_eq!(bytes.len(), 10);
        assert_eq!(bytes[0].len(), 10);
        for row in &bytes {
            for &b in row {
                let _ = b; // All u8 values are valid
            }
        }
    }
}
