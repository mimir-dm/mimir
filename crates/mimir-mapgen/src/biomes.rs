//! Biome preset system.
//!
//! Built-in presets (forest, grassland, cave) providing sensible defaults
//! for terrain textures, object palettes, and noise parameters.

use crate::pipeline::MapConfig;

/// A biome preset with sensible defaults for all generation parameters.
#[derive(Debug, Clone)]
pub struct BiomePreset {
    /// Human-readable name.
    pub name: &'static str,
    /// Description.
    pub description: &'static str,
    /// Default map dimensions (grid squares).
    pub default_size: (u32, u32),
    /// Configuration to use as the base.
    pub config: MapConfig,
}

/// Get a biome preset by name.
pub fn get_preset(name: &str) -> Option<BiomePreset> {
    match name.to_lowercase().as_str() {
        "forest" => Some(forest_preset()),
        "grassland" => Some(grassland_preset()),
        "cave" => Some(cave_preset()),
        _ => None,
    }
}

/// List all available presets.
pub fn list_presets() -> Vec<BiomePreset> {
    vec![forest_preset(), grassland_preset(), cave_preset()]
}

fn forest_preset() -> BiomePreset {
    use crate::elevation::{ContourLevel, ElevationConfig, ShadowPathConfig};
    use crate::noise_gen::NoiseConfig;
    use crate::objects::{ObjectConfig, TreeConfig};
    use crate::paths::RoadConfig;
    use crate::terrain::{TerrainConfig, TerrainSlot};

    BiomePreset {
        name: "forest",
        description: "Dense temperate forest with dirt paths, scattered rocks, and natural clearings",
        default_size: (32, 32),
        config: MapConfig {
            name: "Forest Map".to_string(),
            width: 32,
            height: 32,
            seed: None,
            noise: NoiseConfig {
                seed: 0,
                octaves: 6,
                persistence: 0.5,
                lacunarity: 2.0,
                scale: 0.03,
            },
            island_mode: None,
            terrain: Some(TerrainConfig {
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
            }),
            trees: vec![TreeConfig {
                tree: ObjectConfig {
                    textures: vec![
                        "res://textures/objects/trees/tree_01.png".to_string(),
                        "res://textures/objects/trees/tree_02.png".to_string(),
                        "res://textures/objects/trees/tree_03.png".to_string(),
                    ],
                    min_distance: 180.0,
                    noise_lower: 0.3,
                    noise_upper: 0.8,
                    probability: 0.8,
                    scale_min: 0.8,
                    scale_max: 1.4,
                    layer: 300,
                    random_rotation: true,
                    random_mirror: true,
                    custom_color: None,
                },
                shadow: None,
                canopy: None,
            }],
            clutter: vec![ObjectConfig {
                textures: vec![
                    "res://textures/objects/grass/grass_01.png".to_string(),
                    "res://textures/objects/grass/grass_02.png".to_string(),
                ],
                min_distance: 80.0,
                noise_lower: 0.2,
                noise_upper: 0.7,
                probability: 0.6,
                scale_min: 0.5,
                scale_max: 1.0,
                layer: 100,
                random_rotation: true,
                random_mirror: false,
                custom_color: None,
            }],
            clumps: vec![],
            roads: vec![RoadConfig::default()],
            rivers: vec![],
            water: None,
            elevation: Some(ElevationConfig {
                levels: vec![ContourLevel {
                    threshold: 0.65,
                    texture: "res://textures/paths/path_rocks.png".to_string(),
                    width: 12.0,
                    layer: 100,
                    min_points: 8,
                    smooth_iterations: 2,
                    shadow: Some(ShadowPathConfig {
                        texture: "res://textures/paths/path_rocks.png".to_string(),
                        offset: 8.0,
                        width: 16.0,
                        layer: 50,
                    }),
                }],
                pixels_per_cell: 64.0,
            }),
            lighting: None,
        },
    }
}

fn grassland_preset() -> BiomePreset {
    use crate::noise_gen::NoiseConfig;
    use crate::objects::ObjectConfig;
    use crate::terrain::{TerrainConfig, TerrainSlot};

    BiomePreset {
        name: "grassland",
        description: "Open rolling hills with sparse trees and wildflowers",
        default_size: (32, 32),
        config: MapConfig {
            name: "Grassland Map".to_string(),
            width: 32,
            height: 32,
            seed: None,
            noise: NoiseConfig {
                seed: 0,
                octaves: 4,
                persistence: 0.4,
                lacunarity: 2.0,
                scale: 0.02,
            },
            island_mode: None,
            terrain: Some(TerrainConfig {
                slots: [
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_grass.png".to_string(),
                        lower: 0.0,
                        upper: 0.4,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_dry_grass.png".to_string(),
                        lower: 0.35,
                        upper: 0.65,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_dirt.png".to_string(),
                        lower: 0.6,
                        upper: 0.85,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_gravel.png".to_string(),
                        lower: 0.8,
                        upper: 1.0,
                    },
                ],
                blend_width: 0.08,
                smooth_blending: true,
            }),
            trees: vec![],
            clutter: vec![ObjectConfig {
                textures: vec![
                    "res://textures/objects/grass/grass_01.png".to_string(),
                    "res://textures/objects/grass/grass_02.png".to_string(),
                ],
                min_distance: 60.0,
                noise_lower: 0.1,
                noise_upper: 0.5,
                probability: 0.7,
                scale_min: 0.4,
                scale_max: 0.9,
                layer: 100,
                random_rotation: true,
                random_mirror: false,
                custom_color: None,
            }],
            clumps: vec![],
            roads: vec![],
            rivers: vec![],
            water: None,
            elevation: None,
            lighting: None,
        },
    }
}

fn cave_preset() -> BiomePreset {
    use crate::noise_gen::NoiseConfig;
    use crate::terrain::{TerrainConfig, TerrainSlot};
    use crate::pipeline::LightingConfig;

    BiomePreset {
        name: "cave",
        description: "Underground cavern with rocky terrain and dark ambient lighting",
        default_size: (24, 24),
        config: MapConfig {
            name: "Cave Map".to_string(),
            width: 24,
            height: 24,
            seed: None,
            noise: NoiseConfig {
                seed: 0,
                octaves: 5,
                persistence: 0.6,
                lacunarity: 2.0,
                scale: 0.04,
            },
            island_mode: None,
            terrain: Some(TerrainConfig {
                slots: [
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_stone.png".to_string(),
                        lower: 0.0,
                        upper: 0.4,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_gravel.png".to_string(),
                        lower: 0.35,
                        upper: 0.65,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_dirt.png".to_string(),
                        lower: 0.6,
                        upper: 0.85,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_stone.png".to_string(),
                        lower: 0.8,
                        upper: 1.0,
                    },
                ],
                blend_width: 0.06,
                smooth_blending: false,
            }),
            trees: vec![],
            clutter: vec![],
            clumps: vec![],
            roads: vec![],
            rivers: vec![],
            water: None,
            elevation: None,
            lighting: Some(LightingConfig {
                ambient_light: "ff333333".to_string(),
                ambient_energy: Some(0.3),
                shadow_color: Some("cc000000".to_string()),
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_preset() {
        assert!(get_preset("forest").is_some());
        assert!(get_preset("grassland").is_some());
        assert!(get_preset("cave").is_some());
        assert!(get_preset("Forest").is_some()); // Case insensitive
        assert!(get_preset("unknown").is_none());
    }

    #[test]
    fn test_list_presets() {
        let presets = list_presets();
        assert_eq!(presets.len(), 3);
        let names: Vec<_> = presets.iter().map(|p| p.name).collect();
        assert!(names.contains(&"forest"));
        assert!(names.contains(&"grassland"));
        assert!(names.contains(&"cave"));
    }

    #[test]
    fn test_forest_has_terrain() {
        let preset = get_preset("forest").unwrap();
        assert!(preset.config.terrain.is_some());
        assert!(!preset.config.trees.is_empty());
    }

    #[test]
    fn test_cave_has_lighting() {
        let preset = get_preset("cave").unwrap();
        assert!(preset.config.lighting.is_some());
        let lighting = preset.config.lighting.as_ref().unwrap();
        assert_eq!(lighting.ambient_light, "ff333333");
    }
}
