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

#[test]
fn test_wall_portal_roundtrip() {
    // Parse the three-rooms polygon fixture (1 merged wall, 1 wall-anchored + 4 freestanding portals)
    let json = include_str!("fixtures/polygon_three_rooms.dungeondraft_map");
    let map = DungeondraftMap::from_json(json).expect("Failed to parse three-rooms map");

    let level = map.ground_level().expect("No ground level");

    // Merged perimeter: 1 closed-loop wall
    assert_eq!(level.walls.len(), 1, "Expected 1 merged wall");
    assert!(level.walls[0].is_loop, "Wall 0 should be a closed loop");
    assert_eq!(level.walls[0].wall_type, 0);
    assert_eq!(
        level.walls[0].texture,
        "res://textures/walls/stone.png"
    );
    assert!(
        !level.walls[0].points.0.is_empty(),
        "Wall 0 should have points"
    );

    // 1 wall-anchored portal (window on room_b north wall)
    assert_eq!(
        level.walls[0].portals.len(),
        1,
        "Wall 0 should have 1 wall-anchored portal"
    );
    let window = &level.walls[0].portals[0];
    assert_eq!(window.wall_id, level.walls[0].node_id);

    // 4 freestanding portals (doors on shared corridor edges)
    assert_eq!(
        level.portals.len(),
        4,
        "Level should have 4 freestanding portals"
    );
    for portal in &level.portals {
        assert_eq!(portal.wall_id, "ffffffff");
    }

    // Round-trip: serialize and reparse
    let serialized = map.to_json().expect("Failed to serialize");
    let reparsed =
        DungeondraftMap::from_json(&serialized).expect("Failed to reparse after round-trip");
    let rlevel = reparsed.ground_level().unwrap();

    assert_eq!(rlevel.walls.len(), level.walls.len());
    assert_eq!(rlevel.walls[0].portals.len(), level.walls[0].portals.len());
    assert_eq!(rlevel.portals.len(), level.portals.len());
    assert_eq!(
        rlevel.walls[0].points.0.len(),
        level.walls[0].points.0.len()
    );
}

#[test]
fn test_wall_builder() {
    use mimir_mapgen::format::godot_types::Vector2;
    use mimir_mapgen::format::entities::{MapWall, MapPortal};

    // Build a simple rectangular room wall
    let points = vec![
        Vector2::new(1024.0, 1536.0),  // (4, 6) * 256
        Vector2::new(2304.0, 1536.0),  // (9, 6) * 256
        Vector2::new(2304.0, 2560.0),  // (9, 10) * 256
        Vector2::new(1024.0, 2560.0),  // (4, 10) * 256
    ];

    let portal = MapPortal::new(
        Vector2::new(1664.0, 1536.0),  // center of north wall
        0.0,                            // horizontal wall
        Vector2::new(0.0, -1.0),        // pointing north
        "res://textures/portals/door_00.png",
        128.0,                          // 1 grid square door
        0,                              // first segment
        "a",                            // parent wall ID
        0.5,                            // middle of segment
        "b",                            // portal ID
    );

    let wall = MapWall::new_room(points, "res://textures/walls/battlements.png", "a")
        .with_portals(vec![portal]);

    assert!(wall.is_loop);
    assert_eq!(wall.wall_type, 0);
    assert_eq!(wall.portals.len(), 1);
    assert_eq!(wall.portals[0].radius, 128.0);
    assert_eq!(wall.portals[0].wall_id, "a");

    // Verify it serializes to valid JSON
    let json = serde_json::to_string(&wall).unwrap();
    assert!(json.contains("\"loop\":true"));
    assert!(json.contains("\"type\":0"));
    assert!(json.contains("door_00"));

    // Verify round-trip
    let reparsed: MapWall = serde_json::from_str(&json).unwrap();
    assert_eq!(reparsed.portals.len(), 1);
    assert_eq!(reparsed.node_id, "a");
}
