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

/// One Rust version (MIMIR-T-0748): the workflows and the Dockerfile use the
/// channel of `rust-toolchain.toml`.
#[test]
fn the_rust_pins_agree() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let channel = read("rust-toolchain.toml")
        .lines()
        .find_map(|l| l.strip_prefix("channel = \""))
        .and_then(|v| v.strip_suffix('"'))
        .expect("rust-toolchain.toml has a channel")
        .to_string();
    for wf in [".github/workflows/ci.yml", ".github/workflows/release.yml"] {
        let text = read(wf);
        let pins: Vec<&str> = text
            .lines()
            .filter_map(|l| l.split("dtolnay/rust-toolchain@").nth(1))
            .map(str::trim)
            .collect();
        assert!(!pins.is_empty(), "{wf} installs no Rust");
        assert!(
            pins.iter().all(|p| *p == channel),
            "{wf} pins {pins:?}, rust-toolchain.toml says {channel}"
        );
    }
    let minor = channel.rsplitn(2, '.').nth(1).unwrap();
    let docker = read("Dockerfile");
    let from = docker
        .lines()
        .find(|l| l.starts_with("FROM rust:"))
        .expect("the Dockerfile builds FROM rust");
    assert!(
        from.starts_with(&format!("FROM rust:{minor}-"))
            || from.starts_with(&format!("FROM rust:{channel}-")),
        "Dockerfile: {from}, rust-toolchain.toml says {channel}"
    );
}
