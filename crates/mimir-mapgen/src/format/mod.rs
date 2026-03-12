//! Dungeondraft `.dungeondraft_map` format types and serialization.
//!
//! Provides type-safe Rust structs for the DD map JSON format, including
//! custom serde (de)serialization for Godot types (`Vector2`, `PoolByteArray`).

pub mod entities;
pub mod godot_types;
pub mod header;
pub mod world;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub use entities::*;
pub use godot_types::*;
pub use header::*;
pub use world::*;

/// Top-level `.dungeondraft_map` file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeondraftMap {
    pub header: Header,
    pub world: World,
    /// Mod data section — required by Dungeondraft even if empty.
    #[serde(rename = "mod", default)]
    pub mod_data: ModData,
}

/// Mod data section at the root of a `.dungeondraft_map` file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModData {
    #[serde(rename = ".node_table", default)]
    pub node_table: BTreeMap<String, serde_json::Value>,
}

impl DungeondraftMap {
    /// Create a new empty map with the given dimensions (in grid squares).
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            header: Header::new(true),
            world: World::new(width, height),
            mod_data: ModData::default(),
        }
    }

    /// Parse a `.dungeondraft_map` file from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get a mutable reference to the ground level (level "0").
    pub fn ground_level_mut(&mut self) -> &mut Level {
        self.world
            .levels
            .entry("0".to_string())
            .or_insert_with(|| Level::new_ground(self.world.width, self.world.height))
    }

    /// Get a reference to the ground level.
    pub fn ground_level(&self) -> Option<&Level> {
        self.world.levels.get("0")
    }
}

/// Allocator for unique node reference IDs.
///
/// Dungeondraft assigns incrementing IDs to all entities. This allocator
/// provides thread-safe unique IDs across all entity types.
#[derive(Debug)]
pub struct NodeIdAllocator {
    next_id: AtomicU64,
}

impl NodeIdAllocator {
    pub fn new(start: u64) -> Self {
        Self {
            next_id: AtomicU64::new(start),
        }
    }

    /// Allocate the next unique node ID as a hex string.
    pub fn next(&self) -> String {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        format!("{:x}", id)
    }

    /// Get the current next_node_id value as a hex string (for writing to the world).
    pub fn current(&self) -> String {
        let id = self.next_id.load(Ordering::Relaxed);
        format!("{:x}", id)
    }
}

impl Default for NodeIdAllocator {
    fn default() -> Self {
        Self::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map_serializes() {
        let map = DungeondraftMap::new(32, 32);
        let json = map.to_json().unwrap();
        assert!(json.contains("\"format\": 3"));
        assert!(json.contains("\"width\": 32"));
        assert!(json.contains("\"height\": 32"));
        assert!(json.contains("\"Ground\""));
    }

    #[test]
    fn test_new_map_roundtrip() {
        let map = DungeondraftMap::new(10, 10);
        let json = map.to_json().unwrap();
        let parsed = DungeondraftMap::from_json(&json).unwrap();
        assert_eq!(parsed.world.width, 10);
        assert_eq!(parsed.world.height, 10);
        assert_eq!(parsed.world.format, 3);
    }

    #[test]
    fn test_node_id_allocator() {
        let alloc = NodeIdAllocator::new(100);
        assert_eq!(alloc.next(), "64");  // 100 decimal = 0x64
        assert_eq!(alloc.next(), "65");
        assert_eq!(alloc.next(), "66");
        assert_eq!(alloc.current(), "67");
    }

    #[test]
    fn test_terrain_splat_size() {
        let terrain = Terrain::new_uniform(
            10,
            10,
            [
                "res://textures/terrain/terrain_dirt.png".to_string(),
                "res://textures/terrain/terrain_dry_grass.png".to_string(),
                "res://textures/terrain/terrain_moss.png".to_string(),
                "res://textures/terrain/terrain_gravel.png".to_string(),
            ],
        );
        // 10*4 * 10*4 cells = 1600 cells, 4 bytes each = 6400
        assert_eq!(terrain.splat.0.len(), 6400);
    }

    #[test]
    fn test_map_object_builder() {
        let obj = MapObject::new(
            "res://textures/objects/tree.png",
            Vector2::new(100.0, 200.0),
            "1",
        )
        .with_scale(1.5)
        .with_rotation(1.57)
        .with_layer(200)
        .with_mirror(true);

        assert_eq!(obj.scale, Vector2::new(1.5, 1.5));
        assert_eq!(obj.rotation, 1.57);
        assert_eq!(obj.layer, 200);
        assert!(obj.mirror);
    }
}
