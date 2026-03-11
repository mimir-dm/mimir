//! Round-trip test: parse baseline.dungeondraft_map → serialize → parse again.

use mimir_mapgen::format::DungeondraftMap;

#[test]
fn test_baseline_map_parse() {
    let json = include_str!("fixtures/baseline.dungeondraft_map");
    let map = DungeondraftMap::from_json(json).expect("Failed to parse baseline map");

    assert_eq!(map.world.width, 32);
    assert_eq!(map.world.height, 32);
    assert_eq!(map.world.format, 3);
    assert!(map.header.uses_default_assets);
    assert_eq!(map.header.asset_manifest.len(), 2);

    let level = map.ground_level().expect("No ground level");
    assert_eq!(level.label, "Ground");

    let terrain = level.terrain.as_ref().expect("No terrain");
    assert!(terrain.enabled);
    assert_eq!(terrain.texture_1, "res://textures/terrain/terrain_dirt.png");
    // Splat: 32*4 * 32*4 = 16384 cells * 4 bytes = 65536
    assert_eq!(terrain.splat.0.len(), 65536);
}

#[test]
fn test_baseline_map_roundtrip() {
    let json = include_str!("fixtures/baseline.dungeondraft_map");
    let map = DungeondraftMap::from_json(json).expect("Failed to parse baseline map");
    let serialized = map.to_json().expect("Failed to serialize");
    let reparsed = DungeondraftMap::from_json(&serialized).expect("Failed to reparse");

    assert_eq!(reparsed.world.width, map.world.width);
    assert_eq!(reparsed.world.height, map.world.height);
    assert_eq!(reparsed.world.format, map.world.format);

    let orig_terrain = map.ground_level().unwrap().terrain.as_ref().unwrap();
    let new_terrain = reparsed.ground_level().unwrap().terrain.as_ref().unwrap();
    assert_eq!(orig_terrain.splat.0.len(), new_terrain.splat.0.len());
    assert_eq!(orig_terrain.texture_1, new_terrain.texture_1);
}
