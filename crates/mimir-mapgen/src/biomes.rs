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
        "desert" => Some(desert_preset()),
        "lake" | "pond" => Some(lake_preset()),
        "ice_lake" | "ice-lake" | "icelake" => Some(ice_lake_preset()),
        "arctic" => Some(arctic_preset()),
        "island_tropical" | "island-tropical" | "tropical_island" | "tropical-island" => {
            Some(island_tropical_preset())
        }
        "island_forest" | "island-forest" | "forest_island" | "forest-island" => {
            Some(island_forest_preset())
        }
        "island_arctic" | "island-arctic" | "arctic_island" | "arctic-island" => {
            Some(island_arctic_preset())
        }
        _ => None,
    }
}

/// List all available presets.
pub fn list_presets() -> Vec<BiomePreset> {
    vec![
        forest_preset(),
        grassland_preset(),
        cave_preset(),
        desert_preset(),
        lake_preset(),
        ice_lake_preset(),
        arctic_preset(),
        island_tropical_preset(),
        island_forest_preset(),
        island_arctic_preset(),
    ]
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
                        "res://textures/objects/more_trees/oak_01.png".to_string(),
                        "res://textures/objects/more_trees/oak_02.png".to_string(),
                        "res://textures/objects/more_trees/oak_03.png".to_string(),
                    ],
                    min_distance: 500.0,
                    noise_lower: 0.4,
                    noise_upper: 0.75,
                    probability: 0.5,
                    scale_min: 0.8,
                    scale_max: 1.2,
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
                    "res://textures/objects/vegetation/grass/grass_01.png".to_string(),
                    "res://textures/objects/vegetation/grass/grass_02.png".to_string(),
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
            rooms: vec![],
            corridors: vec![],
            polygons: vec![],
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
                    "res://textures/objects/vegetation/grass/grass_01.png".to_string(),
                    "res://textures/objects/vegetation/grass/grass_02.png".to_string(),
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
            rooms: vec![],
            corridors: vec![],
            polygons: vec![],
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
            rooms: vec![],
            corridors: vec![],
            polygons: vec![],
        },
    }
}

fn desert_preset() -> BiomePreset {
    use crate::elevation::{ContourLevel, ElevationConfig};
    use crate::noise_gen::NoiseConfig;
    use crate::objects::ObjectConfig;
    use crate::terrain::{TerrainConfig, TerrainSlot};

    BiomePreset {
        name: "desert",
        description: "Arid sandy wasteland with rocky outcrops and sparse scrub",
        default_size: (32, 32),
        config: MapConfig {
            name: "Desert Map".to_string(),
            width: 32,
            height: 32,
            seed: None,
            noise: NoiseConfig {
                seed: 0,
                octaves: 4,
                persistence: 0.35,
                lacunarity: 2.0,
                scale: 0.025,
            },
            island_mode: None,
            terrain: Some(TerrainConfig {
                slots: [
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_sand.png".to_string(),
                        lower: 0.0,
                        upper: 0.45,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_dry_grass.png".to_string(),
                        lower: 0.4,
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
                blend_width: 0.07,
                smooth_blending: true,
            }),
            trees: vec![],
            clutter: vec![ObjectConfig {
                textures: vec![
                    "res://textures/objects/vegetation/grass/grass_01.png".to_string(),
                ],
                min_distance: 200.0,
                noise_lower: 0.5,
                noise_upper: 0.7,
                probability: 0.3,
                scale_min: 0.3,
                scale_max: 0.7,
                layer: 100,
                random_rotation: true,
                random_mirror: false,
                custom_color: None,
            }],
            clumps: vec![],
            roads: vec![],
            rivers: vec![],
            water: None,
            elevation: Some(ElevationConfig {
                levels: vec![ContourLevel {
                    threshold: 0.7,
                    texture: "res://textures/paths/path_rocks.png".to_string(),
                    width: 10.0,
                    layer: 100,
                    min_points: 6,
                    smooth_iterations: 2,
                    shadow: None,
                }],
                pixels_per_cell: 64.0,
            }),
            lighting: None,
            rooms: vec![],
            corridors: vec![],
            polygons: vec![],
        },
    }
}

fn lake_preset() -> BiomePreset {
    use crate::noise_gen::NoiseConfig;
    use crate::objects::{ObjectConfig, TreeConfig};
    use crate::terrain::{TerrainConfig, TerrainSlot};
    use crate::water::WaterConfig;

    BiomePreset {
        name: "lake",
        description: "Tranquil woodland pond with grassy shores and scattered trees",
        default_size: (32, 32),
        config: MapConfig {
            name: "Lake Map".to_string(),
            width: 32,
            height: 32,
            seed: None,
            noise: NoiseConfig {
                seed: 0,
                octaves: 5,
                persistence: 0.5,
                lacunarity: 2.0,
                scale: 0.03,
            },
            island_mode: Some(-1.0),
            terrain: Some(TerrainConfig {
                slots: [
                    // Low noise = edges (grass), high noise = center (near water = dirt/mud)
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_grass.png".to_string(),
                        lower: 0.0,
                        upper: 0.35,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_dry_grass.png".to_string(),
                        lower: 0.3,
                        upper: 0.6,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_moss.png".to_string(),
                        lower: 0.55,
                        upper: 0.8,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_dirt.png".to_string(),
                        lower: 0.75,
                        upper: 1.0,
                    },
                ],
                blend_width: 0.06,
                smooth_blending: true,
            }),
            trees: vec![TreeConfig {
                tree: ObjectConfig {
                    textures: vec![
                        "res://textures/objects/more_trees/oak_01.png".to_string(),
                        "res://textures/objects/more_trees/oak_02.png".to_string(),
                    ],
                    min_distance: 600.0,
                    noise_lower: 0.45,
                    noise_upper: 0.7,
                    probability: 0.4,
                    scale_min: 0.8,
                    scale_max: 1.1,
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
                    "res://textures/objects/vegetation/grass/grass_01.png".to_string(),
                    "res://textures/objects/vegetation/grass/grass_02.png".to_string(),
                ],
                min_distance: 70.0,
                noise_lower: 0.3,
                noise_upper: 0.65,
                probability: 0.5,
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
            water: Some(WaterConfig {
                threshold: 0.68,
                deep_color: "ff2a7f6f".to_string(),
                shallow_color: "ff3ac3b2".to_string(),
                blend_distance: 50.0,
                min_contour_points: 20,
                smooth_iterations: 3,
                pixels_per_cell: 64.0,
                disable_border: false,
            }),
            elevation: None,
            lighting: None,
            rooms: vec![],
            corridors: vec![],
            polygons: vec![],
        },
    }
}

fn ice_lake_preset() -> BiomePreset {
    use crate::noise_gen::NoiseConfig;
    use crate::terrain::{TerrainConfig, TerrainSlot};
    use crate::water::WaterConfig;

    BiomePreset {
        name: "ice_lake",
        description: "Frozen lake with cracked ice, snow-covered shores, and frigid water",
        default_size: (32, 32),
        config: MapConfig {
            name: "Ice Lake Map".to_string(),
            width: 32,
            height: 32,
            seed: None,
            noise: NoiseConfig {
                seed: 0,
                octaves: 5,
                persistence: 0.45,
                lacunarity: 2.0,
                scale: 0.03,
            },
            island_mode: Some(-1.0),
            terrain: Some(TerrainConfig {
                slots: [
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_snow.png".to_string(),
                        lower: 0.0,
                        upper: 0.3,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_ice.png".to_string(),
                        lower: 0.25,
                        upper: 0.5,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_gravel.png".to_string(),
                        lower: 0.45,
                        upper: 0.7,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_stone.png".to_string(),
                        lower: 0.65,
                        upper: 1.0,
                    },
                ],
                blend_width: 0.06,
                smooth_blending: true,
            }),
            trees: vec![],
            clutter: vec![],
            clumps: vec![],
            roads: vec![],
            rivers: vec![],
            water: Some(WaterConfig {
                threshold: 0.68,
                deep_color: "ff1a4a5e".to_string(),
                shallow_color: "ff5ea8c0".to_string(),
                blend_distance: 40.0,
                min_contour_points: 20,
                smooth_iterations: 3,
                pixels_per_cell: 64.0,
                disable_border: false,
            }),
            elevation: None,
            lighting: None,
            rooms: vec![],
            corridors: vec![],
            polygons: vec![],
        },
    }
}

fn arctic_preset() -> BiomePreset {
    use crate::elevation::{ContourLevel, ElevationConfig, ShadowPathConfig};
    use crate::noise_gen::NoiseConfig;
    use crate::terrain::{TerrainConfig, TerrainSlot};

    BiomePreset {
        name: "arctic",
        description: "Frozen tundra with snow drifts, exposed rock, and harsh conditions",
        default_size: (32, 32),
        config: MapConfig {
            name: "Arctic Map".to_string(),
            width: 32,
            height: 32,
            seed: None,
            noise: NoiseConfig {
                seed: 0,
                octaves: 5,
                persistence: 0.5,
                lacunarity: 2.0,
                scale: 0.025,
            },
            island_mode: None,
            terrain: Some(TerrainConfig {
                slots: [
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_snow.png".to_string(),
                        lower: 0.0,
                        upper: 0.45,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_ice.png".to_string(),
                        lower: 0.4,
                        upper: 0.65,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_gravel.png".to_string(),
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
            elevation: Some(ElevationConfig {
                levels: vec![ContourLevel {
                    threshold: 0.6,
                    texture: "res://textures/paths/path_rocks.png".to_string(),
                    width: 14.0,
                    layer: 100,
                    min_points: 6,
                    smooth_iterations: 2,
                    shadow: Some(ShadowPathConfig {
                        texture: "res://textures/paths/path_rocks.png".to_string(),
                        offset: 8.0,
                        width: 18.0,
                        layer: 50,
                    }),
                }],
                pixels_per_cell: 64.0,
            }),
            lighting: None,
            rooms: vec![],
            corridors: vec![],
            polygons: vec![],
        },
    }
}

fn island_tropical_preset() -> BiomePreset {
    use crate::noise_gen::NoiseConfig;
    use crate::objects::{ObjectConfig, TreeConfig};
    use crate::terrain::{TerrainConfig, TerrainSlot};
    use crate::water::WaterConfig;

    BiomePreset {
        name: "island_tropical",
        description: "Tropical island with sandy beaches, palm trees, and warm ocean",
        default_size: (32, 32),
        config: MapConfig {
            name: "Tropical Island Map".to_string(),
            width: 32,
            height: 32,
            seed: None,
            noise: NoiseConfig {
                seed: 0,
                octaves: 5,
                persistence: 0.5,
                lacunarity: 2.0,
                scale: 0.035,
            },
            island_mode: Some(1.0),
            terrain: Some(TerrainConfig {
                slots: [
                    // Low noise = center of island (lush), high noise = shore near water
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_grass.png".to_string(),
                        lower: 0.0,
                        upper: 0.35,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_moss.png".to_string(),
                        lower: 0.3,
                        upper: 0.55,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_dry_grass.png".to_string(),
                        lower: 0.5,
                        upper: 0.75,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_sand.png".to_string(),
                        lower: 0.7,
                        upper: 1.0,
                    },
                ],
                blend_width: 0.06,
                smooth_blending: true,
            }),
            trees: vec![TreeConfig {
                tree: ObjectConfig {
                    textures: vec![
                        "res://textures/objects/more_trees/oak_01.png".to_string(),
                        "res://textures/objects/more_trees/oak_02.png".to_string(),
                    ],
                    min_distance: 350.0,
                    noise_lower: 0.0,
                    noise_upper: 0.5,
                    probability: 0.6,
                    scale_min: 0.7,
                    scale_max: 1.1,
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
                    "res://textures/objects/vegetation/grass/grass_01.png".to_string(),
                    "res://textures/objects/vegetation/grass/grass_02.png".to_string(),
                ],
                min_distance: 70.0,
                noise_lower: 0.25,
                noise_upper: 0.6,
                probability: 0.6,
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
            water: Some(WaterConfig {
                threshold: 0.85,
                deep_color: "ff1a6b5a".to_string(),
                shallow_color: "ff30b89a".to_string(),
                blend_distance: 60.0,
                min_contour_points: 20,
                smooth_iterations: 3,
                pixels_per_cell: 64.0,
                disable_border: false,
            }),
            elevation: None,
            lighting: None,
            rooms: vec![],
            corridors: vec![],
            polygons: vec![],
        },
    }
}

fn island_forest_preset() -> BiomePreset {
    use crate::noise_gen::NoiseConfig;
    use crate::objects::{ObjectConfig, TreeConfig};
    use crate::terrain::{TerrainConfig, TerrainSlot};
    use crate::water::WaterConfig;

    BiomePreset {
        name: "island_forest",
        description: "Forested island in a lake with dirt shores and dense tree cover",
        default_size: (32, 32),
        config: MapConfig {
            name: "Forest Island Map".to_string(),
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
            island_mode: Some(1.0),
            terrain: Some(TerrainConfig {
                slots: [
                    // Low noise = center of island (forest floor), high noise = shore
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_moss.png".to_string(),
                        lower: 0.0,
                        upper: 0.35,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_grass.png".to_string(),
                        lower: 0.3,
                        upper: 0.6,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_dry_grass.png".to_string(),
                        lower: 0.55,
                        upper: 0.8,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_dirt.png".to_string(),
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
                        "res://textures/objects/more_trees/oak_01.png".to_string(),
                        "res://textures/objects/more_trees/oak_02.png".to_string(),
                        "res://textures/objects/more_trees/oak_03.png".to_string(),
                    ],
                    min_distance: 300.0,
                    noise_lower: 0.0,
                    noise_upper: 0.55,
                    probability: 0.8,
                    scale_min: 0.8,
                    scale_max: 1.2,
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
                    "res://textures/objects/vegetation/grass/grass_01.png".to_string(),
                    "res://textures/objects/vegetation/grass/grass_02.png".to_string(),
                ],
                min_distance: 80.0,
                noise_lower: 0.2,
                noise_upper: 0.65,
                probability: 0.5,
                scale_min: 0.5,
                scale_max: 1.0,
                layer: 100,
                random_rotation: true,
                random_mirror: false,
                custom_color: None,
            }],
            clumps: vec![],
            roads: vec![],
            rivers: vec![],
            water: Some(WaterConfig {
                threshold: 0.85,
                deep_color: "ff1a5040".to_string(),
                shallow_color: "ff2a8a6a".to_string(),
                blend_distance: 45.0,
                min_contour_points: 20,
                smooth_iterations: 3,
                pixels_per_cell: 64.0,
                disable_border: false,
            }),
            elevation: None,
            lighting: None,
            rooms: vec![],
            corridors: vec![],
            polygons: vec![],
        },
    }
}

fn island_arctic_preset() -> BiomePreset {
    use crate::noise_gen::NoiseConfig;
    use crate::terrain::{TerrainConfig, TerrainSlot};
    use crate::water::WaterConfig;

    BiomePreset {
        name: "island_arctic",
        description: "Snow-covered island surrounded by frigid dark water",
        default_size: (32, 32),
        config: MapConfig {
            name: "Arctic Island Map".to_string(),
            width: 32,
            height: 32,
            seed: None,
            noise: NoiseConfig {
                seed: 0,
                octaves: 5,
                persistence: 0.45,
                lacunarity: 2.0,
                scale: 0.03,
            },
            island_mode: Some(1.0),
            terrain: Some(TerrainConfig {
                slots: [
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_snow.png".to_string(),
                        lower: 0.0,
                        upper: 0.4,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_ice.png".to_string(),
                        lower: 0.35,
                        upper: 0.6,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_gravel.png".to_string(),
                        lower: 0.55,
                        upper: 0.8,
                    },
                    TerrainSlot {
                        texture: "res://textures/terrain/terrain_stone.png".to_string(),
                        lower: 0.75,
                        upper: 1.0,
                    },
                ],
                blend_width: 0.06,
                smooth_blending: true,
            }),
            trees: vec![],
            clutter: vec![],
            clumps: vec![],
            roads: vec![],
            rivers: vec![],
            water: Some(WaterConfig {
                threshold: 0.85,
                deep_color: "ff0f2a3d".to_string(),
                shallow_color: "ff3a7a9a".to_string(),
                blend_distance: 40.0,
                min_contour_points: 20,
                smooth_iterations: 3,
                pixels_per_cell: 64.0,
                disable_border: false,
            }),
            elevation: None,
            lighting: None,
            rooms: vec![],
            corridors: vec![],
            polygons: vec![],
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
        assert!(get_preset("desert").is_some());
        assert!(get_preset("lake").is_some());
        assert!(get_preset("pond").is_some()); // Alias for lake
        assert!(get_preset("ice_lake").is_some());
        assert!(get_preset("ice-lake").is_some()); // Hyphen variant
        assert!(get_preset("arctic").is_some());
        assert!(get_preset("island_tropical").is_some());
        assert!(get_preset("island-tropical").is_some()); // Hyphen variant
        assert!(get_preset("island_forest").is_some());
        assert!(get_preset("island_arctic").is_some());
        assert!(get_preset("Forest").is_some()); // Case insensitive
        assert!(get_preset("unknown").is_none());
    }

    #[test]
    fn test_list_presets() {
        let presets = list_presets();
        assert_eq!(presets.len(), 10);
        let names: Vec<_> = presets.iter().map(|p| p.name).collect();
        assert!(names.contains(&"forest"));
        assert!(names.contains(&"grassland"));
        assert!(names.contains(&"cave"));
        assert!(names.contains(&"desert"));
        assert!(names.contains(&"lake"));
        assert!(names.contains(&"ice_lake"));
        assert!(names.contains(&"arctic"));
        assert!(names.contains(&"island_tropical"));
        assert!(names.contains(&"island_forest"));
        assert!(names.contains(&"island_arctic"));
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

    #[test]
    fn test_lake_has_water() {
        let preset = get_preset("lake").unwrap();
        assert!(preset.config.water.is_some());
        assert!(preset.config.island_mode.is_some());
    }

    #[test]
    fn test_ice_lake_has_water() {
        let preset = get_preset("ice_lake").unwrap();
        assert!(preset.config.water.is_some());
        assert!(preset.config.island_mode.is_some());
    }

    #[test]
    fn test_desert_has_elevation() {
        let preset = get_preset("desert").unwrap();
        assert!(preset.config.elevation.is_some());
        assert!(preset.config.terrain.is_some());
    }

    #[test]
    fn test_arctic_has_elevation() {
        let preset = get_preset("arctic").unwrap();
        assert!(preset.config.elevation.is_some());
        assert!(preset.config.terrain.is_some());
    }

    #[test]
    fn test_island_tropical_has_water_and_trees() {
        let preset = get_preset("island_tropical").unwrap();
        assert!(preset.config.water.is_some());
        assert!(preset.config.island_mode.is_some());
        assert!(!preset.config.trees.is_empty());
    }

    #[test]
    fn test_island_forest_has_water_and_trees() {
        let preset = get_preset("island_forest").unwrap();
        assert!(preset.config.water.is_some());
        assert!(preset.config.island_mode.is_some());
        assert!(!preset.config.trees.is_empty());
    }

    #[test]
    fn test_island_arctic_has_water_no_trees() {
        let preset = get_preset("island_arctic").unwrap();
        assert!(preset.config.water.is_some());
        assert!(preset.config.island_mode.is_some());
        assert!(preset.config.trees.is_empty());
    }
}
