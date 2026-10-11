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
