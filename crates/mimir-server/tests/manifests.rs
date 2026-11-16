//! The crates of the workspace carry one version (one bump per release),
//! and the release workflow tags that version: the manifests must agree.
//! No version is pinned here.

#[test]
fn every_crate_has_the_same_version() {
    let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut versions = Vec::new();
    for entry in std::fs::read_dir(&crates).unwrap().flatten() {
        let manifest = entry.path().join("Cargo.toml");
        let Ok(text) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        let version = text
            .lines()
            .find_map(|l| l.strip_prefix("version = \""))
            .and_then(|v| v.strip_suffix('"'))
            .unwrap_or_else(|| panic!("{} has no version", manifest.display()))
            .to_string();
        versions.push((entry.file_name().to_string_lossy().into_owned(), version));
    }
    assert!(versions.len() >= 8, "{versions:?}");
    let first = &versions[0].1;
    assert!(
        versions.iter().all(|(_, v)| v == first),
        "the crates disagree: {versions:?}"
    );
    assert_eq!(env!("CARGO_PKG_VERSION"), first.as_str());
}

/// The Claude Code plugin and its marketplace entry carry the same version
/// (MIMIR-T-0747): Claude Code keeps a plugin per version, so a plugin that
/// keeps an old version is not updated.
#[test]
fn the_plugin_manifests_have_the_version_of_the_crates() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(root.join(p)).unwrap()).unwrap()
    };
    let version = env!("CARGO_PKG_VERSION");
    let plugin = read("crates/mimir-mcp/plugin/.claude-plugin/plugin.json");
    assert_eq!(plugin["version"], version, "plugin.json");
    let market = read(".claude-plugin/marketplace.json");
    assert_eq!(
        market["metadata"]["version"], version,
        "marketplace.json metadata"
    );
    for p in market["plugins"].as_array().unwrap() {
        assert_eq!(
            p["version"], version,
            "marketplace.json plugin {}",
            p["name"]
        );
    }
}
