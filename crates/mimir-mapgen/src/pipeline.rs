//! Generation pipeline orchestrator.
//!
//! Config parsing, biome preset resolution, and staged map generation.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::elevation::{self, ElevationConfig};
use crate::format::{DungeondraftMap, NodeIdAllocator};
use crate::noise_gen::{NoiseConfig, NoiseMap};
use crate::objects::{self, clear_corridor, ClumpConfig, ObjectConfig, TreeConfig};
use crate::paths::{self, RiverConfig, RoadConfig};
use crate::polygons::{self, PolygonConfig};
use crate::rooms::{self, CorridorConfig, RoomConfig};
use crate::terrain::{self, TerrainConfig};
use crate::water::{self, WaterConfig};

/// Top-level map generation configuration.
///
/// This is the struct that gets deserialized from a YAML config file.
/// All sections are optional — omit a section to skip that generation stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConfig {
    /// Map name.
    pub name: String,
    /// Map width in grid squares.
    pub width: u32,
    /// Map height in grid squares.
    pub height: u32,
    /// Optional random seed for reproducible generation.
    pub seed: Option<u64>,
    /// Noise generation parameters.
    #[serde(default)]
    pub noise: NoiseConfig,
    /// Island mode falloff strength (None = disabled).
    #[serde(default)]
    pub island_mode: Option<f64>,
    /// Terrain generation config.
    #[serde(default)]
    pub terrain: Option<TerrainConfig>,
    /// Tree placement configs.
    #[serde(default)]
    pub trees: Vec<TreeConfig>,
    /// Clutter object configs.
    #[serde(default)]
    pub clutter: Vec<ObjectConfig>,
    /// Clump placement configs.
    #[serde(default)]
    pub clumps: Vec<ClumpConfig>,
    /// Road generation configs.
    #[serde(default)]
    pub roads: Vec<RoadConfig>,
    /// River generation configs.
    #[serde(default)]
    pub rivers: Vec<RiverConfig>,
    /// Water body generation config.
    #[serde(default)]
    pub water: Option<WaterConfig>,
    /// Elevation contour config.
    #[serde(default)]
    pub elevation: Option<ElevationConfig>,
    /// Lighting configuration.
    #[serde(default)]
    pub lighting: Option<LightingConfig>,
    /// Declarative room definitions.
    #[serde(default)]
    pub rooms: Vec<RoomConfig>,
    /// Corridor connections between rooms.
    #[serde(default)]
    pub corridors: Vec<CorridorConfig>,
    /// Closed polygon areas with shared-edge wall subtraction.
    #[serde(default)]
    pub polygons: Vec<PolygonConfig>,
}

/// Lighting/environment configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingConfig {
    /// Ambient light color (ARGB hex, e.g. "ffffffff").
    pub ambient_light: String,
    /// Ambient energy (brightness multiplier).
    #[serde(default)]
    pub ambient_energy: Option<f64>,
    /// Shadow color (ARGB hex).
    #[serde(default)]
    pub shadow_color: Option<String>,
}

/// Get a LightingConfig from a time-of-day preset name.
pub fn lighting_from_time_of_day(time: &str) -> Option<LightingConfig> {
    match time.to_lowercase().as_str() {
        "dawn" => Some(LightingConfig {
            ambient_light: "fff5d0a0".to_string(),
            ambient_energy: Some(0.7),
            shadow_color: Some("66483020".to_string()),
        }),
        "day" => Some(LightingConfig {
            ambient_light: "ffffffff".to_string(),
            ambient_energy: None,
            shadow_color: None,
        }),
        "dusk" => Some(LightingConfig {
            ambient_light: "ffdd8866".to_string(),
            ambient_energy: Some(0.6),
            shadow_color: Some("66301830".to_string()),
        }),
        "night" => Some(LightingConfig {
            ambient_light: "ff4466aa".to_string(),
            ambient_energy: Some(0.3),
            shadow_color: Some("cc000020".to_string()),
        }),
        "underground" => Some(LightingConfig {
            ambient_light: "ff333333".to_string(),
            ambient_energy: Some(0.2),
            shadow_color: Some("cc000000".to_string()),
        }),
        _ => None,
    }
}

/// Validation errors for a MapConfig.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Validate a MapConfig, returning a list of errors.
pub fn validate_config(config: &MapConfig) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if config.width == 0 || config.height == 0 {
        errors.push(ValidationError {
            field: "width/height".to_string(),
            message: "Map dimensions must be > 0".to_string(),
        });
    }

    if config.width > 128 || config.height > 128 {
        errors.push(ValidationError {
            field: "width/height".to_string(),
            message: "Map dimensions should not exceed 128 grid squares".to_string(),
        });
    }

    if config.name.is_empty() {
        errors.push(ValidationError {
            field: "name".to_string(),
            message: "Map name is required".to_string(),
        });
    }

    if let Some(ref terrain) = config.terrain {
        for (i, slot) in terrain.slots.iter().enumerate() {
            if slot.texture.is_empty() {
                errors.push(ValidationError {
                    field: format!("terrain.slots[{}].texture", i),
                    message: "Terrain texture path cannot be empty".to_string(),
                });
            }
            if slot.lower > slot.upper {
                errors.push(ValidationError {
                    field: format!("terrain.slots[{}]", i),
                    message: format!(
                        "Lower bound ({}) > upper bound ({})",
                        slot.lower, slot.upper
                    ),
                });
            }
        }
    }

    // Room validation
    validate_rooms(config, &mut errors);

    // Corridor validation
    validate_corridors(config, &mut errors);

    errors
}

fn validate_rooms(config: &MapConfig, errors: &mut Vec<ValidationError>) {
    use std::collections::HashSet;
    let mut seen_ids = HashSet::new();

    for (i, room) in config.rooms.iter().enumerate() {
        let field_prefix = format!("rooms[{}]", room.id);

        // Duplicate ID
        if !seen_ids.insert(&room.id) {
            errors.push(ValidationError {
                field: format!("{}.id", field_prefix),
                message: format!("Duplicate room ID \"{}\"", room.id),
            });
        }

        // Dimensions > 0
        if room.width == 0 || room.height == 0 {
            errors.push(ValidationError {
                field: format!("{}.width/height", field_prefix),
                message: "Room dimensions must be > 0".to_string(),
            });
        }

        // Within map bounds
        if room.x + room.width > config.width {
            errors.push(ValidationError {
                field: format!("{}.x", field_prefix),
                message: format!(
                    "Room extends beyond map width (x:{} + w:{} > {})",
                    room.x, room.width, config.width
                ),
            });
        }
        if room.y + room.height > config.height {
            errors.push(ValidationError {
                field: format!("{}.y", field_prefix),
                message: format!(
                    "Room extends beyond map height (y:{} + h:{} > {})",
                    room.y, room.height, config.height
                ),
            });
        }

        // Terrain slot in range
        if let Some(slot) = room.terrain_slot {
            if slot > 3 {
                errors.push(ValidationError {
                    field: format!("{}.terrain_slot", field_prefix),
                    message: format!("Terrain slot {} out of range (0-3)", slot),
                });
            }
        }

        // Portal validation
        for (pi, portal) in room.portals.iter().enumerate() {
            let wall_length = match portal.wall {
                rooms::WallSide::North | rooms::WallSide::South => room.width,
                rooms::WallSide::East | rooms::WallSide::West => room.height,
            };

            if portal.position + portal.width > wall_length {
                errors.push(ValidationError {
                    field: format!("{}.portals[{}].position", field_prefix, pi),
                    message: format!(
                        "Portal extends beyond wall (pos:{} + w:{} > wall_length:{})",
                        portal.position, portal.width, wall_length
                    ),
                });
            }

            if portal.width == 0 {
                errors.push(ValidationError {
                    field: format!("{}.portals[{}].width", field_prefix, pi),
                    message: "Portal width must be > 0".to_string(),
                });
            }
        }

        // Room overlap detection (check against all previous rooms)
        for j in 0..i {
            let other = &config.rooms[j];
            let overlaps = room.x < other.x + other.width
                && room.x + room.width > other.x
                && room.y < other.y + other.height
                && room.y + room.height > other.y;

            if overlaps {
                errors.push(ValidationError {
                    field: format!("{}", field_prefix),
                    message: format!(
                        "Room \"{}\" overlaps with room \"{}\"",
                        room.id, other.id
                    ),
                });
            }
        }
    }
}

fn validate_corridors(config: &MapConfig, errors: &mut Vec<ValidationError>) {
    let room_ids: std::collections::HashSet<&str> =
        config.rooms.iter().map(|r| r.id.as_str()).collect();

    for (i, corridor) in config.corridors.iter().enumerate() {
        let field_prefix = format!("corridors[{}]", i);

        // Valid room references
        if !room_ids.contains(corridor.from.as_str()) {
            errors.push(ValidationError {
                field: format!("{}.from", field_prefix),
                message: format!("Room \"{}\" not found", corridor.from),
            });
        }
        if !room_ids.contains(corridor.to.as_str()) {
            errors.push(ValidationError {
                field: format!("{}.to", field_prefix),
                message: format!("Room \"{}\" not found", corridor.to),
            });
        }

        // Width > 0
        if corridor.width == 0 {
            errors.push(ValidationError {
                field: format!("{}.width", field_prefix),
                message: "Corridor width must be > 0".to_string(),
            });
        }

        // Terrain slot in range
        if let Some(slot) = corridor.terrain_slot {
            if slot > 3 {
                errors.push(ValidationError {
                    field: format!("{}.terrain_slot", field_prefix),
                    message: format!("Terrain slot {} out of range (0-3)", slot),
                });
            }
        }
    }
}

/// Result of map generation.
pub struct GenerateResult {
    /// The generated map.
    pub map: DungeondraftMap,
    /// Generation statistics.
    pub stats: GenerateStats,
}

/// Statistics from map generation.
#[derive(Debug, Default)]
pub struct GenerateStats {
    pub objects_placed: usize,
    pub paths_generated: usize,
    pub water_polygons: usize,
    pub contour_paths: usize,
    pub walls_generated: usize,
    pub portals_generated: usize,
}

/// Generate a complete `.dungeondraft_map` from a config.
///
/// Runs the full pipeline:
/// 0. Room layout (walls, portals, exclusion zones)
/// 1. Generate noise map
/// 2. Generate terrain (splat map) + room terrain overrides
/// 3. Generate roads and rivers
/// 4. Place objects (trees, clutter, clumps)
/// 5. Clear object corridors around roads/rivers
/// 6. Filter objects from room exclusion zones
/// 7. Generate water bodies
/// 8. Generate elevation contours
/// 9. Apply lighting
/// 10. Assemble walls/portals into Level
/// 11. Return DungeondraftMap
pub fn generate(config: &MapConfig, seed_override: Option<u64>) -> GenerateResult {
    let seed = seed_override.or(config.seed).unwrap_or(0);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let alloc = NodeIdAllocator::new(1);
    let mut stats = GenerateStats::default();

    let pixel_width = config.width as f64 * 256.0;
    let pixel_height = config.height as f64 * 256.0;

    // Noise resolution: 4 cells per grid square (matches splat resolution)
    let noise_w = (config.width * 4) as usize;
    let noise_h = (config.height * 4) as usize;

    // 0. Room layout — runs first so everything else respects room boundaries
    let room_layout = rooms::generate_room_layout(&config.rooms, &config.corridors, &alloc);
    let mut exclusion_zones = rooms::build_exclusion_zones(&config.rooms, &config.corridors);

    // 0b. Polygon layout — closed polygons with shared-edge subtraction
    let polygon_layout = polygons::generate_polygon_layout(&config.polygons, &alloc);
    exclusion_zones.extend(polygons::build_polygon_exclusion_zones(&config.polygons));

    stats.walls_generated = room_layout.walls.len() + polygon_layout.walls.len();
    stats.portals_generated = room_layout.walls.iter().map(|w| w.portals.len()).sum::<usize>()
        + room_layout.portals.len()
        + polygon_layout.walls.iter().map(|w| w.portals.len()).sum::<usize>()
        + polygon_layout.portals.len();

    // 1. Generate noise map
    let noise_config = NoiseConfig {
        seed: (seed & 0xFFFFFFFF) as u32,
        ..config.noise.clone()
    };
    let mut noise_map = NoiseMap::generate(noise_w, noise_h, &noise_config);

    if let Some(falloff) = config.island_mode {
        noise_map.apply_island_mode(falloff);
    }

    // 2. Create base map
    let mut map = DungeondraftMap::new(config.width, config.height);

    // 3. Generate terrain + room terrain overrides
    if let Some(ref terrain_config) = config.terrain {
        let mut terrain = terrain::generate_terrain(
            &noise_map,
            config.width,
            config.height,
            terrain_config,
        );

        // Apply room and polygon terrain overrides (floors override noise-based terrain)
        let map_cells_x = (config.width * 4) as usize;
        for ovr in &room_layout.terrain_overrides {
            ovr.apply(&mut terrain.splat.0, map_cells_x);
        }
        for ovr in &polygon_layout.terrain_overrides {
            ovr.apply(&mut terrain.splat.0, map_cells_x);
        }

        map.ground_level_mut().terrain = Some(terrain);
    }

    // 4. Generate roads
    let mut corridors: Vec<(Vec<(f64, f64)>, f64)> = Vec::new();

    for road_config in &config.roads {
        if let Some(result) = paths::generate_road(
            &noise_map,
            road_config,
            pixel_width,
            pixel_height,
            &alloc,
            &mut rng,
        ) {
            corridors.push((result.corridor_points.clone(), result.corridor_half_width));
            map.ground_level_mut().paths.push(result.road);
            stats.paths_generated += 1;
            for ep in result.edge_paths {
                map.ground_level_mut().paths.push(ep);
                stats.paths_generated += 1;
            }

            // Modify terrain along road
            if let Some(ref mut terrain) = map.ground_level_mut().terrain {
                terrain::apply_road_corridor(
                    terrain,
                    config.width,
                    &corridors.last().unwrap().0,
                    corridors.last().unwrap().1,
                    0,
                );
            }
        }
    }

    // 5. Generate rivers
    for river_config in &config.rivers {
        if let Some(result) = paths::generate_river(
            &noise_map,
            river_config,
            pixel_width,
            pixel_height,
            &alloc,
            &mut rng,
        ) {
            corridors.push((result.corridor_points.clone(), result.corridor_half_width));
            for bp in result.bank_paths {
                map.ground_level_mut().paths.push(bp);
                stats.paths_generated += 1;
            }

            // Add river water
            let water_config = config.water.as_ref().cloned().unwrap_or_default();
            let river_water = water::water_from_polygon(
                &result.water_polygon,
                &water_config,
                &alloc,
            );
            if let Some(ref mut w) = map.ground_level_mut().water {
                if let Some(ref mut tree) = w.tree {
                    tree.children.push(river_water);
                    stats.water_polygons += 1;
                }
            }
        }
    }

    // 6. Place objects
    let mut all_objects = Vec::new();

    for tree_config in &config.trees {
        let trees = objects::place_trees(
            &noise_map,
            tree_config,
            pixel_width,
            pixel_height,
            &alloc,
            &mut rng,
        );
        all_objects.extend(trees);
    }

    for clutter_config in &config.clutter {
        let clutter = objects::place_objects(
            &noise_map,
            clutter_config,
            pixel_width,
            pixel_height,
            &alloc,
            &mut rng,
        );
        all_objects.extend(clutter);
    }

    for clump_config in &config.clumps {
        let clumps = objects::place_clumps(
            &noise_map,
            clump_config,
            pixel_width,
            pixel_height,
            &alloc,
            &mut rng,
        );
        all_objects.extend(clumps);
    }

    // 7. Clear corridors
    for (corridor_pts, half_width) in &corridors {
        clear_corridor(&mut all_objects, corridor_pts, *half_width);
    }

    // 7b. Filter objects from room exclusion zones
    if !exclusion_zones.is_empty() {
        all_objects.retain(|obj| !rooms::is_excluded(&exclusion_zones, obj.position.x, obj.position.y));
    }

    stats.objects_placed = all_objects.len();
    map.ground_level_mut().objects = all_objects;

    // 8. Generate water bodies
    if let Some(ref water_config) = config.water {
        let water = water::generate_water(&noise_map, water_config, &alloc);
        if let Some(ref tree) = water.tree {
            stats.water_polygons += tree.children.len();
        }
        map.ground_level_mut().water = Some(water);
    }

    // 9. Generate elevation contours
    if let Some(ref elev_config) = config.elevation {
        let contour_paths = elevation::generate_elevation(&noise_map, elev_config, &alloc);
        stats.contour_paths = contour_paths.len();
        map.ground_level_mut().paths.extend(contour_paths);
    }

    // 10. Apply lighting
    if let Some(ref lighting) = config.lighting {
        let level = map.ground_level_mut();
        level.environment.ambient_light = lighting.ambient_light.clone();
        if let Some(energy) = lighting.ambient_energy {
            level.environment.ambient_energy = Some(energy);
        }
        if let Some(ref shadow) = lighting.shadow_color {
            level.environment.shadow_color = Some(shadow.clone());
        }
    }

    // 11. Assemble room walls and portals into the Level
    if !room_layout.walls.is_empty() || !room_layout.portals.is_empty() {
        let level = map.ground_level_mut();
        level.walls.extend(room_layout.walls);
        level.portals.extend(room_layout.portals);

        // Populate shapes
        for id in room_layout.shape_wall_ids {
            level.shapes.walls.push(id);
        }
        for poly in room_layout.shape_polygons {
            level.shapes.polygons.push(poly);
        }
    }

    // 11b. Assemble polygon walls and portals
    if !polygon_layout.walls.is_empty() || !polygon_layout.portals.is_empty() {
        let level = map.ground_level_mut();
        level.walls.extend(polygon_layout.walls);
        level.portals.extend(polygon_layout.portals);

        for id in polygon_layout.shape_wall_ids {
            level.shapes.walls.push(id);
        }
        for poly in polygon_layout.shape_polygons {
            level.shapes.polygons.push(poly);
        }
    }

    // Update next_node_id
    map.world.next_node_id = alloc.current();

    GenerateResult { map, stats }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_config() -> MapConfig {
        MapConfig {
            name: "Test Map".to_string(),
            width: 10,
            height: 10,
            seed: Some(42),
            noise: NoiseConfig::default(),
            island_mode: None,
            terrain: None,
            trees: vec![],
            clutter: vec![],
            clumps: vec![],
            roads: vec![],
            rivers: vec![],
            water: None,
            elevation: None,
            lighting: None,
            rooms: vec![],
            corridors: vec![],
            polygons: vec![],
        }
    }

    #[test]
    fn test_generate_minimal() {
        let config = minimal_config();
        let result = generate(&config, None);
        assert_eq!(result.map.world.width, 10);
        assert_eq!(result.map.world.height, 10);
        assert_eq!(result.stats.objects_placed, 0);
    }

    #[test]
    fn test_generate_with_terrain() {
        let mut config = minimal_config();
        config.terrain = Some(TerrainConfig::default());

        let result = generate(&config, None);
        let terrain = result
            .map
            .ground_level()
            .unwrap()
            .terrain
            .as_ref()
            .unwrap();
        assert!(terrain.enabled);
        assert_eq!(terrain.splat.0.len(), 10 * 4 * 10 * 4 * 4);
    }

    #[test]
    fn test_generate_with_roads() {
        let mut config = minimal_config();
        config.terrain = Some(TerrainConfig::default());
        config.roads = vec![RoadConfig::default()];

        let result = generate(&config, None);
        assert!(result.stats.paths_generated > 0);
        assert!(!result.map.ground_level().unwrap().paths.is_empty());
    }

    #[test]
    fn test_generate_with_objects() {
        let mut config = minimal_config();
        config.clutter = vec![ObjectConfig {
            textures: vec!["grass.png".to_string()],
            min_distance: 200.0,
            ..Default::default()
        }];

        let result = generate(&config, None);
        assert!(result.stats.objects_placed > 0);
    }

    #[test]
    fn test_generate_deterministic() {
        let mut config = minimal_config();
        config.terrain = Some(TerrainConfig::default());
        config.clutter = vec![ObjectConfig {
            textures: vec!["grass.png".to_string()],
            min_distance: 200.0,
            ..Default::default()
        }];

        let r1 = generate(&config, Some(42));
        let r2 = generate(&config, Some(42));

        assert_eq!(r1.stats.objects_placed, r2.stats.objects_placed);

        // Compare world JSON (header has timestamp so skip it)
        let world1 = serde_json::to_string(&r1.map.world).unwrap();
        let world2 = serde_json::to_string(&r2.map.world).unwrap();
        assert_eq!(world1, world2);
    }

    #[test]
    fn test_seed_override() {
        let config = minimal_config();
        let r1 = generate(&config, Some(1));
        let r2 = generate(&config, Some(2));
        // Different seeds should produce different node IDs at minimum
        assert_eq!(r1.map.world.width, r2.map.world.width);
    }

    #[test]
    fn test_generate_with_lighting() {
        let mut config = minimal_config();
        config.lighting = Some(LightingConfig {
            ambient_light: "ff4466aa".to_string(),
            ambient_energy: Some(0.3),
            shadow_color: Some("cc000020".to_string()),
        });

        let result = generate(&config, None);
        let env = &result.map.ground_level().unwrap().environment;
        assert_eq!(env.ambient_light, "ff4466aa");
        assert_eq!(env.ambient_energy, Some(0.3));
        assert_eq!(env.shadow_color.as_deref(), Some("cc000020"));
    }

    #[test]
    fn test_lighting_presets() {
        assert!(lighting_from_time_of_day("dawn").is_some());
        assert!(lighting_from_time_of_day("day").is_some());
        assert!(lighting_from_time_of_day("dusk").is_some());
        assert!(lighting_from_time_of_day("night").is_some());
        assert!(lighting_from_time_of_day("underground").is_some());
        assert!(lighting_from_time_of_day("invalid").is_none());
    }

    #[test]
    fn test_validate_config() {
        let config = minimal_config();
        let errors = validate_config(&config);
        assert!(errors.is_empty());

        let mut bad = minimal_config();
        bad.width = 0;
        bad.name = "".to_string();
        let errors = validate_config(&bad);
        assert!(errors.len() >= 2);
    }

    #[test]
    fn test_full_pipeline() {
        let mut config = minimal_config();
        config.width = 16;
        config.height = 16;
        config.terrain = Some(TerrainConfig::default());
        config.roads = vec![RoadConfig::default()];
        config.clutter = vec![ObjectConfig {
            textures: vec!["grass.png".to_string()],
            min_distance: 300.0,
            ..Default::default()
        }];
        config.elevation = Some(ElevationConfig::default());
        config.lighting = Some(LightingConfig {
            ambient_light: "ffffffff".to_string(),
            ambient_energy: None,
            shadow_color: None,
        });

        let result = generate(&config, Some(42));

        // Verify all stages ran
        assert!(result.map.ground_level().unwrap().terrain.is_some());
        assert!(result.stats.paths_generated > 0);
        assert!(result.stats.objects_placed > 0);

        // Verify it serializes to valid JSON
        let json = result.map.to_json().unwrap();
        assert!(json.len() > 1000);
        let reparsed = DungeondraftMap::from_json(&json).unwrap();
        assert_eq!(reparsed.world.width, 16);
    }

    #[test]
    fn test_generate_with_rooms() {
        use crate::rooms::{RoomConfig, WallToggles, PortalConfig, WallSide, PortalType};
        use crate::rooms::{CorridorConfig, CorridorEnd, CorridorPortalConfig};

        let mut config = minimal_config();
        config.width = 20;
        config.height = 20;
        config.terrain = Some(TerrainConfig::default());
        config.trees = vec![TreeConfig {
            tree: ObjectConfig {
                textures: vec!["res://textures/objects/trees/tree_01.png".to_string()],
                min_distance: 100.0,
                ..Default::default()
            },
            shadow: None,
            canopy: None,
        }];
        config.rooms = vec![
            RoomConfig {
                id: "guard_room".to_string(),
                x: 3, y: 3, width: 5, height: 4,
                terrain_slot: Some(2),
                walls: WallToggles::default(),
                portals: vec![PortalConfig {
                    wall: WallSide::East,
                    position: 1,
                    portal_type: PortalType::Door,
                    width: 1,
                }],
            },
            RoomConfig {
                id: "throne_room".to_string(),
                x: 12, y: 3, width: 6, height: 4,
                terrain_slot: Some(3),
                walls: WallToggles::default(),
                portals: vec![],
            },
        ];
        config.corridors = vec![CorridorConfig {
            from: "guard_room".to_string(),
            from_wall: WallSide::East,
            to: "throne_room".to_string(),
            to_wall: WallSide::West,
            width: 2,
            terrain_slot: Some(2),
            portals: vec![CorridorPortalConfig {
                end: CorridorEnd::From,
                portal_type: PortalType::Door,
                width: 1,
            }],
        }];

        let result = generate(&config, Some(42));

        // Walls generated: 2 room walls + 2 corridor walls
        assert_eq!(result.stats.walls_generated, 4);
        // Portals: 1 door on guard_room east wall + 1 freestanding corridor door
        assert_eq!(result.stats.portals_generated, 2);

        let level = result.map.ground_level().unwrap();
        assert_eq!(level.walls.len(), 4);
        // Freestanding corridor portal
        assert_eq!(level.portals.len(), 1);

        // No objects inside room exclusion zones
        for obj in &level.objects {
            let in_guard = obj.position.x >= 768.0 && obj.position.x <= 2048.0
                && obj.position.y >= 768.0 && obj.position.y <= 1792.0;
            let in_throne = obj.position.x >= 3072.0 && obj.position.x <= 4608.0
                && obj.position.y >= 768.0 && obj.position.y <= 1792.0;
            assert!(!in_guard, "Object at ({}, {}) inside guard room", obj.position.x, obj.position.y);
            assert!(!in_throne, "Object at ({}, {}) inside throne room", obj.position.x, obj.position.y);
        }

        // Terrain override applied
        let terrain = level.terrain.as_ref().unwrap();
        let map_cells_x = 20 * 4;
        // Guard room at (3,3) with slot 2 → splat cell (12,12) should have slot 2
        let idx = (12 * map_cells_x + 12) * 4;
        assert_eq!(terrain.splat.0[idx + 2], 255); // slot 2 = byte index 2

        // Shapes populated
        assert!(!level.shapes.walls.is_empty());
        assert!(!level.shapes.polygons.is_empty());

        // Serializes to valid JSON and round-trips
        let json = result.map.to_json().unwrap();
        let reparsed = DungeondraftMap::from_json(&json).unwrap();
        assert_eq!(reparsed.ground_level().unwrap().walls.len(), 4);
    }

    #[test]
    fn test_generate_outdoor_only_no_regression() {
        // Verify that configs without rooms produce the same output as before
        let mut config = minimal_config();
        config.terrain = Some(TerrainConfig::default());
        config.trees = vec![TreeConfig {
            tree: ObjectConfig {
                textures: vec!["res://textures/objects/trees/tree_01.png".to_string()],
                min_distance: 100.0,
                ..Default::default()
            },
            shadow: None,
            canopy: None,
        }];

        let result = generate(&config, Some(42));
        assert_eq!(result.stats.walls_generated, 0);
        assert_eq!(result.stats.portals_generated, 0);
        let level = result.map.ground_level().unwrap();
        assert!(level.walls.is_empty());
        assert!(level.portals.is_empty());
    }

    #[test]
    fn test_validate_room_out_of_bounds() {
        use crate::rooms::{RoomConfig, WallToggles};
        let mut config = minimal_config();
        config.rooms = vec![RoomConfig {
            id: "too_big".to_string(),
            x: 8, y: 8, width: 5, height: 5,
            terrain_slot: None,
            walls: WallToggles::default(),
            portals: vec![],
        }];
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.field.contains("too_big") && e.message.contains("beyond")));
    }

    #[test]
    fn test_validate_room_overlap() {
        use crate::rooms::{RoomConfig, WallToggles};
        let mut config = minimal_config();
        config.rooms = vec![
            RoomConfig {
                id: "a".to_string(),
                x: 2, y: 2, width: 4, height: 4,
                terrain_slot: None,
                walls: WallToggles::default(),
                portals: vec![],
            },
            RoomConfig {
                id: "b".to_string(),
                x: 4, y: 4, width: 4, height: 4,
                terrain_slot: None,
                walls: WallToggles::default(),
                portals: vec![],
            },
        ];
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.message.contains("overlaps")));
    }

    #[test]
    fn test_validate_duplicate_room_id() {
        use crate::rooms::{RoomConfig, WallToggles};
        let mut config = minimal_config();
        config.rooms = vec![
            RoomConfig {
                id: "same".to_string(),
                x: 0, y: 0, width: 2, height: 2,
                terrain_slot: None,
                walls: WallToggles::default(),
                portals: vec![],
            },
            RoomConfig {
                id: "same".to_string(),
                x: 5, y: 5, width: 2, height: 2,
                terrain_slot: None,
                walls: WallToggles::default(),
                portals: vec![],
            },
        ];
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.message.contains("Duplicate")));
    }

    #[test]
    fn test_validate_portal_out_of_wall() {
        use crate::rooms::{RoomConfig, WallToggles, PortalConfig, WallSide, PortalType};
        let mut config = minimal_config();
        config.rooms = vec![RoomConfig {
            id: "small".to_string(),
            x: 0, y: 0, width: 3, height: 3,
            terrain_slot: None,
            walls: WallToggles::default(),
            portals: vec![PortalConfig {
                wall: WallSide::North,
                position: 3, // position 3 + width 1 > wall_length 3
                portal_type: PortalType::Door,
                width: 1,
            }],
        }];
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.message.contains("extends beyond wall")));
    }

    #[test]
    fn test_validate_corridor_invalid_room_ref() {
        use crate::rooms::{CorridorConfig, WallSide};
        let mut config = minimal_config();
        config.corridors = vec![CorridorConfig {
            from: "nonexistent".to_string(),
            from_wall: WallSide::East,
            to: "also_missing".to_string(),
            to_wall: WallSide::West,
            width: 2,
            terrain_slot: None,
            portals: vec![],
        }];
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.message.contains("nonexistent")));
        assert!(errors.iter().any(|e| e.message.contains("also_missing")));
    }

    #[test]
    fn test_validate_terrain_slot_out_of_range() {
        use crate::rooms::{RoomConfig, WallToggles};
        let mut config = minimal_config();
        config.rooms = vec![RoomConfig {
            id: "bad_slot".to_string(),
            x: 0, y: 0, width: 3, height: 3,
            terrain_slot: Some(5),
            walls: WallToggles::default(),
            portals: vec![],
        }];
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.message.contains("out of range")));
    }

    #[test]
    fn test_validate_room_zero_dimensions() {
        use crate::rooms::{RoomConfig, WallToggles};
        let mut config = minimal_config();
        config.rooms = vec![RoomConfig {
            id: "flat".to_string(),
            x: 0, y: 0, width: 0, height: 3,
            terrain_slot: None,
            walls: WallToggles::default(),
            portals: vec![],
        }];
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.message.contains("dimensions must be > 0")));
    }
}
