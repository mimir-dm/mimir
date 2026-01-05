//! Map and token seeding for dev data.
//!
//! Map assets are loaded from disk at runtime (not embedded in binary).
//! Set MIMIR_DEV_ASSETS env var to override the path.

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::campaign::light_sources::NewLightSource;
use crate::models::campaign::modules::Module;
use crate::models::campaign::{GridType, NewMap, NewToken, TokenSize, TokenType};
use crate::models::character::Character;
use crate::services::{LightSourceService, MapService, TokenService};
use std::path::PathBuf;
use tracing::{info, warn};

// Map dimensions (these are constants, not the actual data)
const GOBLIN_HIDEOUT_WIDTH: i32 = 2592;
const GOBLIN_HIDEOUT_HEIGHT: i32 = 1458;
const GOBLIN_HIDEOUT_GRID_PX: i32 = 54;

const GOBLIN_REGION_WIDTH: i32 = 1792;
const GOBLIN_REGION_HEIGHT: i32 = 1024;

/// Get the seed assets directory path
fn get_seed_assets_dir() -> Option<PathBuf> {
    // 1. Check env var override
    if let Ok(path) = std::env::var("MIMIR_DEV_ASSETS") {
        // If it's a dev assets dir, look for seed assets relative to repo
        let p = PathBuf::from(&path);
        if let Some(repo_root) = p.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
            let seed_assets = repo_root.join("crates/mimir-dm-core/src/seed/assets");
            if seed_assets.exists() {
                return Some(seed_assets);
            }
        }
    }

    // 2. Try to find repo root from executable location
    if let Ok(exe) = std::env::current_exe() {
        let mut current = exe.parent();
        while let Some(dir) = current {
            let cargo_toml = dir.join("Cargo.toml");
            if cargo_toml.exists() {
                let assets_dir = dir.join("crates/mimir-dm-core/src/seed/assets");
                if assets_dir.exists() {
                    return Some(assets_dir);
                }
            }
            current = dir.parent();
        }
    }

    // 3. Try relative to current working directory
    let cwd_assets = PathBuf::from("crates/mimir-dm-core/src/seed/assets");
    if cwd_assets.exists() {
        return Some(cwd_assets);
    }

    None
}

/// Load a seed asset file from disk
fn load_seed_asset(filename: &str) -> Option<Vec<u8>> {
    let assets_dir = get_seed_assets_dir()?;
    let path = assets_dir.join(filename);

    match std::fs::read(&path) {
        Ok(data) => {
            info!("Loaded seed asset {} ({} bytes)", filename, data.len());
            Some(data)
        }
        Err(e) => {
            warn!("Failed to load seed asset {}: {}", filename, e);
            None
        }
    }
}

/// Seed maps and tokens.
pub fn seed(
    conn: &mut DbConnection,
    campaign_id: i32,
    modules: &[Module],
    characters: &[Character],
    data_dir: &str,
) -> Result<()> {
    seed_battle_map(conn, campaign_id, modules, characters, data_dir)?;
    seed_region_map(conn, campaign_id, data_dir)?;
    Ok(())
}

fn seed_battle_map(
    conn: &mut DbConnection,
    campaign_id: i32,
    modules: &[Module],
    characters: &[Character],
    data_dir: &str,
) -> Result<()> {
    // Load map asset from disk
    let Some(uvtt_data) = load_seed_asset("goblin-hideout.dd2vtt") else {
        warn!("Skipping battle map - asset not found. Set MIMIR_DEV_ASSETS or run from repo root.");
        return Ok(());
    };

    let module = modules.iter().find(|m| m.name == "Cragmaw Hideout");
    let module_id = module.map(|m| m.id);

    // Write UVTT file
    let maps_dir = if let Some(mid) = module_id {
        PathBuf::from(data_dir).join("modules").join(mid.to_string()).join("maps")
    } else {
        PathBuf::from(data_dir).join("campaigns").join(campaign_id.to_string()).join("maps")
    };
    std::fs::create_dir_all(&maps_dir)?;

    let filename = "goblin-hideout.dd2vtt";
    std::fs::write(maps_dir.join(filename), &uvtt_data)?;

    // Create map record
    let mut new_map = NewMap::new(
        campaign_id,
        "Goblin Hideout".into(),
        filename.into(),
        GOBLIN_HIDEOUT_WIDTH,
        GOBLIN_HIDEOUT_HEIGHT,
        GOBLIN_HIDEOUT_WIDTH,
        GOBLIN_HIDEOUT_HEIGHT,
    )
    .with_grid(GridType::Square, GOBLIN_HIDEOUT_GRID_PX, 0, 0);

    if let Some(mid) = module_id {
        new_map = new_map.with_module(mid);
    }

    let mut service = MapService::new(conn);
    let map = service.create_map(new_map)?;

    // Add monster tokens (PCs can be added via UI "Add PCs" button)
    seed_monster_tokens(conn, map.id)?;

    // Add special tokens: trap, NPC, point of interest
    seed_special_tokens(conn, map.id, characters)?;

    // Add light sources
    seed_light_sources(conn, map.id)?;

    info!("Created battle map with tokens and lights");
    Ok(())
}

fn seed_monster_tokens(conn: &mut DbConnection, map_id: i32) -> Result<()> {
    let tokens: &[(&str, f32, f32, TokenSize)] = &[
        ("Adult Black Dragon", 513.0, 351.0, TokenSize::Huge),
        ("Bugbear", 621.0, 837.0, TokenSize::Medium),
        ("Goblin", 729.0, 945.0, TokenSize::Small),
        ("Goblin", 621.0, 945.0, TokenSize::Small),
        ("Goblin", 1701.0, 459.0, TokenSize::Small),
        ("Goblin", 1593.0, 459.0, TokenSize::Small),
        ("Goblin", 1647.0, 567.0, TokenSize::Small),
        ("Goblin", 1593.0, 513.0, TokenSize::Small),
        ("Goblin", 1647.0, 675.0, TokenSize::Small),
        ("Goblin", 2025.0, 567.0, TokenSize::Small),
        ("Goblin", 1809.0, 1053.0, TokenSize::Small),
        ("Wolf", 1863.0, 1107.0, TokenSize::Medium),
        ("Wolf", 1755.0, 1107.0, TokenSize::Medium),
    ];

    let mut service = TokenService::new(conn);
    for (name, x, y, size) in tokens {
        let mut token = NewToken::new(map_id, name.to_string(), *x, *y)
            .with_type(TokenType::Monster)
            .with_size(*size)
            .with_visibility(true);
        token.image_path = Some(format!("img/bestiary/tokens/MM/{}.webp", name));
        service.create_token(token)?;
    }
    Ok(())
}

fn seed_special_tokens(conn: &mut DbConnection, map_id: i32, characters: &[Character]) -> Result<()> {
    let mut service = TokenService::new(conn);

    // Trap - pit trap in the cave entrance area
    let trap = NewToken::trap(map_id, "Hidden Pit Trap".into(), 1100.0, 700.0);
    service.create_token(trap)?;

    // NPC - Sildar Hallwinter as captive
    if let Some(sildar) = characters.iter().find(|c| c.character_name == "Sildar Hallwinter") {
        let npc = NewToken::character(
            map_id,
            sildar.character_name.clone(),
            sildar.id,
            false, // is_pc = false for NPC
            TokenSize::Medium,
            1900.0,
            400.0,
        );
        service.create_token(npc)?;
    }

    // Point of interest marker - treasure cache
    let poi = NewToken::marker(map_id, "Hidden Treasure Cache".into(), 2200.0, 900.0);
    service.create_token(poi)?;

    info!("Created special tokens (trap, NPC, POI)");
    Ok(())
}

fn seed_light_sources(conn: &mut DbConnection, map_id: i32) -> Result<()> {
    let mut service = LightSourceService::new(conn);

    // Torch in the main chamber (bright 20ft, dim 40ft)
    let torch1 = NewLightSource::torch(map_id, 800.0, 800.0)
        .with_name("Wall Torch".into());
    service.create_light_source(torch1)?;

    // Lantern near treasure cache (bright 30ft, dim 60ft)
    let lantern = NewLightSource::lantern(map_id, 2100.0, 850.0)
        .with_name("Hanging Lantern".into());
    service.create_light_source(lantern)?;

    // Inactive torch (to demonstrate toggle)
    let torch2 = NewLightSource::torch(map_id, 1500.0, 600.0)
        .with_name("Unlit Torch".into())
        .inactive();
    service.create_light_source(torch2)?;

    info!("Created light sources (2 active, 1 inactive)");
    Ok(())
}

fn seed_region_map(conn: &mut DbConnection, campaign_id: i32, data_dir: &str) -> Result<()> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    // Load PNG from disk
    let Some(png_data) = load_seed_asset("GoblinRegion.png") else {
        warn!("Skipping region map - asset not found. Set MIMIR_DEV_ASSETS or run from repo root.");
        return Ok(());
    };

    let maps_dir = PathBuf::from(data_dir)
        .join("campaigns")
        .join(campaign_id.to_string())
        .join("maps");
    std::fs::create_dir_all(&maps_dir)?;

    // Create UVTT wrapper for PNG
    let uvtt = serde_json::json!({
        "format": 0.3,
        "resolution": {
            "map_origin": {"x": 0, "y": 0},
            "map_size": {"x": GOBLIN_REGION_WIDTH as f64 / 70.0, "y": GOBLIN_REGION_HEIGHT as f64 / 70.0},
            "pixels_per_grid": 70
        },
        "image": STANDARD.encode(&png_data),
        "line_of_sight": [],
        "portals": [],
        "lights": []
    });

    let filename = "goblin-region.dd2vtt";
    std::fs::write(maps_dir.join(filename), serde_json::to_vec_pretty(&uvtt)?)?;

    let new_map = NewMap::new(
        campaign_id,
        "Goblin Region".into(),
        filename.into(),
        GOBLIN_REGION_WIDTH,
        GOBLIN_REGION_HEIGHT,
        GOBLIN_REGION_WIDTH,
        GOBLIN_REGION_HEIGHT,
    );

    let mut service = MapService::new(conn);
    service.create_map(new_map)?;

    info!("Created region map");
    Ok(())
}
