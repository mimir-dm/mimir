//! Development database seeder.
//!
//! Seeds the database with "The Lost Mine of Phandelver" test data.

use diesel::SqliteConnection;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{
    Character, Module, NewCampaignAsset, NewCharacter, NewLightSource, NewMap, NewMapPoi,
    NewMapTrap, NewModuleMonster, NewModuleNpc, NewTokenPlacement,
};
use crate::services::{
    CampaignService, CreateCampaignInput, CreateModuleInput, ModuleService, ModuleType,
    ServiceResult,
};

/// IDs of seeded monsters for token placement.
struct SeededMonsters {
    entrance_guards: String,
    bridge_archers: String,
    main_chamber: String,
    klarg: String,
    wolves: String,
    yeemik_guards: String,
    amethyst_dragon: String,
}

/// IDs of seeded NPCs for token placement.
struct SeededNpcs {
    sildar: String,
    yeemik: String,
}

/// Name of the test campaign for detection.
pub const TEST_CAMPAIGN_NAME: &str = "The Lost Mine of Phandelver";

/// Check if dev seed data already exists.
pub fn is_already_seeded(conn: &mut SqliteConnection) -> ServiceResult<bool> {
    let campaigns = dal::list_campaigns(conn, false)?;
    Ok(campaigns.iter().any(|c| c.name == TEST_CAMPAIGN_NAME))
}

/// Seed development data into the database.
///
/// Creates a test campaign with modules, characters, monsters, and NPCs.
/// Stateful - only seeds if no dev data exists yet.
///
/// **Prerequisites**: Import MM book via Library for full monster data.
///
/// Returns true if seeding was performed, false if data already existed.
pub fn seed_dev_data(conn: &mut SqliteConnection, app_data_dir: &Path) -> ServiceResult<bool> {
    if is_already_seeded(conn)? {
        info!("Dev seed data already exists, skipping");
        return Ok(false);
    }

    info!("Seeding development data...");

    // 1. Create campaign
    let campaign = seed_campaign(conn)?;
    info!("Created campaign: {}", campaign.name);

    // 2. Create characters (PCs and NPCs)
    let characters = seed_characters(conn, &campaign.id)?;
    info!("Created {} characters", characters.len());

    // 3. Create module
    let module = seed_module(conn, &campaign.id)?;
    info!("Created module: {}", module.name);

    // 4. Add monsters to module
    let monsters = seed_monsters(conn, &module.id)?;
    info!("Added 7 monster groups to module");

    // 5. Add NPCs to module
    let npcs = seed_npcs(conn, &module.id)?;
    info!("Added 2 NPCs to module");

    // 6. Seed maps if assets available
    let map_id = if let Some(assets_dir) = get_seed_assets_dir() {
        let id = seed_maps(conn, &campaign.id, &module.id, &assets_dir, app_data_dir)?;
        info!("Created maps from seed assets");
        id
    } else {
        warn!("Seed assets not found, skipping map creation");
        None
    };

    // 7. Seed tokens, lights, traps, and POIs on the map
    if let Some(ref map_id) = map_id {
        let token_count = seed_tokens(conn, map_id, &monsters, &npcs)?;
        info!("Placed {} tokens on map", token_count);

        let light_count = seed_lights(conn, map_id)?;
        info!("Added {} light sources to map", light_count);

        let trap_count = seed_traps(conn, map_id)?;
        info!("Added {} traps to map", trap_count);

        let poi_count = seed_pois(conn, map_id)?;
        info!("Added {} POIs to map", poi_count);
    }

    info!("Dev seed data created successfully");
    Ok(true)
}

/// Clear existing dev seed data.
pub fn clear_dev_seed_data(conn: &mut SqliteConnection) -> ServiceResult<()> {
    let campaigns = dal::list_campaigns(conn, false)?;
    let campaign = campaigns.iter().find(|c| c.name == TEST_CAMPAIGN_NAME);

    if let Some(campaign) = campaign {
        info!("Clearing dev seed data for campaign {}", campaign.id);

        // Delete campaign (cascade deletes modules, characters, documents, etc.)
        dal::delete_campaign(conn, &campaign.id)?;

        info!("Cleared dev seed data");
    }

    Ok(())
}

// =============================================================================
// Campaign Seeding
// =============================================================================

fn seed_campaign(conn: &mut SqliteConnection) -> ServiceResult<crate::models::campaign::Campaign> {
    let input = CreateCampaignInput::new(TEST_CAMPAIGN_NAME)
        .with_description("A classic D&D adventure for 4-5 characters of levels 1-5");

    CampaignService::new(conn).create(input)
}

// =============================================================================
// Character Seeding
// =============================================================================

fn seed_characters(conn: &mut SqliteConnection, campaign_id: &str) -> ServiceResult<Vec<Character>> {
    let mut characters = Vec::new();

    // Player Characters
    let pcs = [
        ("Thorin Ironforge", "Alice", "Dwarf", 14, 12, 16, 10, 12, 8),
        ("Elara Moonwhisper", "Bob", "Elf", 8, 14, 12, 17, 13, 10),
        ("Finn Lightfoot", "Charlie", "Halfling", 10, 18, 12, 13, 10, 14),
        ("Sister Helena", "Diana", "Human", 14, 10, 14, 10, 16, 13),
    ];

    for (name, player, race, str, dex, con, int, wis, cha) in pcs {
        let id = Uuid::new_v4().to_string();
        let character = NewCharacter::new_pc(&id, campaign_id, name, player)
            .with_race(race, "PHB")
            .with_ability_scores(str, dex, con, int, wis, cha)
            .with_currency(0, 0, 0, 50, 0);

        dal::insert_character(conn, &character)?;
        characters.push(dal::get_character(conn, &id)?);
    }

    // NPCs
    let npcs = [
        ("Sildar Hallwinter", "Human", "Ally"),
        ("Gundren Rockseeker", "Dwarf", "Quest Giver"),
        ("Toblen Stonehill", "Human", "Innkeeper"),
        ("Iarno Albrek", "Human", "Antagonist"),
        ("Klarg", "Bugbear", "Boss"),
    ];

    for (name, race, role) in npcs {
        let id = Uuid::new_v4().to_string();
        let character = NewCharacter::new_npc(&id, campaign_id, name)
            .with_race(race, "PHB")
            .with_ability_scores(14, 12, 14, 10, 10, 10)
            .with_npc_info(Some(role), None, None);

        dal::insert_character(conn, &character)?;
        characters.push(dal::get_character(conn, &id)?);
    }

    Ok(characters)
}

// =============================================================================
// Module Seeding
// =============================================================================

fn seed_module(conn: &mut SqliteConnection, campaign_id: &str) -> ServiceResult<Module> {
    let input = CreateModuleInput::new(campaign_id, "Cragmaw Hideout")
        .with_description("A goblin hideout in the Triboar Trail area")
        .with_type(ModuleType::Dungeon);

    ModuleService::new(conn).create(input)
}

// =============================================================================
// Monster Seeding
// =============================================================================

fn seed_monsters(conn: &mut SqliteConnection, module_id: &str) -> ServiceResult<SeededMonsters> {
    // Generate IDs upfront so we can return them for token placement
    let entrance_guards_id = Uuid::new_v4().to_string();
    let bridge_archers_id = Uuid::new_v4().to_string();
    let main_chamber_id = Uuid::new_v4().to_string();
    let klarg_id = Uuid::new_v4().to_string();
    let wolves_id = Uuid::new_v4().to_string();
    let yeemik_guards_id = Uuid::new_v4().to_string();
    let amethyst_dragon_id = Uuid::new_v4().to_string();

    // Monsters from Monster Manual (MM) and Fizban's Treasury of Dragons (FTD)
    // (id, name, source, quantity, display_name, notes)
    let monsters: &[(&str, &str, &str, i32, Option<&str>, Option<&str>)] = &[
        (&entrance_guards_id, "Goblin", "MM", 6, None, Some("Cave entrance guards")),
        (&bridge_archers_id, "Goblin", "MM", 3, Some("Goblin Archers"), Some("Guard post on bridge")),
        (&main_chamber_id, "Goblin", "MM", 5, None, Some("Main chamber")),
        (&klarg_id, "Bugbear", "MM", 1, Some("Klarg"), Some("Bugbear chief - boss encounter")),
        (&wolves_id, "Wolf", "MM", 2, Some("Ripper & Fang"), Some("Klarg's pet wolves")),
        (&yeemik_guards_id, "Goblin", "MM", 2, Some("Yeemik's Guards"), Some("With the second-in-command")),
        (&amethyst_dragon_id, "Adult Amethyst Dragon", "FTD", 1, None, Some("Surprise boss encounter")),
    ];

    for (id, name, source, qty, display_name, notes) in monsters {
        let mut monster = NewModuleMonster::new(id, module_id, name, source)
            .with_quantity(*qty);

        if let Some(dn) = display_name {
            monster = monster.with_display_name(dn);
        }
        if let Some(n) = notes {
            monster = monster.with_notes(n);
        }

        dal::insert_module_monster(conn, &monster)?;
    }

    Ok(SeededMonsters {
        entrance_guards: entrance_guards_id,
        bridge_archers: bridge_archers_id,
        main_chamber: main_chamber_id,
        klarg: klarg_id,
        wolves: wolves_id,
        yeemik_guards: yeemik_guards_id,
        amethyst_dragon: amethyst_dragon_id,
    })
}

// =============================================================================
// NPC Seeding
// =============================================================================

fn seed_npcs(conn: &mut SqliteConnection, module_id: &str) -> ServiceResult<SeededNpcs> {
    // Generate IDs upfront so we can return them for token placement
    let sildar_id = Uuid::new_v4().to_string();
    let yeemik_id = Uuid::new_v4().to_string();

    // Custom NPCs for the module
    // (id, name, role, description, appearance, secrets)
    let npcs: &[(&str, &str, &str, &str, Option<&str>, Option<&str>)] = &[
        (
            &sildar_id,
            "Sildar Hallwinter",
            "Captive",
            "A human warrior, beaten and bound. Agent of the Lords' Alliance.",
            Some("Middle-aged human male with a military bearing, currently bruised and bloodied"),
            Some("Knows location of Cragmaw Castle and Wave Echo Cave"),
        ),
        (
            &yeemik_id,
            "Yeemik",
            "Second-in-Command",
            "A cunning goblin who wants to overthrow Klarg.",
            Some("Scrawny goblin with calculating eyes and a sneer"),
            Some("Will betray Klarg if offered a deal"),
        ),
    ];

    for (id, name, role, description, appearance, secrets) in npcs {
        let mut npc = NewModuleNpc::new(id, module_id, name)
            .with_role(role)
            .with_description(description);

        if let Some(app) = appearance {
            npc = npc.with_appearance(app);
        }
        if let Some(sec) = secrets {
            npc = npc.with_secrets(sec);
        }

        dal::insert_module_npc(conn, &npc)?;
    }

    Ok(SeededNpcs {
        sildar: sildar_id,
        yeemik: yeemik_id,
    })
}

// =============================================================================
// Map Seeding
// =============================================================================

/// Get the seed assets directory path.
fn get_seed_assets_dir() -> Option<PathBuf> {
    // Try relative to current working directory (for running from repo root)
    let cwd_assets = PathBuf::from("crates/mimir-core/src/seed/assets");
    if cwd_assets.exists() {
        return Some(cwd_assets);
    }

    // Try relative to executable
    if let Ok(exe) = std::env::current_exe() {
        let mut current = exe.parent();
        while let Some(dir) = current {
            let cargo_toml = dir.join("Cargo.toml");
            if cargo_toml.exists() {
                let assets_dir = dir.join("crates/mimir-core/src/seed/assets");
                if assets_dir.exists() {
                    return Some(assets_dir);
                }
            }
            current = dir.parent();
        }
    }

    // Check MIMIR_SEED_ASSETS env var
    if let Ok(path) = std::env::var("MIMIR_SEED_ASSETS") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Some(p);
        }
    }

    None
}

fn seed_maps(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    module_id: &str,
    assets_dir: &Path,
    app_data_dir: &Path,
) -> ServiceResult<Option<String>> {
    let mut battle_map_id = None;

    // Seed battle map (UVTT)
    let uvtt_path = assets_dir.join("goblin-hideout.dd2vtt");
    if uvtt_path.exists() {
        battle_map_id = Some(seed_uvtt_map(conn, campaign_id, Some(module_id), &uvtt_path, app_data_dir, "Goblin Hideout")?);
    }

    // Seed region map (PNG wrapped in UVTT)
    let region_path = assets_dir.join("GoblinRegion.png");
    if region_path.exists() {
        seed_png_map(conn, campaign_id, None, &region_path, app_data_dir, "Triboar Trail Region")?;
    }

    Ok(battle_map_id)
}

fn seed_uvtt_map(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    module_id: Option<&str>,
    uvtt_path: &Path,
    app_data_dir: &Path,
    name: &str,
) -> ServiceResult<String> {
    let data = std::fs::read(uvtt_path)?;
    let filename = uvtt_path.file_name().unwrap().to_str().unwrap();

    // Create asset
    let asset_id = Uuid::new_v4().to_string();
    let relative_path = format!("assets/{}.dd2vtt", asset_id);
    let full_path = app_data_dir.join(&relative_path);

    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&full_path, &data)?;

    let asset = if let Some(mid) = module_id {
        NewCampaignAsset::for_module(&asset_id, mid, filename, "application/octet-stream", &relative_path)
    } else {
        NewCampaignAsset::for_campaign(&asset_id, campaign_id, filename, "application/octet-stream", &relative_path)
    };
    let asset = asset.with_file_size(data.len() as i32);
    dal::insert_campaign_asset(conn, &asset)?;

    // Create map referencing the asset
    let map_id = Uuid::new_v4().to_string();
    let map = if let Some(mid) = module_id {
        NewMap::for_module(&map_id, campaign_id, mid, name, &asset_id)
    } else {
        NewMap::for_campaign(&map_id, campaign_id, name, &asset_id)
    };
    dal::insert_map(conn, &map)?;

    info!("Created map '{}' from {}", name, filename);
    Ok(map_id)
}

fn seed_png_map(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    module_id: Option<&str>,
    png_path: &Path,
    app_data_dir: &Path,
    name: &str,
) -> ServiceResult<()> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let png_data = std::fs::read(png_path)?;
    let filename = png_path.file_name().unwrap().to_str().unwrap();

    // Create UVTT wrapper for PNG
    let uvtt = serde_json::json!({
        "format": 0.3,
        "resolution": {
            "map_origin": {"x": 0, "y": 0},
            "map_size": {"x": 25.0, "y": 14.0},
            "pixels_per_grid": 70
        },
        "image": STANDARD.encode(&png_data),
        "line_of_sight": [],
        "portals": [],
        "lights": []
    });

    let uvtt_data = serde_json::to_vec_pretty(&uvtt)
        .map_err(|e| crate::services::ServiceError::validation(format!("JSON error: {}", e)))?;

    // Create asset
    let asset_id = Uuid::new_v4().to_string();
    let relative_path = format!("assets/{}.dd2vtt", asset_id);
    let full_path = app_data_dir.join(&relative_path);

    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&full_path, &uvtt_data)?;

    let uvtt_filename = filename.replace(".png", ".dd2vtt");
    let asset = if let Some(mid) = module_id {
        NewCampaignAsset::for_module(&asset_id, mid, &uvtt_filename, "application/octet-stream", &relative_path)
    } else {
        NewCampaignAsset::for_campaign(&asset_id, campaign_id, &uvtt_filename, "application/octet-stream", &relative_path)
    };
    let asset = asset.with_file_size(uvtt_data.len() as i32);
    dal::insert_campaign_asset(conn, &asset)?;

    // Create map referencing the asset
    let map_id = Uuid::new_v4().to_string();
    let map = if let Some(mid) = module_id {
        NewMap::for_module(&map_id, campaign_id, mid, name, &asset_id)
    } else {
        NewMap::for_campaign(&map_id, campaign_id, name, &asset_id)
    };
    dal::insert_map(conn, &map)?;

    info!("Created region map '{}' from {}", name, filename);
    Ok(())
}

// =============================================================================
// Token Seeding
// =============================================================================

fn seed_tokens(
    conn: &mut SqliteConnection,
    map_id: &str,
    monsters: &SeededMonsters,
    npcs: &SeededNpcs,
) -> ServiceResult<usize> {
    // Map is 2592x1458 pixels with 54px grid = ~48x27 grid cells
    // Positions based on actual placed tokens in dev database
    let mut count = 0;

    // Klarg the Bugbear boss
    let klarg_id = Uuid::new_v4().to_string();
    let klarg_placement = NewTokenPlacement::for_monster(&klarg_id, map_id, &monsters.klarg, 12, 15)
        .with_label("Klarg")
        .with_faction_color("#7C3AED"); // Purple for boss
    dal::insert_token_placement(conn, &klarg_placement)?;
    count += 1;

    // Goblins near Klarg
    let klarg_goblin_positions = [(14, 18), (12, 18)];
    for (i, (x, y)) in klarg_goblin_positions.iter().enumerate() {
        let id = Uuid::new_v4().to_string();
        let label = format!("Goblin {}", i + 1);
        let placement = NewTokenPlacement::for_monster(&id, map_id, &monsters.entrance_guards, *x, *y)
            .with_label(&label)
            .with_faction_color("#DC2626"); // Red for enemies
        dal::insert_token_placement(conn, &placement)?;
        count += 1;
    }

    // Bridge goblin archers
    let archer_positions = [(31, 9), (30, 9), (31, 10), (30, 10), (31, 13)];
    for (i, (x, y)) in archer_positions.iter().enumerate() {
        let id = Uuid::new_v4().to_string();
        let label = format!("Archer {}", i + 1);
        let placement = NewTokenPlacement::for_monster(&id, map_id, &monsters.bridge_archers, *x, *y)
            .with_label(&label)
            .with_faction_color("#DC2626");
        dal::insert_token_placement(conn, &placement)?;
        count += 1;
    }

    // Goblin in passage
    let passage_goblin_id = Uuid::new_v4().to_string();
    let passage_goblin = NewTokenPlacement::for_monster(&passage_goblin_id, map_id, &monsters.main_chamber, 38, 10)
        .with_label("Goblin Sentry")
        .with_faction_color("#DC2626");
    dal::insert_token_placement(conn, &passage_goblin)?;
    count += 1;

    // Wolves - Ripper and Fang
    let wolf_positions = [(34, 20), (32, 20)];
    for (i, (x, y)) in wolf_positions.iter().enumerate() {
        let id = Uuid::new_v4().to_string();
        let label = if i == 0 { "Ripper" } else { "Fang" };
        let placement = NewTokenPlacement::for_monster(&id, map_id, &monsters.wolves, *x, *y)
            .with_label(label)
            .with_faction_color("#DC2626");
        dal::insert_token_placement(conn, &placement)?;
        count += 1;
    }

    // Goblin wolf handler
    let wolf_goblin_id = Uuid::new_v4().to_string();
    let wolf_goblin = NewTokenPlacement::for_monster(&wolf_goblin_id, map_id, &monsters.yeemik_guards, 33, 19)
        .with_label("Wolf Handler")
        .with_faction_color("#DC2626");
    dal::insert_token_placement(conn, &wolf_goblin)?;
    count += 1;

    // Sildar Hallwinter - captive NPC
    let sildar_token_id = Uuid::new_v4().to_string();
    let sildar_placement = NewTokenPlacement::for_npc(&sildar_token_id, map_id, &npcs.sildar, 35, 7)
        .with_label("Sildar (captive)")
        .with_faction_color("#22C55E") // Green for ally
        .hidden(); // Hidden until discovered
    dal::insert_token_placement(conn, &sildar_placement)?;
    count += 1;

    // Yeemik - negotiable goblin leader
    let yeemik_token_id = Uuid::new_v4().to_string();
    let yeemik_placement = NewTokenPlacement::for_npc(&yeemik_token_id, map_id, &npcs.yeemik, 32, 7)
        .with_label("Yeemik")
        .with_faction_color("#F59E0B"); // Amber for neutral/negotiable
    dal::insert_token_placement(conn, &yeemik_placement)?;
    count += 1;

    // Adult Amethyst Dragon - surprise boss
    let dragon_id = Uuid::new_v4().to_string();
    let dragon_placement = NewTokenPlacement::for_monster(&dragon_id, map_id, &monsters.amethyst_dragon, 8, 5)
        .with_label("Adult Amethyst Dragon")
        .with_faction_color("#dc2626"); // Red for enemy
    dal::insert_token_placement(conn, &dragon_placement)?;
    count += 1;

    Ok(count)
}

// =============================================================================
// Light Source Seeding
// =============================================================================

fn seed_lights(conn: &mut SqliteConnection, map_id: &str) -> ServiceResult<usize> {
    // Positions based on actual placed lights in dev database
    let mut count = 0;

    // Wall torch in main chamber
    let torch1_id = Uuid::new_v4().to_string();
    let torch1 = NewLightSource::torch(&torch1_id, map_id, 16, 14)
        .with_name("Wall Torch");
    dal::insert_light_source(conn, &torch1)?;
    count += 1;

    // Hanging lantern near treasure cache
    let lantern_id = Uuid::new_v4().to_string();
    let lantern = NewLightSource::lantern(&lantern_id, map_id, 43, 16)
        .with_name("Hanging Lantern");
    dal::insert_light_source(conn, &lantern)?;
    count += 1;

    // Unlit torch - players can light it
    let unlit_id = Uuid::new_v4().to_string();
    let unlit_torch = NewLightSource::torch(&unlit_id, map_id, 28, 5)
        .with_name("Unlit Torch")
        .inactive();
    dal::insert_light_source(conn, &unlit_torch)?;
    count += 1;

    Ok(count)
}

// =============================================================================
// Trap Seeding
// =============================================================================

fn seed_traps(conn: &mut SqliteConnection, map_id: &str) -> ServiceResult<usize> {
    let mut count = 0;

    // Pit trap near entrance (catalog name is "Pits" in DMG)
    let pit_trap_id = Uuid::new_v4().to_string();
    let pit_trap = NewMapTrap::new(&pit_trap_id, map_id, "Pits", 21, 19)
        .visible();
    dal::insert_map_trap(conn, &pit_trap)?;
    count += 1;

    // Poison dart trap in corridor
    let dart_trap_id = Uuid::new_v4().to_string();
    let dart_trap = NewMapTrap::new(&dart_trap_id, map_id, "Poison Darts", 25, 18)
        .visible();
    dal::insert_map_trap(conn, &dart_trap)?;
    count += 1;

    Ok(count)
}

// =============================================================================
// POI Seeding
// =============================================================================

fn seed_pois(conn: &mut SqliteConnection, map_id: &str) -> ServiceResult<usize> {
    let mut count = 0;

    // Point of interest marker
    let poi_id = Uuid::new_v4().to_string();
    let poi = NewMapPoi::new(&poi_id, map_id, "Point of Interest", 38, 4)
        .with_icon("pin")
        .with_color("#9333ea")
        .visible();
    dal::insert_map_poi(conn, &poi)?;
    count += 1;

    Ok(count)
}
