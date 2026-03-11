//! CLI integration tests for mimir-mapgen binary.

use std::process::Command;

fn mapgen_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mimir-mapgen"))
}

#[test]
fn test_list_presets() {
    let output = mapgen_bin().arg("list-presets").output().unwrap();

    assert!(output.status.success(), "list-presets should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("forest"));
    assert!(stdout.contains("grassland"));
    assert!(stdout.contains("cave"));
}

#[test]
fn test_generate_from_preset() {
    let dir = tempfile::tempdir().unwrap();
    let output_path = dir.path().join("test.dungeondraft_map");

    let output = mapgen_bin()
        .args([
            "generate",
            "--preset",
            "forest",
            "--seed",
            "42",
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "generate from preset should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists(), "Output file should exist");

    // Verify it's valid JSON that parses as a DungeondraftMap
    let contents = std::fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert!(parsed.get("header").is_some(), "Should have header");
    assert!(parsed.get("world").is_some(), "Should have world");
}

#[test]
fn test_generate_from_yaml_config() {
    let dir = tempfile::tempdir().unwrap();
    let output_path = dir.path().join("test.dungeondraft_map");
    let config_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sample_config.yaml"
    );

    let output = mapgen_bin()
        .args([
            "generate",
            config_path,
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "generate from YAML should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists());

    let contents = std::fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert!(parsed.get("header").is_some());
    assert!(parsed.get("world").is_some());
}

#[test]
fn test_generate_deterministic_output() {
    let dir = tempfile::tempdir().unwrap();
    let out1 = dir.path().join("map1.dungeondraft_map");
    let out2 = dir.path().join("map2.dungeondraft_map");

    for out in [&out1, &out2] {
        let output = mapgen_bin()
            .args([
                "generate",
                "--preset",
                "grassland",
                "--seed",
                "12345",
                "-o",
                out.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    // Compare world sections (header has timestamps)
    let c1: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out1).unwrap()).unwrap();
    let c2: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out2).unwrap()).unwrap();
    assert_eq!(c1["world"], c2["world"], "Same seed should produce identical world");
}

#[test]
fn test_validate_valid_config() {
    let config_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sample_config.yaml"
    );

    let output = mapgen_bin()
        .args(["validate", config_path])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid"));
}

#[test]
fn test_generate_no_args_fails() {
    let output = mapgen_bin().arg("generate").output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn test_generate_both_config_and_preset_fails() {
    let config_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sample_config.yaml"
    );

    let output = mapgen_bin()
        .args(["generate", config_path, "--preset", "forest"])
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_generate_unknown_preset_fails() {
    let output = mapgen_bin()
        .args(["generate", "--preset", "nonexistent"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown preset"));
}
