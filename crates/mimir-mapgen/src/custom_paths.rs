//! General-purpose path generation for Dungeondraft maps.
//!
//! Four styles beyond the existing road/river generators:
//! - **Waypoints**: explicit user-defined points with optional Bezier smoothing.
//! - **RoomToRoom**: connect two rooms by name via their centers.
//! - **Offset**: companion path at perpendicular distance from a parent feature.
//! - **Intermittent**: break a parent path into segments with random gaps.

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::curves;
use crate::format::entities::MapPath;
use crate::format::godot_types::Vector2;
use crate::format::NodeIdAllocator;
use crate::pipeline::GeneratedFeatures;

/// Configuration for a custom path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPathConfig {
    /// Path style.
    pub style: PathStyle,
    /// Path texture.
    pub texture: String,
    /// Path width in grid squares.
    pub width: f64,
    /// Path color (ARGB hex).
    #[serde(default = "default_color")]
    pub color: String,
    /// DD layer.
    #[serde(default = "default_layer")]
    pub layer: i32,
    /// Bezier smoothing density (0 = no smoothing).
    #[serde(default = "default_smooth")]
    pub smooth: usize,
    /// Whether path forms a closed loop.
    #[serde(default)]
    pub loop_path: bool,
}

fn default_color() -> String {
    "ffffffff".to_string()
}

fn default_layer() -> i32 {
    100
}

fn default_smooth() -> usize {
    8
}

/// Path generation style.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PathStyle {
    /// Explicit waypoints in grid coordinates.
    Waypoints {
        /// Points as [x, y] in grid squares.
        points: Vec<[f64; 2]>,
    },
    /// Connect two rooms by name via their centers.
    RoomToRoom {
        /// Source room ID.
        from: String,
        /// Destination room ID.
        to: String,
    },
    /// Companion path offset from a named parent feature.
    Offset {
        /// Named feature to offset from (road, river, elevation, etc.).
        along: String,
        /// Perpendicular offset distance in grid squares (positive = left).
        offset: f64,
        /// Reverse the path direction.
        #[serde(default)]
        reverse: bool,
    },
    /// Break a parent path into segments with random gaps.
    Intermittent {
        /// Named feature to follow.
        along: String,
        /// Average segment length in grid squares.
        segment_length: f64,
        /// Variation in segment length (±).
        #[serde(default)]
        segment_variation: f64,
        /// Maximum gap between segments in grid squares.
        gap: f64,
        /// Perpendicular offset from parent path in grid squares.
        #[serde(default)]
        offset: f64,
        /// Reverse the path direction.
        #[serde(default)]
        reverse: bool,
    },
}

/// Generate custom paths from configs.
pub fn generate_custom_paths(
    configs: &[CustomPathConfig],
    features: &GeneratedFeatures,
    alloc: &NodeIdAllocator,
    rng: &mut ChaCha8Rng,
) -> Vec<MapPath> {
    let mut paths = Vec::new();

    for config in configs {
        match &config.style {
            PathStyle::Waypoints { points } => {
                if points.len() < 2 {
                    continue;
                }
                // Convert grid coords to pixels
                let mut pixel_points: Vec<(f64, f64)> = points
                    .iter()
                    .map(|p| (p[0] * 256.0, p[1] * 256.0))
                    .collect();

                // Apply smoothing
                if config.smooth > 0 && pixel_points.len() >= 3 {
                    pixel_points = curves::bezier_smooth(&pixel_points, config.smooth);
                }

                let vectors: Vec<Vector2> = pixel_points
                    .iter()
                    .map(|&(x, y)| Vector2::new(x, y))
                    .collect();

                let path = MapPath::new(&config.texture, vectors, config.width * 256.0, &alloc.next())
                    .with_layer(config.layer)
                    .with_loop(config.loop_path);
                paths.push(path);
            }

            PathStyle::RoomToRoom { from, to } => {
                let from_room = features.get_room(from);
                let to_room = features.get_room(to);

                if let (Some(fr), Some(tr)) = (from_room, to_room) {
                    let mut pixel_points = vec![fr.center, tr.center];

                    if config.smooth > 0 && pixel_points.len() >= 3 {
                        pixel_points = curves::bezier_smooth(&pixel_points, config.smooth);
                    }

                    let vectors: Vec<Vector2> = pixel_points
                        .iter()
                        .map(|&(x, y)| Vector2::new(x, y))
                        .collect();

                    let path = MapPath::new(&config.texture, vectors, config.width * 256.0, &alloc.next())
                        .with_layer(config.layer);
                    paths.push(path);
                }
            }

            PathStyle::Offset {
                along,
                offset,
                reverse,
            } => {
                if let Some(parent_points) = features.get_path(along) {
                    let mut points = parent_points.clone();
                    if *reverse {
                        points.reverse();
                    }

                    let offset_px = offset * 256.0;
                    let offset_points = curves::offset_polyline(&points, offset_px);

                    if offset_points.len() >= 2 {
                        let vectors: Vec<Vector2> = offset_points
                            .iter()
                            .map(|&(x, y)| Vector2::new(x, y))
                            .collect();

                        let path = MapPath::new(&config.texture, vectors, config.width * 256.0, &alloc.next())
                            .with_layer(config.layer);
                        paths.push(path);
                    }
                }
            }

            PathStyle::Intermittent {
                along,
                segment_length,
                segment_variation,
                gap,
                offset,
                reverse,
            } => {
                if let Some(parent_points) = features.get_path(along) {
                    let mut points = parent_points.clone();
                    if *reverse {
                        points.reverse();
                    }

                    // Apply offset if non-zero
                    let working_points = if offset.abs() > f64::EPSILON {
                        curves::offset_polyline(&points, offset * 256.0)
                    } else {
                        points
                    };

                    // Break into segments
                    let segments = break_into_segments(
                        &working_points,
                        *segment_length * 256.0,
                        *segment_variation * 256.0,
                        *gap * 256.0,
                        rng,
                    );

                    for segment in segments {
                        if segment.len() >= 2 {
                            let vectors: Vec<Vector2> = segment
                                .iter()
                                .map(|&(x, y)| Vector2::new(x, y))
                                .collect();

                            let path = MapPath::new(
                                &config.texture,
                                vectors,
                                config.width * 256.0,
                                &alloc.next(),
                            )
                            .with_layer(config.layer);
                            paths.push(path);
                        }
                    }
                }
            }
        }
    }

    paths
}

/// Break a polyline into intermittent segments with gaps.
fn break_into_segments(
    points: &[(f64, f64)],
    segment_length: f64,
    variation: f64,
    gap: f64,
    rng: &mut ChaCha8Rng,
) -> Vec<Vec<(f64, f64)>> {
    if points.len() < 2 {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut current_segment = Vec::new();
    let mut distance_in_segment = 0.0;
    let mut target_length = segment_length + rng.gen_range(-variation..variation);
    let mut in_gap = false;
    let mut gap_remaining = 0.0;

    // Walk along the polyline
    current_segment.push(points[0]);

    for window in points.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        let dx = x1 - x0;
        let dy = y1 - y0;
        let seg_len = (dx * dx + dy * dy).sqrt();

        if seg_len < f64::EPSILON {
            continue;
        }

        let mut consumed = 0.0;

        while consumed < seg_len {
            let remaining_in_seg = seg_len - consumed;

            if in_gap {
                if gap_remaining <= remaining_in_seg {
                    consumed += gap_remaining;
                    gap_remaining = 0.0;
                    in_gap = false;
                    // Start new segment
                    let t = consumed / seg_len;
                    let px = x0 + dx * t;
                    let py = y0 + dy * t;
                    current_segment.push((px, py));
                    target_length = segment_length + rng.gen_range(-variation..variation);
                    distance_in_segment = 0.0;
                } else {
                    gap_remaining -= remaining_in_seg;
                    consumed = seg_len;
                }
            } else {
                let need = target_length - distance_in_segment;
                if need <= remaining_in_seg {
                    consumed += need;
                    let t = consumed / seg_len;
                    let px = x0 + dx * t;
                    let py = y0 + dy * t;
                    current_segment.push((px, py));
                    // End segment, start gap
                    if current_segment.len() >= 2 {
                        segments.push(std::mem::take(&mut current_segment));
                    } else {
                        current_segment.clear();
                    }
                    in_gap = true;
                    gap_remaining = rng.gen_range(0.0..gap);
                } else {
                    distance_in_segment += remaining_in_seg;
                    consumed = seg_len;
                }
            }
        }

        if !in_gap && !current_segment.is_empty() {
            // Add endpoint if we're mid-segment
            if *current_segment.last().unwrap() != window[1] {
                current_segment.push(window[1]);
            }
        }
    }

    // Flush final segment
    if current_segment.len() >= 2 {
        segments.push(current_segment);
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waypoints_basic() {
        use rand::SeedableRng;
        let features = GeneratedFeatures::default();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let configs = vec![CustomPathConfig {
            style: PathStyle::Waypoints {
                points: vec![[5.0, 10.0], [15.0, 10.0], [25.0, 15.0]],
            },
            texture: "res://textures/paths/cobblestone.png".to_string(),
            width: 1.0,
            color: default_color(),
            layer: 100,
            smooth: 0,
            loop_path: false,
        }];

        let paths = generate_custom_paths(&configs, &features, &alloc, &mut rng);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].texture, "res://textures/paths/cobblestone.png");
    }

    #[test]
    fn test_waypoints_with_smoothing() {
        use rand::SeedableRng;
        let features = GeneratedFeatures::default();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let configs = vec![CustomPathConfig {
            style: PathStyle::Waypoints {
                points: vec![[0.0, 0.0], [5.0, 5.0], [10.0, 0.0]],
            },
            texture: "test.png".to_string(),
            width: 1.0,
            color: default_color(),
            layer: 100,
            smooth: 8,
            loop_path: false,
        }];

        let paths = generate_custom_paths(&configs, &features, &alloc, &mut rng);
        assert_eq!(paths.len(), 1);
        // Smoothed path should have more points than the 3 input points
        assert!(paths[0].edit_points.0.len() > 3);
    }

    #[test]
    fn test_room_to_room() {
        use rand::SeedableRng;
        use crate::pipeline::RoomGeometry;

        let mut features = GeneratedFeatures::default();
        features.rooms.insert("hall".to_string(), RoomGeometry {
            center: (1280.0, 1280.0),
            boundary: vec![(1024.0, 1024.0), (1536.0, 1024.0), (1536.0, 1536.0), (1024.0, 1536.0)],
        });
        features.rooms.insert("treasury".to_string(), RoomGeometry {
            center: (3840.0, 1280.0),
            boundary: vec![(3584.0, 1024.0), (4096.0, 1024.0), (4096.0, 1536.0), (3584.0, 1536.0)],
        });
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let configs = vec![CustomPathConfig {
            style: PathStyle::RoomToRoom {
                from: "hall".to_string(),
                to: "treasury".to_string(),
            },
            texture: "res://textures/paths/cobblestone.png".to_string(),
            width: 1.5,
            color: default_color(),
            layer: 100,
            smooth: 0,
            loop_path: false,
        }];

        let paths = generate_custom_paths(&configs, &features, &alloc, &mut rng);
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn test_offset_path() {
        use rand::SeedableRng;
        let mut features = GeneratedFeatures::default();
        features.paths.insert(
            "cliff".to_string(),
            vec![(0.0, 128.0), (256.0, 128.0), (512.0, 128.0)],
        );
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let configs = vec![CustomPathConfig {
            style: PathStyle::Offset {
                along: "cliff".to_string(),
                offset: 0.5, // half grid square
                reverse: false,
            },
            texture: "res://textures/paths/shadow.png".to_string(),
            width: 2.0,
            color: default_color(),
            layer: 300,
            smooth: 0,
            loop_path: false,
        }];

        let paths = generate_custom_paths(&configs, &features, &alloc, &mut rng);
        assert_eq!(paths.len(), 1);
        // Offset path should be shifted perpendicular
        let first_y = paths[0].position.y + paths[0].edit_points.0[0].y;
        assert!((first_y - (128.0 + 128.0)).abs() < 10.0 || (first_y - (128.0 - 128.0)).abs() < 10.0,
            "Offset path should be shifted from parent, got y={}", first_y);
    }

    #[test]
    fn test_intermittent_path() {
        use rand::SeedableRng;
        let mut features = GeneratedFeatures::default();
        // Long straight path: 20 grid squares
        features.paths.insert(
            "river".to_string(),
            vec![(0.0, 128.0), (5120.0, 128.0)],
        );
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let configs = vec![CustomPathConfig {
            style: PathStyle::Intermittent {
                along: "river".to_string(),
                segment_length: 3.0,
                segment_variation: 1.0,
                gap: 1.0,
                offset: 0.0,
                reverse: false,
            },
            texture: "res://textures/paths/water_flow.png".to_string(),
            width: 0.5,
            color: default_color(),
            layer: 100,
            smooth: 0,
            loop_path: false,
        }];

        let paths = generate_custom_paths(&configs, &features, &alloc, &mut rng);
        assert!(paths.len() >= 2, "Should produce multiple segments, got {}", paths.len());
    }

    #[test]
    fn test_missing_reference() {
        use rand::SeedableRng;
        let features = GeneratedFeatures::default();
        let alloc = NodeIdAllocator::new(1);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let configs = vec![
            CustomPathConfig {
                style: PathStyle::RoomToRoom {
                    from: "nope".to_string(),
                    to: "also_nope".to_string(),
                },
                texture: "test.png".to_string(),
                width: 1.0,
                color: default_color(),
                layer: 100,
                smooth: 0,
                loop_path: false,
            },
            CustomPathConfig {
                style: PathStyle::Offset {
                    along: "nope".to_string(),
                    offset: 1.0,
                    reverse: false,
                },
                texture: "test.png".to_string(),
                width: 1.0,
                color: default_color(),
                layer: 100,
                smooth: 0,
                loop_path: false,
            },
        ];

        let paths = generate_custom_paths(&configs, &features, &alloc, &mut rng);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_break_into_segments() {
        use rand::SeedableRng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // 10 grid squares straight line
        let points = vec![(0.0, 0.0), (2560.0, 0.0)];
        let segments = break_into_segments(
            &points,
            768.0,  // ~3 grid squares per segment
            128.0,  // ± half grid square variation
            256.0,  // up to 1 grid square gap
            &mut rng,
        );
        assert!(segments.len() >= 2, "Should have multiple segments, got {}", segments.len());
        for seg in &segments {
            assert!(seg.len() >= 2, "Each segment should have at least 2 points");
        }
    }
}
