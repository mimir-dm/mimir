//! Terrain generation from noise maps.
//!
//! Maps noise values to 4-texture splat weights with boundary blending,
//! outputting DD-compatible terrain data.

use serde::{Deserialize, Serialize};

use crate::format::godot_types::PoolByteArray;
use crate::format::world::Terrain;
use crate::noise_gen::NoiseMap;

/// Configuration for a single terrain texture slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainSlot {
    /// Asset path, e.g. `"res://textures/terrain/terrain_dirt.png"`.
    pub texture: String,
    /// Lower noise boundary (0.0–1.0). Below this, weight is 0.
    pub lower: f64,
    /// Upper noise boundary (0.0–1.0). Above this, weight is 0.
    pub upper: f64,
}

/// Configuration for terrain generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainConfig {
    /// The 4 terrain texture slots with noise boundaries.
    pub slots: [TerrainSlot; 4],
    /// Width of the blend zone between slots (in noise units, e.g. 0.05).
    pub blend_width: f64,
    /// Whether to enable smooth blending in DD.
    pub smooth_blending: bool,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            slots: [
                TerrainSlot {
                    texture: "res://textures/terrain/terrain_dirt.png".to_string(),
                    lower: 0.0,
                    upper: 0.3,
                },
                TerrainSlot {
                    texture: "res://textures/terrain/terrain_dry_grass.png".to_string(),
                    lower: 0.25,
                    upper: 0.55,
                },
                TerrainSlot {
                    texture: "res://textures/terrain/terrain_moss.png".to_string(),
                    lower: 0.5,
                    upper: 0.8,
                },
                TerrainSlot {
                    texture: "res://textures/terrain/terrain_gravel.png".to_string(),
                    lower: 0.75,
                    upper: 1.0,
                },
            ],
            blend_width: 0.05,
            smooth_blending: false,
        }
    }
}

/// Generate terrain from a noise map.
///
/// The splat map has 4x4 cells per grid square. Each cell has 4 bytes (RGBA),
/// one weight per texture slot, summing to 255.
///
/// # Arguments
/// * `noise_map` - Source noise (any resolution — will be sampled)
/// * `width` - Map width in grid squares
/// * `height` - Map height in grid squares
/// * `config` - Terrain slot configuration
pub fn generate_terrain(
    noise_map: &NoiseMap,
    width: u32,
    height: u32,
    config: &TerrainConfig,
) -> Terrain {
    let cells_x = (width * 4) as usize;
    let cells_y = (height * 4) as usize;
    let total_cells = cells_x * cells_y;

    let mut splat_data = Vec::with_capacity(total_cells * 4);

    let noise_scale_x = noise_map.width as f64 / cells_x as f64;
    let noise_scale_y = noise_map.height as f64 / cells_y as f64;

    for cy in 0..cells_y {
        for cx in 0..cells_x {
            let nx = cx as f64 * noise_scale_x;
            let ny = cy as f64 * noise_scale_y;
            let noise_val = noise_map.sample(nx, ny);

            let weights = compute_weights(noise_val, &config.slots, config.blend_width);
            splat_data.extend_from_slice(&weights);
        }
    }

    Terrain {
        enabled: true,
        expand_slots: false,
        smooth_blending: config.smooth_blending,
        texture_1: config.slots[0].texture.clone(),
        texture_2: config.slots[1].texture.clone(),
        texture_3: config.slots[2].texture.clone(),
        texture_4: config.slots[3].texture.clone(),
        splat: PoolByteArray(splat_data),
    }
}

/// Modify terrain splat data along a road corridor.
///
/// Overrides terrain weights within the corridor to use a specific slot
/// (typically slot 0 for dirt/road texture).
///
/// # Arguments
/// * `terrain` - Mutable terrain to modify
/// * `width` - Map width in grid squares
/// * `corridor` - List of (x, y) points defining the road center (in pixel space, 256px/grid)
/// * `road_half_width` - Half-width of the road in pixels
/// * `road_slot` - Which texture slot (0-3) to use for the road
pub fn apply_road_corridor(
    terrain: &mut Terrain,
    width: u32,
    corridor: &[(f64, f64)],
    road_half_width: f64,
    road_slot: usize,
) {
    let cells_x = (width * 4) as usize;
    let px_per_cell = 256.0 / 4.0; // 64 pixels per cell

    let half_sq = road_half_width * road_half_width;

    for (i, byte) in terrain.splat.0.chunks_mut(4).enumerate() {
        let cx = (i % cells_x) as f64 * px_per_cell + px_per_cell / 2.0;
        let cy = (i / cells_x) as f64 * px_per_cell + px_per_cell / 2.0;

        // Check if this cell is within the road corridor
        let in_corridor = corridor.iter().any(|&(rx, ry)| {
            let dx = cx - rx;
            let dy = cy - ry;
            dx * dx + dy * dy <= half_sq
        });

        if in_corridor {
            let slot = road_slot.min(3);
            byte[0] = if slot == 0 { 255 } else { 0 };
            byte[1] = if slot == 1 { 255 } else { 0 };
            byte[2] = if slot == 2 { 255 } else { 0 };
            byte[3] = if slot == 3 { 255 } else { 0 };
        }
    }
}

/// Compute normalized RGBA weights for a noise value across 4 slots.
fn compute_weights(noise_val: f64, slots: &[TerrainSlot; 4], blend_width: f64) -> [u8; 4] {
    let mut raw_weights = [0.0f64; 4];

    for (i, slot) in slots.iter().enumerate() {
        raw_weights[i] = slot_weight(noise_val, slot.lower, slot.upper, blend_width);
    }

    // Normalize to sum to 255
    let total: f64 = raw_weights.iter().sum();
    if total < f64::EPSILON {
        // Fallback: assign to first slot
        return [255, 0, 0, 0];
    }

    let scale = 255.0 / total;
    let mut result = [0u8; 4];
    let mut assigned: u16 = 0;

    // Use floor for first 3 slots, give remainder to the largest slot
    for i in 0..4 {
        result[i] = (raw_weights[i] * scale).floor() as u8;
        assigned += result[i] as u16;
    }

    // Distribute remaining weight (due to floor rounding) to the slot
    // with the largest fractional part
    let remainder = 255u16.saturating_sub(assigned) as u8;
    if remainder > 0 {
        let max_frac_idx = (0..4)
            .max_by(|&a, &b| {
                let fa = (raw_weights[a] * scale).fract();
                let fb = (raw_weights[b] * scale).fract();
                fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(0);
        result[max_frac_idx] = result[max_frac_idx].saturating_add(remainder);
    }

    result
}

/// Compute the raw weight for a single slot based on noise value.
fn slot_weight(noise_val: f64, lower: f64, upper: f64, blend_width: f64) -> f64 {
    if noise_val >= lower + blend_width && noise_val <= upper - blend_width {
        // Fully inside the slot
        1.0
    } else if noise_val >= lower - blend_width && noise_val < lower + blend_width {
        // Blending in from below
        let t = (noise_val - (lower - blend_width)) / (2.0 * blend_width);
        t.clamp(0.0, 1.0)
    } else if noise_val > upper - blend_width && noise_val <= upper + blend_width {
        // Blending out toward above
        let t = ((upper + blend_width) - noise_val) / (2.0 * blend_width);
        t.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noise_gen::{NoiseConfig, NoiseMap};

    #[test]
    fn test_generate_terrain_dimensions() {
        let noise = NoiseMap::generate(100, 100, &NoiseConfig::default());
        let config = TerrainConfig::default();
        let terrain = generate_terrain(&noise, 10, 10, &config);

        // 10*4 * 10*4 = 1600 cells * 4 bytes = 6400
        assert_eq!(terrain.splat.0.len(), 6400);
        assert!(terrain.enabled);
    }

    #[test]
    fn test_splat_weights_sum_to_255() {
        let noise = NoiseMap::generate(
            50,
            50,
            &NoiseConfig {
                seed: 42,
                ..Default::default()
            },
        );
        let config = TerrainConfig::default();
        let terrain = generate_terrain(&noise, 10, 10, &config);

        for chunk in terrain.splat.0.chunks(4) {
            let sum: u16 = chunk.iter().map(|&b| b as u16).sum();
            assert_eq!(sum, 255, "Weights must sum to 255, got {}", sum);
        }
    }

    #[test]
    fn test_compute_weights_pure_slot() {
        let slots = [
            TerrainSlot {
                texture: "a".into(),
                lower: 0.0,
                upper: 0.25,
            },
            TerrainSlot {
                texture: "b".into(),
                lower: 0.25,
                upper: 0.5,
            },
            TerrainSlot {
                texture: "c".into(),
                lower: 0.5,
                upper: 0.75,
            },
            TerrainSlot {
                texture: "d".into(),
                lower: 0.75,
                upper: 1.0,
            },
        ];

        // Value firmly in slot 0
        let w = compute_weights(0.1, &slots, 0.02);
        assert_eq!(w[0], 255);
        assert_eq!(w[1], 0);

        // Value firmly in slot 2
        let w = compute_weights(0.6, &slots, 0.02);
        assert_eq!(w[2], 255);
    }

    #[test]
    fn test_compute_weights_blend_zone() {
        let slots = [
            TerrainSlot {
                texture: "a".into(),
                lower: 0.0,
                upper: 0.5,
            },
            TerrainSlot {
                texture: "b".into(),
                lower: 0.5,
                upper: 1.0,
            },
            TerrainSlot {
                texture: "c".into(),
                lower: 2.0,
                upper: 3.0,
            },
            TerrainSlot {
                texture: "d".into(),
                lower: 2.0,
                upper: 3.0,
            },
        ];

        // At the boundary between slot 0 and 1, both should have weight
        let w = compute_weights(0.5, &slots, 0.1);
        assert!(w[0] > 0, "Slot 0 should have weight at boundary");
        assert!(w[1] > 0, "Slot 1 should have weight at boundary");
        let sum: u16 = w.iter().map(|&b| b as u16).sum();
        assert_eq!(sum, 255);
    }

    #[test]
    fn test_road_corridor() {
        let noise = NoiseMap::generate(50, 50, &NoiseConfig::default());
        let config = TerrainConfig::default();
        let mut terrain = generate_terrain(&noise, 5, 5, &config);

        // Place a road point at center of the map
        let center = (5.0 * 256.0 / 2.0, 5.0 * 256.0 / 2.0);
        apply_road_corridor(&mut terrain, 5, &[center], 100.0, 0);

        // Check that the center cell is now 100% slot 0
        // At least some cells near center should be road
        let mut road_cells = 0;
        for chunk in terrain.splat.0.chunks(4) {
            if chunk[0] == 255 && chunk[1] == 0 && chunk[2] == 0 && chunk[3] == 0 {
                road_cells += 1;
            }
        }
        assert!(road_cells > 0, "Should have some road cells");
    }

    #[test]
    fn test_terrain_textures() {
        let noise = NoiseMap::generate(50, 50, &NoiseConfig::default());
        let config = TerrainConfig::default();
        let terrain = generate_terrain(&noise, 10, 10, &config);

        assert_eq!(terrain.texture_1, "res://textures/terrain/terrain_dirt.png");
        assert_eq!(
            terrain.texture_2,
            "res://textures/terrain/terrain_dry_grass.png"
        );
        assert_eq!(terrain.texture_3, "res://textures/terrain/terrain_moss.png");
        assert_eq!(
            terrain.texture_4,
            "res://textures/terrain/terrain_gravel.png"
        );
    }
}
