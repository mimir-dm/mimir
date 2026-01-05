//! Map and token seeding for dev data.

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::campaign::modules::Module;
use crate::models::campaign::{GridType, NewMap, NewToken, TokenSize, TokenType};
use crate::services::{MapService, TokenService};
use std::path::PathBuf;
use tracing::info;

// Embedded map files
const GOBLIN_HIDEOUT_UVTT: &[u8] = include_bytes!("../assets/goblin-hideout.dd2vtt");
const GOBLIN_HIDEOUT_WIDTH: i32 = 2592;
const GOBLIN_HIDEOUT_HEIGHT: i32 = 1458;
const GOBLIN_HIDEOUT_GRID_PX: i32 = 54;

const GOBLIN_REGION_PNG: &[u8] = include_bytes!("../assets/GoblinRegion.png");
const GOBLIN_REGION_WIDTH: i32 = 1792;
const GOBLIN_REGION_HEIGHT: i32 = 1024;

/// Seed maps and tokens.
pub fn seed(conn: &mut DbConnection, campaign_id: i32, modules: &[Module], data_dir: &str) -> Result<()> {
    seed_battle_map(conn, campaign_id, modules, data_dir)?;
    seed_region_map(conn, campaign_id, data_dir)?;
    Ok(())
}

fn seed_battle_map(conn: &mut DbConnection, campaign_id: i32, modules: &[Module], data_dir: &str) -> Result<()> {
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
    std::fs::write(maps_dir.join(filename), GOBLIN_HIDEOUT_UVTT)?;

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

    info!("Created battle map with monster tokens");
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

fn seed_region_map(conn: &mut DbConnection, campaign_id: i32, data_dir: &str) -> Result<()> {
    use base64::{engine::general_purpose::STANDARD, Engine};

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
        "image": STANDARD.encode(GOBLIN_REGION_PNG),
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
