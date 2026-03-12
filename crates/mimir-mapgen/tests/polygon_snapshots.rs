//! Polygon layout snapshot tests.
//!
//! Each test generates a map from a seeded YAML fixture, then compares the
//! `world` section byte-for-byte against a checked-in baseline. This catches
//! regressions in wall geometry, portal placement, shape fill, and terrain
//! splat maps — essentially a pixel test for the DD output format.
//!
//! To update baselines after intentional changes:
//!   cargo run -p mimir-mapgen -- generate <fixture>.yaml -o <fixture>.dungeondraft_map

use mimir_mapgen::pipeline;

fn fixture(name: &str) -> String {
    format!(
        "{}/tests/fixtures/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    )
}

fn load_config(yaml_name: &str) -> pipeline::MapConfig {
    let path = fixture(yaml_name);
    let yaml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));
    serde_yaml::from_str(&yaml)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path, e))
}

fn assert_world_matches(yaml_name: &str, baseline_name: &str) {
    let config = load_config(yaml_name);
    let result = pipeline::generate(&config, None);
    let generated_json = result.map.to_json().expect("Failed to serialize generated map");
    let generated: serde_json::Value =
        serde_json::from_str(&generated_json).expect("Failed to parse generated JSON");

    let baseline_path = fixture(baseline_name);
    let baseline_json = std::fs::read_to_string(&baseline_path)
        .unwrap_or_else(|e| panic!("Failed to read baseline {}: {}", baseline_path, e));
    let baseline: serde_json::Value =
        serde_json::from_str(&baseline_json).expect("Failed to parse baseline JSON");

    assert_eq!(
        generated["world"], baseline["world"],
        "World section mismatch for {}. Re-run generation to update baseline.",
        yaml_name
    );
}

#[test]
fn snapshot_single_polygon() {
    assert_world_matches("polygon_single.yaml", "polygon_single.dungeondraft_map");
}

#[test]
fn snapshot_two_adjacent_rooms() {
    assert_world_matches(
        "polygon_two_adjacent.yaml",
        "polygon_two_adjacent.dungeondraft_map",
    );
}

#[test]
fn snapshot_three_rooms_two_corridors() {
    assert_world_matches(
        "polygon_three_rooms.yaml",
        "polygon_three_rooms.dungeondraft_map",
    );
}

#[test]
fn snapshot_overlapping_ovals() {
    assert_world_matches(
        "polygon_overlapping_ovals.yaml",
        "polygon_overlapping_ovals.dungeondraft_map",
    );
}
