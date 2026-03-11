//! Object placement: trees, clumps, and clutter.
//!
//! Uses noise-gated Poisson Disc sampling for natural object distribution.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

use crate::distribution::{Point, PoissonDisc};
use crate::format::entities::MapObject;
use crate::format::godot_types::Vector2;
use crate::format::NodeIdAllocator;
use crate::noise_gen::NoiseMap;

/// Configuration for a single object type to place.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectConfig {
    /// List of texture paths to randomly select from.
    pub textures: Vec<String>,
    /// Minimum distance between objects of this type.
    pub min_distance: f64,
    /// Noise thresholds for placement (lower, upper). Objects only placed where noise is in range.
    pub noise_lower: f64,
    pub noise_upper: f64,
    /// Probability of keeping each valid point (0.0–1.0).
    pub probability: f64,
    /// Size range (min_scale, max_scale).
    pub scale_min: f64,
    pub scale_max: f64,
    /// Layer in the DD map.
    pub layer: i32,
    /// Whether to randomize rotation.
    pub random_rotation: bool,
    /// Whether to randomly mirror objects.
    pub random_mirror: bool,
    /// Optional custom color (ARGB hex string).
    pub custom_color: Option<String>,
}

impl Default for ObjectConfig {
    fn default() -> Self {
        Self {
            textures: Vec::new(),
            min_distance: 20.0,
            noise_lower: 0.0,
            noise_upper: 1.0,
            probability: 1.0,
            scale_min: 1.0,
            scale_max: 1.0,
            layer: 100,
            random_rotation: true,
            random_mirror: false,
            custom_color: None,
        }
    }
}

/// Configuration for tree placement with optional shadow and canopy layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeConfig {
    /// Base tree object config.
    pub tree: ObjectConfig,
    /// Optional shadow object placed underneath (lower layer).
    pub shadow: Option<ShadowConfig>,
    /// Optional canopy object placed on upper level.
    pub canopy: Option<CanopyConfig>,
}

/// Shadow configuration for trees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowConfig {
    /// Shadow texture path.
    pub texture: String,
    /// Offset from tree position in pixels.
    pub offset: Vector2,
    /// Layer (should be below tree layer).
    pub layer: i32,
    /// Scale relative to tree scale.
    pub scale_factor: f64,
}

/// Canopy configuration for trees (level 1 overhead foliage).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanopyConfig {
    /// Canopy texture path.
    pub texture: String,
    /// Layer on level 1.
    pub layer: i32,
    /// Scale relative to tree scale.
    pub scale_factor: f64,
}

/// Configuration for clump placement (primary + clustered secondaries).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClumpConfig {
    /// Primary object config (placed by Poisson Disc).
    pub primary: ObjectConfig,
    /// Secondary objects scattered around each primary.
    pub secondary: ObjectConfig,
    /// Number of secondaries per primary (min, max).
    pub secondary_count: (u32, u32),
    /// Max distance of secondaries from primary in pixels.
    pub secondary_radius: f64,
}

/// Place simple objects using noise-gated Poisson Disc sampling.
///
/// # Arguments
/// * `noise_map` - Noise field for gating placement
/// * `config` - Object placement configuration
/// * `pixel_width` - Map width in pixels (grid_squares * 256)
/// * `pixel_height` - Map height in pixels
/// * `alloc` - Node ID allocator
/// * `rng` - Seeded random number generator
pub fn place_objects(
    noise_map: &NoiseMap,
    config: &ObjectConfig,
    pixel_width: f64,
    pixel_height: f64,
    alloc: &NodeIdAllocator,
    rng: &mut impl Rng,
) -> Vec<MapObject> {
    if config.textures.is_empty() {
        return Vec::new();
    }

    // Scale noise coordinates: noise map covers the full pixel area
    let scale_x = noise_map.width as f64 / pixel_width;
    let scale_y = noise_map.height as f64 / pixel_height;

    let points = PoissonDisc::sample(pixel_width, pixel_height, config.min_distance, rng, 30);

    // Filter points by noise gate and probability
    let filtered: Vec<_> = points
        .into_iter()
        .filter(|&(x, y)| {
            let nx = x * scale_x;
            let ny = y * scale_y;
            let noise_val = noise_map.sample(nx, ny);
            noise_val >= config.noise_lower
                && noise_val <= config.noise_upper
                && rng.gen::<f64>() < config.probability
        })
        .collect();

    // Build map objects
    filtered
        .into_iter()
        .map(|(x, y)| {
            let texture = &config.textures[rng.gen_range(0..config.textures.len())];
            let scale = rng.gen_range(config.scale_min..=config.scale_max);
            let rotation = if config.random_rotation {
                rng.gen_range(0.0..2.0 * PI)
            } else {
                0.0
            };
            let mirror = config.random_mirror && rng.gen::<bool>();

            let mut obj =
                MapObject::new(texture, Vector2::new(x, y), &alloc.next())
                    .with_scale(scale)
                    .with_rotation(rotation)
                    .with_layer(config.layer)
                    .with_mirror(mirror);

            if let Some(ref color) = config.custom_color {
                obj = obj.with_custom_color(color);
            }

            obj
        })
        .collect()
}

/// Place trees with optional shadow and canopy objects.
pub fn place_trees(
    noise_map: &NoiseMap,
    config: &TreeConfig,
    pixel_width: f64,
    pixel_height: f64,
    alloc: &NodeIdAllocator,
    rng: &mut impl Rng,
) -> Vec<MapObject> {
    let trees = place_objects(noise_map, &config.tree, pixel_width, pixel_height, alloc, rng);
    let mut result = Vec::with_capacity(trees.len() * 3);

    for tree in &trees {
        // Shadow first (below tree)
        if let Some(ref shadow) = config.shadow {
            let shadow_obj = MapObject::new(
                &shadow.texture,
                Vector2::new(
                    tree.position.x + shadow.offset.x,
                    tree.position.y + shadow.offset.y,
                ),
                &alloc.next(),
            )
            .with_scale(tree.scale.x * shadow.scale_factor)
            .with_rotation(tree.rotation)
            .with_layer(shadow.layer);
            result.push(shadow_obj);
        }

        // Tree itself
        result.push(tree.clone());

        // Canopy on top
        if let Some(ref canopy) = config.canopy {
            let canopy_obj = MapObject::new(
                &canopy.texture,
                tree.position,
                &alloc.next(),
            )
            .with_scale(tree.scale.x * canopy.scale_factor)
            .with_rotation(tree.rotation)
            .with_layer(canopy.layer);
            result.push(canopy_obj);
        }
    }

    result
}

/// Place clumps: primary objects with clustered secondaries around each.
pub fn place_clumps(
    noise_map: &NoiseMap,
    config: &ClumpConfig,
    pixel_width: f64,
    pixel_height: f64,
    alloc: &NodeIdAllocator,
    rng: &mut impl Rng,
) -> Vec<MapObject> {
    let primaries = place_objects(
        noise_map,
        &config.primary,
        pixel_width,
        pixel_height,
        alloc,
        rng,
    );

    let mut result = Vec::new();

    for primary in &primaries {
        result.push(primary.clone());

        // Scatter secondaries around the primary
        let count = rng.gen_range(config.secondary_count.0..=config.secondary_count.1);
        for _ in 0..count {
            let angle = rng.gen_range(0.0..2.0 * PI);
            let dist = rng.gen_range(0.0..config.secondary_radius);
            let sx = primary.position.x + angle.cos() * dist;
            let sy = primary.position.y + angle.sin() * dist;

            if sx < 0.0 || sx >= pixel_width || sy < 0.0 || sy >= pixel_height {
                continue;
            }

            let texture = if config.secondary.textures.is_empty() {
                continue;
            } else {
                &config.secondary.textures[rng.gen_range(0..config.secondary.textures.len())]
            };

            let scale = rng.gen_range(config.secondary.scale_min..=config.secondary.scale_max);
            let rotation = if config.secondary.random_rotation {
                rng.gen_range(0.0..2.0 * PI)
            } else {
                0.0
            };

            let mut obj =
                MapObject::new(texture, Vector2::new(sx, sy), &alloc.next())
                    .with_scale(scale)
                    .with_rotation(rotation)
                    .with_layer(config.secondary.layer);

            if let Some(ref color) = config.secondary.custom_color {
                obj = obj.with_custom_color(color);
            }

            result.push(obj);
        }
    }

    result
}

/// Remove objects that fall within a corridor (e.g., road clearing).
///
/// # Arguments
/// * `objects` - Mutable list of objects to filter
/// * `corridor` - List of center points defining the corridor
/// * `half_width` - Half-width of the corridor in pixels
pub fn clear_corridor(objects: &mut Vec<MapObject>, corridor: &[Point], half_width: f64) {
    let half_sq = half_width * half_width;
    objects.retain(|obj| {
        !corridor.iter().any(|&(cx, cy)| {
            let dx = obj.position.x - cx;
            let dy = obj.position.y - cy;
            dx * dx + dy * dy <= half_sq
        })
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noise_gen::{NoiseConfig, NoiseMap};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn test_noise() -> NoiseMap {
        NoiseMap::generate(100, 100, &NoiseConfig { seed: 42, ..Default::default() })
    }

    #[test]
    fn test_place_objects_basic() {
        let noise = test_noise();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = ObjectConfig {
            textures: vec!["res://textures/objects/tree.png".to_string()],
            min_distance: 200.0,
            ..Default::default()
        };

        let objects = place_objects(&noise, &config, 2560.0, 2560.0, &alloc, &mut rng);
        assert!(!objects.is_empty());

        for obj in &objects {
            assert!(obj.position.x >= 0.0 && obj.position.x < 2560.0);
            assert!(obj.position.y >= 0.0 && obj.position.y < 2560.0);
            assert_eq!(obj.texture, "res://textures/objects/tree.png");
        }
    }

    #[test]
    fn test_place_objects_noise_gated() {
        let noise = test_noise();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let narrow = ObjectConfig {
            textures: vec!["tree.png".to_string()],
            min_distance: 100.0,
            noise_lower: 0.4,
            noise_upper: 0.6,
            ..Default::default()
        };

        let wide = ObjectConfig {
            textures: vec!["tree.png".to_string()],
            min_distance: 100.0,
            noise_lower: 0.0,
            noise_upper: 1.0,
            ..Default::default()
        };

        let narrow_objs = place_objects(&noise, &narrow, 2560.0, 2560.0, &alloc, &mut rng);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let wide_objs = place_objects(&noise, &wide, 2560.0, 2560.0, &alloc, &mut rng2);

        assert!(narrow_objs.len() < wide_objs.len());
    }

    #[test]
    fn test_place_objects_deterministic() {
        let noise = test_noise();
        let alloc1 = NodeIdAllocator::new(1);
        let alloc2 = NodeIdAllocator::new(1);
        let config = ObjectConfig {
            textures: vec!["tree.png".to_string()],
            min_distance: 200.0,
            ..Default::default()
        };

        let mut rng1 = ChaCha8Rng::seed_from_u64(99);
        let mut rng2 = ChaCha8Rng::seed_from_u64(99);

        let objs1 = place_objects(&noise, &config, 2560.0, 2560.0, &alloc1, &mut rng1);
        let objs2 = place_objects(&noise, &config, 2560.0, 2560.0, &alloc2, &mut rng2);

        assert_eq!(objs1.len(), objs2.len());
        for (a, b) in objs1.iter().zip(objs2.iter()) {
            assert_eq!(a.position.x, b.position.x);
            assert_eq!(a.position.y, b.position.y);
        }
    }

    #[test]
    fn test_place_trees_with_shadow() {
        let noise = test_noise();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let config = TreeConfig {
            tree: ObjectConfig {
                textures: vec!["tree.png".to_string()],
                min_distance: 300.0,
                ..Default::default()
            },
            shadow: Some(ShadowConfig {
                texture: "shadow.png".to_string(),
                offset: Vector2::new(10.0, 10.0),
                layer: -100,
                scale_factor: 1.2,
            }),
            canopy: None,
        };

        let objects = place_trees(&noise, &config, 2560.0, 2560.0, &alloc, &mut rng);
        // Should have 2 objects per tree (shadow + tree)
        assert!(objects.len() > 0);
        assert_eq!(objects.len() % 2, 0);

        // First object of each pair should be shadow
        assert_eq!(objects[0].texture, "shadow.png");
        assert_eq!(objects[1].texture, "tree.png");
    }

    #[test]
    fn test_place_clumps() {
        let noise = test_noise();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let config = ClumpConfig {
            primary: ObjectConfig {
                textures: vec!["rock.png".to_string()],
                min_distance: 400.0,
                ..Default::default()
            },
            secondary: ObjectConfig {
                textures: vec!["pebble.png".to_string()],
                scale_min: 0.5,
                scale_max: 0.8,
                ..Default::default()
            },
            secondary_count: (2, 4),
            secondary_radius: 100.0,
        };

        let objects = place_clumps(&noise, &config, 2560.0, 2560.0, &alloc, &mut rng);
        assert!(!objects.is_empty());

        let rocks: Vec<_> = objects.iter().filter(|o| o.texture == "rock.png").collect();
        let pebbles: Vec<_> = objects.iter().filter(|o| o.texture == "pebble.png").collect();
        assert!(!rocks.is_empty());
        assert!(pebbles.len() >= rocks.len() * 2);
    }

    #[test]
    fn test_clear_corridor() {
        let alloc = NodeIdAllocator::new(1);
        let mut objects = vec![
            MapObject::new("tree.png", Vector2::new(100.0, 100.0), &alloc.next()),
            MapObject::new("tree.png", Vector2::new(500.0, 500.0), &alloc.next()),
            MapObject::new("tree.png", Vector2::new(1000.0, 1000.0), &alloc.next()),
        ];

        // Corridor through (100, 100) with radius 50
        clear_corridor(&mut objects, &[(100.0, 100.0)], 50.0);
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].position.x, 500.0);
    }

    #[test]
    fn test_custom_color() {
        let noise = test_noise();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let config = ObjectConfig {
            textures: vec!["grass.png".to_string()],
            min_distance: 200.0,
            custom_color: Some("ff00ff00".to_string()),
            ..Default::default()
        };

        let objects = place_objects(&noise, &config, 2560.0, 2560.0, &alloc, &mut rng);
        assert!(!objects.is_empty());
        for obj in &objects {
            assert_eq!(obj.custom_color.as_deref(), Some("ff00ff00"));
        }
    }

    #[test]
    fn test_unique_node_refs() {
        let noise = test_noise();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = ObjectConfig {
            textures: vec!["tree.png".to_string()],
            min_distance: 200.0,
            ..Default::default()
        };

        let objects = place_objects(&noise, &config, 2560.0, 2560.0, &alloc, &mut rng);
        let refs: std::collections::HashSet<_> = objects.iter().map(|o| &o.node_ref).collect();
        assert_eq!(refs.len(), objects.len(), "All node refs must be unique");
    }
}
