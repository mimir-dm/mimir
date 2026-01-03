fn main() {
    // Declare custom cfg flags to avoid unexpected_cfgs warnings
    println!("cargo::rustc-check-cfg=cfg(has_dev_phb)");
    println!("cargo::rustc-check-cfg=cfg(has_dev_mm)");
    println!("cargo::rustc-check-cfg=cfg(has_dev_dmg)");
    println!("cargo::rustc-check-cfg=cfg(has_dev_tokens)");

    // Check which dev test books exist and set cfg flags
    // This allows conditional compilation to only include files that exist
    let dev_files = [
        ("phb", "assets/dev/phb.tar.gz"),
        ("mm", "assets/dev/mm.tar.gz"),
        ("dmg", "assets/dev/dmg.tar.gz"),
        ("tokens", "assets/dev/dev-tokens.tar.gz"),
    ];

    for (name, path) in &dev_files {
        if std::path::Path::new(path).exists() {
            println!("cargo:rustc-cfg=has_dev_{}", name);
        }
    }

    tauri_build::build()
}
