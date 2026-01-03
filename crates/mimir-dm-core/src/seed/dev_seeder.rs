//! Development database seeder
//!
//! Seeds the database with test data for development and testing purposes.
//! This includes a sample campaign, players, and characters with various
//! levels and configurations.

use crate::connection::DbConnection;
use crate::dal::campaign::campaigns::CampaignRepository;
use crate::error::Result;
use crate::models::character::CharacterData;
use crate::models::character::{
    AbilityScores, Appearance, ClassLevel, Currency, EquippedItems, FeatureReference, InventoryItem,
    LegendaryAction, Personality, Proficiencies, RoleplayNotes, SpellData, SpellReference, SpellSlots,
};
use crate::services::{
    CampaignService, CampaignSummaryService, CharacterService, DocumentService,
    MapService, ModuleMonsterService, ModuleService, PlayerService, TokenService,
};
use chrono::Utc;
use std::collections::HashMap;
use tracing::info;

/// Monster entry for seeding: (name, source, quantity, encounter_tag)
type MonsterSeedEntry<'a> = (&'a str, &'a str, i32, Option<&'a str>);

/// Module monsters for seeding: (module_name, monsters)
type ModuleMonstersSeed<'a> = (&'a str, Vec<MonsterSeedEntry<'a>>);

/// Name of the test campaign used to check idempotency
const TEST_CAMPAIGN_NAME: &str = "The Lost Mine of Phandelver";

/// Check if dev seed data already exists
pub fn is_already_seeded(conn: &mut DbConnection) -> Result<bool> {
    let mut repo = CampaignRepository::new(conn);
    let campaigns = repo.list()?;
    Ok(campaigns.iter().any(|c| c.name == TEST_CAMPAIGN_NAME))
}

/// Clear existing dev seed data to allow re-seeding
fn clear_dev_seed_data(conn: &mut DbConnection) -> Result<()> {
    use crate::dal::campaign::documents::DocumentRepository;
    use crate::dal::campaign::module_monsters::ModuleMonsterRepository;
    use crate::dal::campaign::modules::ModuleRepository;
    use crate::dal::character::CharacterRepository;
    use crate::dal::player::PlayerRepository;

    // First, find the dev campaign
    let campaign_info = {
        let mut repo = CampaignRepository::new(conn);
        let campaigns = repo.list()?;
        campaigns
            .into_iter()
            .find(|c| c.name == TEST_CAMPAIGN_NAME)
            .map(|c| (c.id, c.directory_path.clone()))
    };

    let Some((campaign_id, directory_path)) = campaign_info else {
        return Ok(()); // No dev campaign exists, nothing to clear
    };

    info!(
        "Clearing existing dev seed data for campaign id: {}",
        campaign_id
    );

    // Get module IDs
    let module_ids: Vec<i32> = {
        let mut module_repo = ModuleRepository::new(conn);
        module_repo
            .list_by_campaign(campaign_id)?
            .into_iter()
            .map(|m| m.id)
            .collect()
    };

    // 1. Delete module monsters
    for module_id in &module_ids {
        let mut monster_repo = ModuleMonsterRepository::new(conn);
        monster_repo.delete_by_module(*module_id)?;
    }

    // 2. Delete documents
    let doc_ids: Vec<i32> = DocumentRepository::find_by_campaign(conn, campaign_id)?
        .into_iter()
        .map(|d| d.id)
        .collect();
    for doc_id in doc_ids {
        DocumentRepository::delete(conn, doc_id)?;
    }

    // 4. Delete characters for this campaign
    let character_ids: Vec<i32> = {
        let mut char_repo = CharacterRepository::new(conn);
        char_repo
            .list_for_campaign(campaign_id)?
            .into_iter()
            .map(|c| c.id)
            .collect()
    };
    for character_id in character_ids {
        let mut char_repo = CharacterRepository::new(conn);
        char_repo.delete(character_id)?;
    }

    // 5. Delete modules
    for module_id in module_ids {
        let mut module_repo = ModuleRepository::new(conn);
        module_repo.delete(module_id)?;
    }

    // 6. Delete players (they're created by dev seeder)
    let dev_player_names = ["Alice", "Bob", "Charlie", "Diana"];
    let player_ids: Vec<i32> = {
        let mut player_repo = PlayerRepository::new(conn);
        player_repo
            .list()?
            .into_iter()
            .filter(|p| dev_player_names.contains(&p.name.as_str()))
            .map(|p| p.id)
            .collect()
    };
    for player_id in player_ids {
        let mut player_repo = PlayerRepository::new(conn);
        player_repo.delete(player_id)?;
    }

    // 7. Delete campaign
    {
        let mut repo = CampaignRepository::new(conn);
        repo.delete(campaign_id)?;
    }

    // 8. Delete campaign directory
    if std::path::Path::new(&directory_path).exists() {
        if let Err(e) = std::fs::remove_dir_all(&directory_path) {
            info!("Note: Could not remove campaign directory: {}", e);
        }
    }

    info!("Cleared existing dev seed data");
    Ok(())
}

/// Seed development data into the database
///
/// Creates a test campaign with modules, sessions, players, and characters.
/// This function is stateful - it only seeds if no dev data exists yet.
/// Data persists between restarts to allow testing real user workflows.
///
/// # Arguments
/// * `conn` - Database connection
/// * `campaigns_directory` - Base directory for campaign files
/// * `data_directory` - App data directory (for maps storage)
///
/// # Returns
/// * `Ok(bool)` - true if seeding was performed, false if data already existed
pub fn seed_dev_data(conn: &mut DbConnection, campaigns_directory: &str, data_directory: &str) -> Result<bool> {
    // Check if dev data already exists - if so, preserve it
    if is_already_seeded(conn)? {
        info!("Dev seed data already exists, skipping (stateful mode)");
        return Ok(false);
    }

    info!("Seeding development data...");

    // Create campaign
    let campaign = seed_campaign(conn, campaigns_directory)?;
    info!(
        "Created test campaign: {} (id={})",
        campaign.name, campaign.id
    );

    // Create modules
    let modules = seed_modules(conn, campaign.id)?;
    info!("Created {} modules", modules.len());

    // Transition Goblin Ambush module to "active" stage
    if let Some(ambush_module) = modules.iter().find(|m| m.name == "Goblin Ambush") {
        transition_module_to_stage(conn, ambush_module.id, &campaign.directory_path, "active")?;
        info!(
            "Transitioned module '{}' to active stage",
            ambush_module.name
        );
    }

    // Transition Cragmaw Hideout module to "ready" stage (it has the battle map)
    if let Some(cragmaw_module) = modules.iter().find(|m| m.name == "Cragmaw Hideout") {
        transition_module_to_stage(conn, cragmaw_module.id, &campaign.directory_path, "ready")?;
        info!(
            "Transitioned module '{}' to ready stage",
            cragmaw_module.name
        );
    }

    // Add monsters to modules
    seed_module_monsters(conn, &modules)?;
    info!("Added monsters to modules");

    // Fill in document content for modules
    seed_module_document_content(conn, &modules, &campaign.directory_path)?;
    info!("Populated module document content");

    // Create session notes documents
    seed_session_notes(conn, campaign.id, &campaign.directory_path)?;
    info!("Created session notes documents");

    // Generate campaign summary from session notes
    seed_campaign_summary(conn, campaign.id, &campaign.directory_path)?;
    info!("Generated campaign summary");

    // Create players
    let players = seed_players(conn)?;
    info!("Created {} test players", players.len());

    // Create characters (PCs and NPCs)
    let characters = seed_characters(conn, Some(campaign.id), &campaign.directory_path, &players)?;
    let pc_count = characters.iter().filter(|c| !c.is_npc()).count();
    let npc_count = characters.iter().filter(|c| c.is_npc()).count();
    info!("Created {} test characters ({} PCs, {} NPCs)", characters.len(), pc_count, npc_count);

    // Seed maps (associated with modules)
    let maps_count = seed_maps(conn, campaign.id, &modules, data_directory)?;
    info!("Created {} test maps", maps_count);

    info!("Dev seed data created successfully");
    Ok(true)
}

/// Embedded UVTT map with LOS data (Goblin Hideout - module level)
/// Has 4 wall segments, 11 portals (doors), and 3 lights
const GOBLIN_HIDEOUT_UVTT: &[u8] = include_bytes!("assets/goblin-hideout.dd2vtt");
/// Goblin Hideout dimensions: 48x27 grid at 54px = 2592x1458
const GOBLIN_HIDEOUT_WIDTH: i32 = 2592;
const GOBLIN_HIDEOUT_HEIGHT: i32 = 1458;
const GOBLIN_HIDEOUT_GRID_PX: i32 = 54;

/// Embedded test map image (Goblin Region - campaign level, no LOS)
const GOBLIN_REGION_PNG: &[u8] = include_bytes!("assets/GoblinRegion.png");
const GOBLIN_REGION_WIDTH: i32 = 1792;
const GOBLIN_REGION_HEIGHT: i32 = 1024;

/// Create a minimal UVTT wrapper for a plain image (no LOS/portals/lights)
fn create_image_uvtt(image_bytes: &[u8], width_px: u32, height_px: u32, grid_size_px: u32) -> serde_json::Value {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let image_base64 = STANDARD.encode(image_bytes);
    let grid_cols = width_px as f64 / grid_size_px as f64;
    let grid_rows = height_px as f64 / grid_size_px as f64;

    serde_json::json!({
        "format": 0.3,
        "resolution": {
            "map_origin": {"x": 0, "y": 0},
            "map_size": {"x": grid_cols, "y": grid_rows},
            "pixels_per_grid": grid_size_px
        },
        "image": image_base64,
        "line_of_sight": [],
        "portals": [],
        "lights": []
    })
}

/// Seed test maps for modules
fn seed_maps(
    conn: &mut DbConnection,
    campaign_id: i32,
    modules: &[crate::models::campaign::modules::Module],
    data_directory: &str,
) -> Result<usize> {
    use crate::models::campaign::{NewMap, GridType, NewToken, TokenType, TokenSize, VisionType};
    use std::path::PathBuf;

    // Find the "Cragmaw Hideout" module to associate the map with
    let module = modules.iter().find(|m| m.name == "Cragmaw Hideout");
    let module_id = module.map(|m| m.id);

    // Determine storage path based on module association
    let maps_dir = if let Some(mid) = module_id {
        PathBuf::from(data_directory).join("modules").join(mid.to_string()).join("maps")
    } else {
        PathBuf::from(data_directory).join("campaigns").join(campaign_id.to_string()).join("maps")
    };
    std::fs::create_dir_all(&maps_dir)?;

    // Use the embedded UVTT file directly (has real LOS walls from DungeonDraft)
    let stored_filename = "goblin-hideout.dd2vtt".to_string();
    let uvtt_path = maps_dir.join(&stored_filename);
    std::fs::write(&uvtt_path, GOBLIN_HIDEOUT_UVTT)?;
    info!("Wrote UVTT map to {:?} ({} KB)", uvtt_path, GOBLIN_HIDEOUT_UVTT.len() / 1024);

    // Create database record with grid configuration
    let new_map = NewMap::new(
        campaign_id,
        "Goblin Hideout".to_string(),
        stored_filename,
        GOBLIN_HIDEOUT_WIDTH,
        GOBLIN_HIDEOUT_HEIGHT,
        GOBLIN_HIDEOUT_WIDTH,
        GOBLIN_HIDEOUT_HEIGHT,
    )
    .with_grid(GridType::Square, GOBLIN_HIDEOUT_GRID_PX, 0, 0);

    // Associate with module if found
    let new_map = if let Some(mid) = module_id {
        new_map.with_module(mid)
    } else {
        new_map
    };

    let mut service = MapService::new(conn);
    let map = service.create_map(new_map)?;

    // Add monster tokens for Goblin Hideout
    // Map is 48x27 grid at 54px = 2592x1458 pixels
    // Cell center = (cell * 54 + 27)
    // Format: (name, x, y, size)
    let monster_tokens: Vec<(&str, f32, f32, TokenSize)> = vec![
        // Adult Black Dragon - huge boss in upper left area
        ("Adult Black Dragon", 513.0, 351.0, TokenSize::Huge),
        // Bugbear in lower left area
        ("Bugbear", 621.0, 837.0, TokenSize::Medium),
        // Goblins in lower left cave area
        ("Goblin", 729.0, 945.0, TokenSize::Small),
        ("Goblin", 621.0, 945.0, TokenSize::Small),
        // Goblins in central cave
        ("Goblin", 1701.0, 459.0, TokenSize::Small),
        ("Goblin", 1593.0, 459.0, TokenSize::Small),
        ("Goblin", 1647.0, 567.0, TokenSize::Small),
        ("Goblin", 1593.0, 513.0, TokenSize::Small),
        ("Goblin", 1647.0, 675.0, TokenSize::Small),
        // Guard goblin near entrance
        ("Goblin", 2025.0, 567.0, TokenSize::Small),
        // Goblin in lower area
        ("Goblin", 1809.0, 1053.0, TokenSize::Small),
        // Wolves in kennel area
        ("Wolf", 1863.0, 1107.0, TokenSize::Medium),
        ("Wolf", 1755.0, 1107.0, TokenSize::Medium),
    ];

    let mut token_service = TokenService::new(conn);
    for (name, x, y, size) in &monster_tokens {
        let mut token = NewToken::new(map.id, name.to_string(), *x, *y)
            .with_type(TokenType::Monster)
            .with_size(*size)
            .with_visibility(true);

        // Set token image path for MM monsters (works once MM book is imported)
        // Note: monster_id not set since catalog may not be populated yet
        token.image_path = Some(format!("img/bestiary/tokens/MM/{}.webp", name));

        token_service.create_token(token)?;
    }
    info!("Created {} monster tokens on battle map", monster_tokens.len());

    // Add PC tokens at entrance (bottom right of map)
    let pc_tokens = [
        ("Thorin Ironforge", 2187.0, 1431.0, TokenSize::Medium, VisionType::Darkvision, Some(60.0)),
        ("Elara Moonwhisper", 2187.0, 1377.0, TokenSize::Medium, VisionType::Darkvision, Some(60.0)),
        ("Finn Lightfoot", 2241.0, 1431.0, TokenSize::Small, VisionType::Normal, None),
        ("Sister Helena", 2241.0, 1431.0, TokenSize::Medium, VisionType::Normal, None),
    ];

    for (name, x, y, size, vision_type, vision_range) in pc_tokens {
        let token = NewToken::new(map.id, name.to_string(), x, y)
            .with_type(TokenType::PC)
            .with_size(size)
            .with_visibility(true)
            .with_vision(vision_type, vision_range);
        token_service.create_token(token)?;
    }
    info!("Created {} PC tokens on battle map", pc_tokens.len());

    // Create campaign-level map (Goblin Region - not associated with any module)
    // Campaign maps go to campaigns/{campaign_id}/maps/
    let campaign_maps_dir = PathBuf::from(data_directory)
        .join("campaigns")
        .join(campaign_id.to_string())
        .join("maps");
    std::fs::create_dir_all(&campaign_maps_dir)?;

    // Region map has no LOS/portals (it's a world map) - wrap PNG in minimal UVTT
    let region_uvtt = create_image_uvtt(
        GOBLIN_REGION_PNG,
        GOBLIN_REGION_WIDTH as u32,
        GOBLIN_REGION_HEIGHT as u32,
        70, // Standard 70px grid for region maps
    );

    let region_filename = "dev-seed-goblin-region.dd2vtt".to_string();
    let region_path = campaign_maps_dir.join(&region_filename);
    let region_bytes = serde_json::to_vec_pretty(&region_uvtt)?;
    std::fs::write(&region_path, &region_bytes)?;
    info!("Wrote campaign UVTT map to {:?} ({} KB)", region_path, region_bytes.len() / 1024);

    let region_map = NewMap::new(
        campaign_id,
        "Goblin Region".to_string(),
        region_filename,
        GOBLIN_REGION_WIDTH,
        GOBLIN_REGION_HEIGHT,
        GOBLIN_REGION_WIDTH,
        GOBLIN_REGION_HEIGHT,
    );
    // No .with_module() - this is a campaign-level map
    let mut service = MapService::new(conn);
    service.create_map(region_map)?;

    Ok(2)
}

/// Seed the test campaign
fn seed_campaign(
    conn: &mut DbConnection,
    campaigns_directory: &str,
) -> Result<crate::models::campaign::campaigns::Campaign> {
    let mut service = CampaignService::new(conn);
    let campaign = service.create_campaign(
        TEST_CAMPAIGN_NAME,
        Some("A classic D&D adventure for 4-5 characters of levels 1-5".to_string()),
        campaigns_directory,
    )?;

    // Transition through stages to create all stage documents
    // concept -> session_zero -> integration -> active
    info!("Transitioning campaign through stages to create all documents...");

    let mut service = CampaignService::new(conn);
    service.transition_campaign_stage(campaign.id, "session_zero")?;
    info!("  -> session_zero (created session zero documents)");

    let mut service = CampaignService::new(conn);
    service.transition_campaign_stage(campaign.id, "integration")?;
    info!("  -> integration (created integration documents)");

    let mut service = CampaignService::new(conn);
    let campaign = service.transition_campaign_stage(campaign.id, "active")?;
    info!("  -> active (campaign ready for play)");

    Ok(campaign)
}

/// Seed test modules
fn seed_modules(
    conn: &mut DbConnection,
    campaign_id: i32,
) -> Result<Vec<crate::models::campaign::modules::Module>> {
    let mut modules = Vec::new();

    let module_data = [
        ("Goblin Ambush", 2, Some("dungeon")),
        ("Cragmaw Hideout", 3, Some("dungeon")),
    ];

    for (name, expected_sessions, module_type) in module_data {
        let mut service = ModuleService::new(conn);
        let module = service.create_module_with_documents(
            campaign_id,
            name.to_string(),
            expected_sessions,
            module_type.map(String::from),
        )?;
        modules.push(module);
    }

    Ok(modules)
}

/// Transition a module through stages to a target stage and initialize stage documents
fn transition_module_to_stage(
    conn: &mut DbConnection,
    module_id: i32,
    campaign_directory: &str,
    target_stage: &str,
) -> Result<()> {
    // Module stages: planning -> development -> ready -> active -> completed
    let all_stages = ["development", "ready", "active", "completed"];

    for stage in all_stages {
        let mut service = ModuleService::new(conn);
        service.transition_module_stage(module_id, stage)?;

        // Initialize documents for the new stage
        let mut service = ModuleService::new(conn);
        let docs = service.initialize_module_documents(module_id, campaign_directory)?;
        if !docs.is_empty() {
            info!("Initialized {} documents for stage '{}'", docs.len(), stage);
        }

        // Stop when we reach the target stage
        if stage == target_stage {
            break;
        }
    }

    Ok(())
}

/// Seed module monsters with encounter tags
fn seed_module_monsters(
    conn: &mut DbConnection,
    modules: &[crate::models::campaign::modules::Module],
) -> Result<()> {
    // Monster data for each module: (module_name, monsters)
    // Each monster: (name, source, quantity, encounter_tag)
    let module_monsters: Vec<ModuleMonstersSeed> = vec![
        (
            "Goblin Ambush",
            vec![
                ("Goblin", "MM", 4, Some("Ambush - Road")),
                ("Goblin", "MM", 2, Some("Ambush - Woods")),
                ("Wolf", "MM", 2, Some("Ambush - Road")),
            ],
        ),
        (
            "Cragmaw Hideout",
            vec![
                ("Goblin", "MM", 6, Some("Cave Entrance")),
                ("Goblin", "MM", 3, Some("Guard Post")),
                ("Goblin", "MM", 5, Some("Main Chamber")),
                ("Mage", "MM", 1, Some("Main Chamber")), // Spellcaster!
                ("Wolf", "MM", 2, Some("Kennel")),
                ("Adult Black Dragon", "MM", 1, Some("Boss Chamber")), // Good luck!
                ("Goblin", "MM", 2, Some("Boss Chamber")),
            ],
        ),
    ];

    for module in modules {
        // Find matching monster data for this module
        if let Some((_, monsters)) = module_monsters.iter().find(|(name, _)| *name == module.name) {
            for (monster_name, source, quantity, encounter_tag) in monsters {
                let mut service = ModuleMonsterService::new(conn);
                service.add_monster(
                    module.id,
                    monster_name.to_string(),
                    source.to_string(),
                    *quantity,
                    encounter_tag.map(String::from),
                )?;
            }
        }
    }

    Ok(())
}

/// Seed module document content with Lost Mine of Phandelver content
fn seed_module_document_content(
    conn: &mut DbConnection,
    modules: &[crate::models::campaign::modules::Module],
    campaign_directory: &str,
) -> Result<()> {
    use std::path::PathBuf;

    let module_content: Vec<(&str, &str)> = vec![
        (
            "Goblin Ambush",
            r#"---
title: "Goblin Ambush - Module Overview"
type: module_overview
---

# Goblin Ambush

## Overview

The party encounters a goblin ambush on the Triboar Trail while escorting supplies to Phandalin. This serves as the adventure's opening encounter and hook into the main plot.

## Key Objectives

- Survive the goblin ambush
- Discover the captured dwarf Gundren Rockseeker was taken to Cragmaw Hideout
- Follow the goblin trail to rescue Gundren's bodyguard Sildar Hallwinter

## Encounters

### The Ambush (Road)
- **Trigger**: The party discovers two dead horses blocking the trail
- **Enemies**: 4 Goblins hiding in the woods, 2 Wolves
- **Tactics**: Goblins attack from cover, wolves rush melee targets
- **Difficulty**: Medium for level 1 party

### Secondary Ambush (Woods)
- **Location**: If party pursues fleeing goblins
- **Enemies**: 2 Goblins with snare traps
- **Complication**: Pit trap (DC 10 Perception to spot)

## Important NPCs

- **Gundren Rockseeker** (mentioned) - Dwarf entrepreneur who hired the party
- **Sildar Hallwinter** (mentioned) - Human warrior escorting Gundren

## Treasure

- 25 gp in a belt pouch on one of the horses
- Empty map case (map was taken by goblins)
- Trail supplies worth 50 gp

## Hooks to Next Module

- Goblin trail leads northeast to Cragmaw Hideout
- One goblin can be captured and interrogated for information
- Horse brands identify them as belonging to Gundren Rockseeker

## DM Notes

- This encounter establishes the threat level and introduces combat
- Allow creative solutions - parley, stealth, or combat all work
- Emphasize the mystery of who was taken and why
"#,
        ),
        (
            "Cragmaw Hideout",
            r#"---
title: "Cragmaw Hideout - Module Overview"
type: module_overview
---

# Cragmaw Hideout

## Overview

A goblin lair hidden in a cave system where Sildar Hallwinter is being held prisoner. The hideout is controlled by Klarg, a bugbear working for the mysterious Black Spider.

## Key Objectives

- Rescue Sildar Hallwinter from the goblins
- Learn about the Black Spider's involvement
- Discover Gundren was taken to Cragmaw Castle
- Recover stolen supplies and treasure

## Dungeon Overview

The hideout consists of several connected cave chambers:

1. **Cave Entrance** - Thicket-hidden entrance with goblin guards
2. **Kennel** - Wolves chained as guard animals
3. **Guard Post** - Elevated platform with archer goblins
4. **Twin Pools** - Water reservoir with flood trap potential
5. **Main Chamber** - Goblin common area with resident Mage
6. **Boss Chamber** - Voaraghamanthar's lair (Adult Black Dragon!)

## Encounters

### Cave Entrance
- **Enemies**: 2 Goblin sentries
- **Tactics**: One flees to warn others if spotted
- **Hazard**: Stream makes stealthy approach difficult

### Kennel
- **Enemies**: 2 Wolves (chained)
- **Note**: Wolves alert goblins with howling if agitated
- **Opportunity**: Can be bypassed or fed to pacify

### Guard Post
- **Enemies**: 3 Goblins with shortbows
- **Advantage**: Elevated position, half cover
- **Tactics**: Fire at intruders, call for reinforcements

### Main Chamber
- **Enemies**: 5 Goblins + 1 Mage (the goblin's magical advisor)
- **Spellcaster**: The Mage knows fireball, counterspell, and shield
- **Complication**: Yeemik threatens to kill Sildar
- **Opportunity**: Negotiate - Yeemik wants Voaraghamanthar dead

### Boss Chamber
- **Enemies**: Voaraghamanthar (Adult Black Dragon), 2 Goblins
- **Legendary Actions**: Detect, Tail Attack, Wing Attack (2 actions)
- **Breath Weapon**: 60 ft. line, 54 (12d8) acid damage, DC 18 Dex save
- **Treasure**: Dragon hoard, stolen goods
- **Difficulty**: DEADLY - This is a CR 14 creature. TPK likely for low-level parties.

## Important NPCs

- **Sildar Hallwinter** - Captive, member of Lords' Alliance, knows about Wave Echo Cave
- **Voaraghamanthar** - Adult Black Dragon, has dominated this goblin tribe
- **The Mage** - Human spellcaster allied with the dragon
- **Yeemik** - Ambitious goblin, wants to overthrow the dragon (good luck)

## Treasure

- 600 cp, 110 sp, 2 potions of healing
- Jade statuette of a frog (40 gp)
- Stolen Lionshield Coster supplies (50 gp reward)
- Sildar's gear (longsword, chainmail)

## Environmental Features

- **Flood Trap**: Dam in Twin Pools can be released
- **Chimney**: Natural shaft to Boss Chamber
- **Fissure**: Connects Guard Post to Twin Pools

## Hooks to Next Module

- Sildar asks party to escort him to Phandalin
- Information about Cragmaw Castle location
- Mention of the Black Spider seeking Wave Echo Cave
- Lionshield supplies can be returned for reward in Phandalin
"#,
        ),
    ];

    for module in modules {
        if let Some((_, content)) = module_content.iter().find(|(name, _)| *name == module.name) {
            // Build the file path for the module overview
            let module_dir = PathBuf::from(campaign_directory)
                .join("modules")
                .join(format!("module_{:02}", module.module_number));
            let overview_path = module_dir.join("module-overview.md");

            // Write the content
            let doc_service = DocumentService::new(conn);
            doc_service.save_document_file(&overview_path.to_string_lossy(), content)?;
        }
    }

    Ok(())
}

/// Seed session notes documents for campaign summary testing
fn seed_session_notes(
    conn: &mut DbConnection,
    campaign_id: i32,
    campaign_directory: &str,
) -> Result<()> {
    use crate::models::campaign::documents::NewDocument;
    use std::path::PathBuf;

    let session_notes = vec![
        (
            1,
            "Session 1: The Road to Phandalin",
            r#"# Session 1: The Road to Phandalin

## Date: Campaign Start

## Summary

The party was hired by Gundren Rockseeker, a dwarf entrepreneur, to escort a wagon of supplies from Neverwinter to Phandalin. Gundren rode ahead with his bodyguard Sildar Hallwinter to "take care of business."

## Key Events

1. **The Ambush**: On the Triboar Trail, the party discovered two dead horses blocking the road - they belonged to Gundren and Sildar. Goblins ambushed the party from the treeline.

2. **Combat**: The party fought off 4 goblins and discovered a trail leading into the forest. One goblin was captured and revealed that "King Grol" had ordered them to capture the dwarf.

3. **Following the Trail**: The party followed the goblin trail, avoiding a snare trap, to find Cragmaw Hideout.

## NPCs Met
- Gundren Rockseeker (mentioned, captured)
- Sildar Hallwinter (mentioned, captured)

## Loot
- 25 gp from dead horses' saddlebags
- Empty map case (the map was taken)

## Clues Discovered
- Gundren was taken to "Cragmaw Castle" by order of "The Black Spider"
- The goblins work for a bugbear named Klarg at the hideout

## Party Status
All party members at full health after the encounter.
"#,
        ),
        (
            2,
            "Session 2: Cragmaw Hideout",
            r#"# Session 2: Cragmaw Hideout

## Date: One week after session 1

## Summary

The party infiltrated Cragmaw Hideout to rescue Sildar Hallwinter and gather information about Gundren Rockseeker's whereabouts.

## Key Events

1. **Cave Entrance**: The party dealt with goblin sentries and discovered a cave system with a stream running through it.

2. **Flood Trap**: The goblins released dammed water to flood the passage. Quick thinking saved the party from being washed away.

3. **Sildar Rescued**: Found Sildar Hallwinter held prisoner in the goblin den. He was beaten but alive. He revealed he's a member of the Lords' Alliance.

4. **Klarg Defeated**: The party confronted Klarg the bugbear in his lair. After a fierce battle, Klarg was slain.

## NPCs Met
- Sildar Hallwinter (rescued) - Lords' Alliance agent
- Klarg (defeated) - Bugbear leader of the hideout

## Important Information from Sildar
- Gundren and his brothers discovered the location of Wave Echo Cave
- Wave Echo Cave contains the legendary Forge of Spells
- Someone called "The Black Spider" wants this information
- Gundren was taken to Cragmaw Castle, location unknown
- Sildar asks the party to help him reach Phandalin and find his contact Iarno Albrek

## Loot
- Klarg's treasure chest: 600 cp, 110 sp, two potions of healing
- Supplies marked with a blue lion (Lionshield Coster goods)

## Current Objectives
- Escort Sildar to Phandalin
- Find Iarno Albrek at Phandalin
- Discover the location of Cragmaw Castle
- Learn more about the Black Spider

## Party Status
Helena used most of her spell slots healing the party.
"#,
        ),
        (
            3,
            "Session 3: Welcome to Phandalin",
            r#"# Session 3: Welcome to Phandalin

## Date: Arrival in Phandalin

## Summary

The party arrived in Phandalin with Sildar and delivered the supplies. They discovered the town is being terrorized by a gang called the Redbrands.

## Key Events

1. **Arrival**: Delivered supplies to Barthen's Provisions. Elmar Barthen paid the agreed 10 gp each and expressed concern about his friend Gundren.

2. **Town Investigation**: The party learned about the Redbrands - a gang of ruffians in red cloaks who have been extorting townspeople.

3. **Stonehill Inn**: At the inn, Toblen Stonehill shared that the Redbrands frequent the Sleeping Giant tap house. A local woodcarver named Thel Dendrar stood up to them and hasn't been seen since.

4. **Confrontation**: The party encountered Redbrand ruffians who tried to shake them down. A fight ensued, and the party captured one for questioning.

## NPCs Met
- Elmar Barthen (merchant, friend of Gundren)
- Toblen Stonehill (innkeeper)
- Linene Graywind (Lionshield Coster - grateful for recovered goods)
- Daran Edermath (retired adventurer, warned about Redbrands)

## Important Information
- The Redbrands are led by someone called "Glasstaff"
- Their hideout is beneath Tresendar Manor on the east side of town
- Iarno Albrek (Sildar's contact) hasn't been seen in Phandalin
- Sister Garaele at the shrine has a task for capable adventurers
- Halia Thornton at the Miner's Exchange has offered a bounty on Glasstaff

## Current Objectives
- Investigate the Redbrands
- Find Glasstaff
- Learn what happened to Thel Dendrar
- Find the location of Cragmaw Castle
- Look for Iarno Albrek (missing Lords' Alliance agent)

## Party Status
Full health, well-rested after a night at Stonehill Inn.
"#,
        ),
    ];

    for (session_number, title, content) in session_notes {
        // Create the document in the database
        let file_path = format!("sessions/session_{}_notes.md", session_number);
        let full_path = PathBuf::from(campaign_directory).join(&file_path);

        // Ensure directory exists
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write the file
        std::fs::write(&full_path, content)?;

        // Create document record
        // Note: session_id is None because we don't have actual session records,
        // just session notes documents. The document_type identifies these as session notes.
        let new_doc = NewDocument {
            campaign_id,
            module_id: None,
            session_id: None,
            template_id: format!("session_notes_{}", session_number),
            document_type: "session_notes".to_string(),
            title: title.to_string(),
            file_path,
        };

        let mut service = DocumentService::new(conn);
        service.create_document(new_doc)?;
    }

    Ok(())
}

/// Seed campaign summary from session notes
///
/// Creates a pre-written summary that matches the session notes content.
/// This provides deterministic test data without requiring LLM calls.
fn seed_campaign_summary(
    conn: &mut DbConnection,
    campaign_id: i32,
    campaign_directory: &str,
) -> Result<()> {
    use crate::services::campaign_summary_service::CampaignSummary;

    // Pre-written summary that matches the session notes content
    let summary_text = r#"The party began by escorting Gundren Rockseeker's wagon from Neverwinter to Phandalin, where a goblin ambush on the Triboar Trail led to the capture of Gundren and Sildar Hallwinter. They followed the goblins' trail, evaded a snare, and entered the Cragmaw Hideout. Inside, the party rescued Sildar, defeated the bugbear leader Klarg, and learned that Gundren had discovered Wave Echo Cave—home of the legendary Forge of Spells—and had been taken to a place called Cragmaw Castle by an ominous figure known as the Black Spider.

After escaping the hideout, the group escorted Sildar to Phandalin, where they delivered the wagon goods to Barthen's Provisions and uncovered a town in distress. They learned of the Redbrands, a red‑cloaked gang extorting the townsfolk, and that their leader was a mysterious "Glasstaff" whose hideout lies beneath Tresendar Manor. They captured a Redbrand thug to interrogate and discovered that Thel Dendrar—the only townsperson who confronted the bad guys—disappeared after confronting them. Halia Thornton at the Miner's Exchange and Sister Garaele at the shrine offered assistance in dealing with the Redbrands.

Presently, the party is in Phandalin, investigating the Redbrands, locating Glasstaff, and searching for the missing Iarno Albrek. They also need to find Cragmaw Castle's location and learn more about the Black Spider's motives. Their main immediate objectives are to uncover the Redbrand base, recover Thel Dendrar, and identify Cragmaw Castle's whereabouts so they can rescue Gundren.

Key NPCs encountered include the captured dwarf Gundren Rockseeker, the loyal Sildar Hallwinter, the bugbear leader Klarg, the merchant Elmar Barthen, the innkeeper Toblen Stonehill, the secretive Iarno Albrek, the Redbrand leader Glasstaff, the informant Linene Graywind, the former adventurer Daran Edermath, and the helpful figures Halia Thornton and Sister Garaele."#;

    // Gather source materials to compute hash
    let mut summary_service = CampaignSummaryService::new(conn);
    let source = summary_service.gather_source_materials(campaign_id, campaign_directory)?;
    let source_hash = CampaignSummaryService::calculate_source_hash(&source);

    let summary = CampaignSummary {
        summary: summary_text.to_string(),
        generated_at: Utc::now().to_rfc3339(),
        source_hash,
        campaign_id,
    };

    summary_service.save_summary(campaign_directory, &summary)?;

    Ok(())
}

/// Seed test players
fn seed_players(conn: &mut DbConnection) -> Result<Vec<crate::models::player::Player>> {
    let mut players = Vec::new();

    let player_data = [
        (
            "Alice",
            Some("alice@test.com"),
            Some("Experienced player, loves tactical combat"),
        ),
        (
            "Bob",
            Some("bob@test.com"),
            Some("Creative roleplayer, enjoys magic users"),
        ),
        (
            "Charlie",
            Some("charlie@test.com"),
            Some("New to D&D, learning the ropes"),
        ),
        (
            "Diana",
            Some("diana@test.com"),
            Some("Forever DM trying player side"),
        ),
    ];

    for (name, email, notes) in player_data {
        let mut service = PlayerService::new(conn);
        let player =
            service.create_player(name, email.map(String::from), notes.map(String::from))?;
        players.push(player);
    }

    Ok(players)
}

/// Seed test characters
fn seed_characters(
    conn: &mut DbConnection,
    campaign_id: Option<i32>,
    base_directory: &str,
    players: &[crate::models::player::Player],
) -> Result<Vec<crate::models::character::Character>> {
    let mut characters = Vec::new();
    let now = Utc::now().to_rfc3339();

    // Map player names to IDs for character assignment
    let player_map: HashMap<&str, i32> = players.iter().map(|p| (p.name.as_str(), p.id)).collect();

    // Thorin Ironforge - Level 5 Dwarf Fighter (Alice's character)
    if let Some(&player_id) = player_map.get("Alice") {
        let character_data = create_thorin(player_id, &now);
        let mut service = CharacterService::new(conn);
        let character =
            service.create_character(campaign_id, Some(player_id), false, base_directory, character_data)?;
        characters.push(character);
    }

    // Elara Moonwhisper - Level 5 Elf Wizard (Bob's character)
    if let Some(&player_id) = player_map.get("Bob") {
        let character_data = create_elara(player_id, &now);
        let mut service = CharacterService::new(conn);
        let character =
            service.create_character(campaign_id, Some(player_id), false, base_directory, character_data)?;
        characters.push(character);
    }

    // Finn Lightfoot - Level 1 Halfling Rogue (Charlie's character)
    if let Some(&player_id) = player_map.get("Charlie") {
        let character_data = create_finn(player_id, &now);
        let mut service = CharacterService::new(conn);
        let character =
            service.create_character(campaign_id, Some(player_id), false, base_directory, character_data)?;
        characters.push(character);
    }

    // Sister Helena - Level 10 Human Cleric (Diana's character)
    if let Some(&player_id) = player_map.get("Diana") {
        let character_data = create_helena(player_id, &now);
        let mut service = CharacterService::new(conn);
        let character =
            service.create_character(campaign_id, Some(player_id), false, base_directory, character_data)?;
        characters.push(character);
    }

    // Create NPCs
    let npc_characters = seed_npcs(conn, campaign_id, base_directory, &now)?;
    characters.extend(npc_characters);

    Ok(characters)
}

/// Seed test NPCs
fn seed_npcs(
    conn: &mut DbConnection,
    campaign_id: Option<i32>,
    base_directory: &str,
    created_at: &str,
) -> Result<Vec<crate::models::character::Character>> {
    let mut npcs = Vec::new();

    // Sildar Hallwinter - Human Fighter, Lords' Alliance
    let sildar = create_sildar(created_at);
    let mut service = CharacterService::new(conn);
    let character = service.create_character(campaign_id, None, true, base_directory, sildar)?;
    npcs.push(character);

    // Gundren Rockseeker - Dwarf quest giver
    let gundren = create_gundren(created_at);
    let mut service = CharacterService::new(conn);
    let character = service.create_character(campaign_id, None, true, base_directory, gundren)?;
    npcs.push(character);

    // Toblen Stonehill - Innkeeper
    let toblen = create_toblen(created_at);
    let mut service = CharacterService::new(conn);
    let character = service.create_character(campaign_id, None, true, base_directory, toblen)?;
    npcs.push(character);

    // Iarno Albrek (Glasstaff) - Human Wizard, villain
    let iarno = create_iarno(created_at);
    let mut service = CharacterService::new(conn);
    let character = service.create_character(campaign_id, None, true, base_directory, iarno)?;
    npcs.push(character);

    Ok(npcs)
}

/// Create Thorin Ironforge - Level 5 Dwarf Fighter
fn create_thorin(player_id: i32, created_at: &str) -> CharacterData {
    CharacterData {
        character_name: "Thorin Ironforge".to_string(),
        player_id: Some(player_id),
        level: 5,
        experience_points: 6500,
        version: 1,
        snapshot_reason: Some("Dev seed character".to_string()),
        created_at: created_at.to_string(),
        race: "Dwarf".to_string(),
        subrace: Some("Mountain".to_string()),
        classes: vec![ClassLevel {
            class_name: "Fighter".to_string(),
            level: 5,
            subclass: Some("Champion".to_string()),
            hit_dice_type: "d10".to_string(),
            hit_dice_remaining: 5,
        }],
        background: "Soldier".to_string(),
        alignment: Some("Lawful Good".to_string()),
        abilities: AbilityScores {
            strength: 18,
            dexterity: 12,
            constitution: 16,
            intelligence: 10,
            wisdom: 13,
            charisma: 8,
        },
        max_hp: 49,
        current_hp: 49,
        speed: 25,
        proficiencies: Proficiencies {
            skills: vec![
                "Athletics".to_string(),
                "Intimidation".to_string(),
                "Perception".to_string(),
                "Survival".to_string(),
            ],
            saves: vec!["Strength".to_string(), "Constitution".to_string()],
            armor: vec![
                "Light armor".to_string(),
                "Medium armor".to_string(),
                "Heavy armor".to_string(),
                "Shields".to_string(),
            ],
            weapons: vec!["Simple weapons".to_string(), "Martial weapons".to_string()],
            tools: vec!["Smith's tools".to_string(), "Dice set".to_string()],
            languages: vec!["Common".to_string(), "Dwarvish".to_string()],
        },
        class_features: vec![
            FeatureReference::new("Fighting Style", "Fighter", "PHB", 1),
            FeatureReference::new("Second Wind", "Fighter", "PHB", 1),
            FeatureReference::new("Action Surge", "Fighter", "PHB", 2),
            FeatureReference::with_subclass("Improved Critical", "Fighter", "Champion", "PHB", 3),
            FeatureReference::new("Extra Attack", "Fighter", "PHB", 5),
        ],
        feats: vec![],
        spells: SpellData::default(),
        inventory: vec![
            InventoryItem {
                name: "Chain Mail".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 55.0,
                value: 75.0,
                notes: Some("AC 16".to_string()),
            },
            InventoryItem {
                name: "Battleaxe".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 4.0,
                value: 10.0,
                notes: Some("1d8 slashing, versatile (1d10)".to_string()),
            },
            InventoryItem {
                name: "Shield".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 6.0,
                value: 10.0,
                notes: Some("+2 AC".to_string()),
            },
            InventoryItem {
                name: "Handaxe".to_string(),
                source: Some("PHB".to_string()),
                quantity: 2,
                weight: 2.0,
                value: 5.0,
                notes: Some("1d6 slashing, light, thrown".to_string()),
            },
            InventoryItem {
                name: "Explorer's Pack".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 59.0,
                value: 10.0,
                notes: None,
            },
        ],
        currency: Currency {
            copper: 0,
            silver: 15,
            electrum: 0,
            gold: 45,
            platinum: 0,
        },
        equipped: EquippedItems {
            armor: Some("Chain Mail".to_string()),
            shield: Some("Shield".to_string()),
            main_hand: Some("Battleaxe".to_string()),
            off_hand: None,
        },
        personality: Personality {
            traits: Some("I face problems head-on. A simple, direct solution is the best path to success.".to_string()),
            ideals: Some("Responsibility. I do what I must and obey just authority.".to_string()),
            bonds: Some("I would still lay down my life for the people I served with.".to_string()),
            flaws: Some("I made a terrible mistake in battle that cost many lives, and I would do anything to keep that mistake secret.".to_string()),
        },
        player_name: None,
        appearance: Appearance::default(),
        backstory: None,
        background_feature: None,
        roleplay_notes: RoleplayNotes::default(),
        npc_role: None,
        npc_location: None,
        npc_faction: None,
        npc_notes: None,
        legendary_actions: Vec::new(),
        legendary_action_count: None,
    }
}

/// Create Elara Moonwhisper - Level 5 Elf Wizard
fn create_elara(player_id: i32, created_at: &str) -> CharacterData {
    let mut spell_slots = HashMap::new();
    spell_slots.insert(1, SpellSlots::new(4));
    spell_slots.insert(2, SpellSlots::new(3));
    spell_slots.insert(3, SpellSlots::new(2));

    CharacterData {
        character_name: "Elara Moonwhisper".to_string(),
        player_id: Some(player_id),
        level: 5,
        experience_points: 6500,
        version: 1,
        snapshot_reason: Some("Dev seed character".to_string()),
        created_at: created_at.to_string(),
        race: "Elf".to_string(),
        subrace: Some("High".to_string()),
        classes: vec![ClassLevel {
            class_name: "Wizard".to_string(),
            level: 5,
            subclass: Some("School of Evocation".to_string()),
            hit_dice_type: "d6".to_string(),
            hit_dice_remaining: 5,
        }],
        background: "Sage".to_string(),
        alignment: Some("Neutral Good".to_string()),
        abilities: AbilityScores {
            strength: 8,
            dexterity: 14,
            constitution: 13,
            intelligence: 18,
            wisdom: 12,
            charisma: 10,
        },
        max_hp: 27,
        current_hp: 27,
        speed: 30,
        proficiencies: Proficiencies {
            skills: vec![
                "Arcana".to_string(),
                "History".to_string(),
                "Investigation".to_string(),
                "Perception".to_string(),
            ],
            saves: vec!["Intelligence".to_string(), "Wisdom".to_string()],
            armor: vec![],
            weapons: vec![
                "Daggers".to_string(),
                "Darts".to_string(),
                "Slings".to_string(),
                "Quarterstaffs".to_string(),
                "Light crossbows".to_string(),
                "Longsword".to_string(),
                "Shortsword".to_string(),
                "Shortbow".to_string(),
                "Longbow".to_string(),
            ],
            tools: vec![],
            languages: vec![
                "Common".to_string(),
                "Elvish".to_string(),
                "Draconic".to_string(),
                "Celestial".to_string(),
            ],
        },
        class_features: vec![
            FeatureReference::new("Arcane Recovery", "Wizard", "PHB", 1),
            FeatureReference::with_subclass("Evocation Savant", "Wizard", "Evocation", "PHB", 2),
            FeatureReference::with_subclass("Sculpt Spells", "Wizard", "Evocation", "PHB", 2),
        ],
        feats: vec![],
        spells: SpellData {
            cantrips: vec![
                SpellReference::new("Fire Bolt", "PHB"),
                SpellReference::new("Light", "PHB"),
                SpellReference::new("Mage Hand", "PHB"),
                SpellReference::new("Prestidigitation", "PHB"),
            ],
            known_spells: vec![
                SpellReference::new("Magic Missile", "PHB"),
                SpellReference::new("Shield", "PHB"),
                SpellReference::new("Mage Armor", "PHB"),
                SpellReference::new("Detect Magic", "PHB"),
                SpellReference::new("Identify", "PHB"),
                SpellReference::new("Misty Step", "PHB"),
                SpellReference::new("Scorching Ray", "PHB"),
                SpellReference::new("Shatter", "PHB"),
                SpellReference::new("Fireball", "PHB"),
                SpellReference::new("Counterspell", "PHB"),
            ],
            prepared_spells: vec![
                SpellReference::new("Magic Missile", "PHB"),
                SpellReference::new("Shield", "PHB"),
                SpellReference::new("Mage Armor", "PHB"),
                SpellReference::new("Misty Step", "PHB"),
                SpellReference::new("Fireball", "PHB"),
                SpellReference::new("Counterspell", "PHB"),
            ],
            spell_slots,
        },
        inventory: vec![
            InventoryItem {
                name: "Quarterstaff".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 4.0,
                value: 0.2,
                notes: Some("Arcane focus".to_string()),
            },
            InventoryItem {
                name: "Spellbook".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 3.0,
                value: 50.0,
                notes: Some("Contains all known spells".to_string()),
            },
            InventoryItem {
                name: "Component Pouch".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 2.0,
                value: 25.0,
                notes: None,
            },
            InventoryItem {
                name: "Scholar's Pack".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 10.0,
                value: 40.0,
                notes: None,
            },
        ],
        currency: Currency {
            copper: 0,
            silver: 0,
            electrum: 0,
            gold: 75,
            platinum: 0,
        },
        equipped: EquippedItems {
            armor: None,
            shield: None,
            main_hand: Some("Quarterstaff".to_string()),
            off_hand: None,
        },
        personality: Personality {
            traits: Some("I use polysyllabic words that convey the impression of great erudition.".to_string()),
            ideals: Some("Knowledge. The path to power and self-improvement is through knowledge.".to_string()),
            bonds: Some("I have an ancient text that holds terrible secrets that must not fall into the wrong hands.".to_string()),
            flaws: Some("I overlook obvious solutions in favor of complicated ones.".to_string()),
        },
        player_name: None,
        appearance: Appearance::default(),
        backstory: None,
        background_feature: None,
        roleplay_notes: RoleplayNotes::default(),
        npc_role: None,
        npc_location: None,
        npc_faction: None,
        npc_notes: None,
        legendary_actions: Vec::new(),
        legendary_action_count: None,
    }
}

/// Create Finn Lightfoot - Level 1 Halfling Rogue
fn create_finn(player_id: i32, created_at: &str) -> CharacterData {
    CharacterData {
        character_name: "Finn Lightfoot".to_string(),
        player_id: Some(player_id),
        level: 1,
        experience_points: 0,
        version: 1,
        snapshot_reason: Some("Dev seed character".to_string()),
        created_at: created_at.to_string(),
        race: "Halfling".to_string(),
        subrace: Some("Lightfoot".to_string()),
        classes: vec![ClassLevel {
            class_name: "Rogue".to_string(),
            level: 1,
            subclass: None,
            hit_dice_type: "d8".to_string(),
            hit_dice_remaining: 1,
        }],
        background: "Criminal".to_string(),
        alignment: Some("Chaotic Neutral".to_string()),
        abilities: AbilityScores {
            strength: 8,
            dexterity: 17,
            constitution: 12,
            intelligence: 13,
            wisdom: 10,
            charisma: 14,
        },
        max_hp: 9,
        current_hp: 9,
        speed: 25,
        proficiencies: Proficiencies {
            skills: vec![
                "Acrobatics".to_string(),
                "Deception".to_string(),
                "Sleight of Hand".to_string(),
                "Stealth".to_string(),
            ],
            saves: vec!["Dexterity".to_string(), "Intelligence".to_string()],
            armor: vec!["Light armor".to_string()],
            weapons: vec![
                "Simple weapons".to_string(),
                "Hand crossbows".to_string(),
                "Longswords".to_string(),
                "Rapiers".to_string(),
                "Shortswords".to_string(),
            ],
            tools: vec!["Thieves' tools".to_string(), "Dice set".to_string()],
            languages: vec![
                "Common".to_string(),
                "Halfling".to_string(),
                "Thieves' Cant".to_string(),
            ],
        },
        class_features: vec![
            FeatureReference::new("Expertise", "Rogue", "PHB", 1),
            FeatureReference::new("Sneak Attack", "Rogue", "PHB", 1),
            FeatureReference::new("Thieves' Cant", "Rogue", "PHB", 1),
        ],
        feats: vec![],
        spells: SpellData::default(),
        inventory: vec![
            InventoryItem {
                name: "Leather Armor".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 10.0,
                value: 10.0,
                notes: Some("AC 11 + Dex".to_string()),
            },
            InventoryItem {
                name: "Shortsword".to_string(),
                source: Some("PHB".to_string()),
                quantity: 2,
                weight: 2.0,
                value: 10.0,
                notes: Some("1d6 piercing, finesse, light".to_string()),
            },
            InventoryItem {
                name: "Thieves' Tools".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 1.0,
                value: 25.0,
                notes: None,
            },
            InventoryItem {
                name: "Burglar's Pack".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 44.0,
                value: 16.0,
                notes: None,
            },
        ],
        currency: Currency {
            copper: 0,
            silver: 0,
            electrum: 0,
            gold: 15,
            platinum: 0,
        },
        equipped: EquippedItems {
            armor: Some("Leather Armor".to_string()),
            shield: None,
            main_hand: Some("Shortsword".to_string()),
            off_hand: Some("Shortsword".to_string()),
        },
        personality: Personality {
            traits: Some("I always have a plan for what to do when things go wrong.".to_string()),
            ideals: Some(
                "Freedom. Chains are meant to be broken, as are those who would forge them."
                    .to_string(),
            ),
            bonds: Some(
                "I'm trying to pay off an old debt I owe to a generous benefactor.".to_string(),
            ),
            flaws: Some(
                "When I see something valuable, I can't think about anything but how to steal it."
                    .to_string(),
            ),
        },
        player_name: None,
        appearance: Appearance::default(),
        backstory: None,
        background_feature: None,
        roleplay_notes: RoleplayNotes::default(),
        npc_role: None,
        npc_location: None,
        npc_faction: None,
        npc_notes: None,
        legendary_actions: Vec::new(),
        legendary_action_count: None,
    }
}

/// Create Sister Helena - Level 10 Human Cleric
fn create_helena(player_id: i32, created_at: &str) -> CharacterData {
    let mut spell_slots = HashMap::new();
    spell_slots.insert(1, SpellSlots::new(4));
    spell_slots.insert(2, SpellSlots::new(3));
    spell_slots.insert(3, SpellSlots::new(3));
    spell_slots.insert(4, SpellSlots::new(3));
    spell_slots.insert(5, SpellSlots::new(2));

    CharacterData {
        character_name: "Sister Helena".to_string(),
        player_id: Some(player_id),
        level: 10,
        experience_points: 64000,
        version: 1,
        snapshot_reason: Some("Dev seed character".to_string()),
        created_at: created_at.to_string(),
        race: "Human".to_string(),
        subrace: None,
        classes: vec![ClassLevel {
            class_name: "Cleric".to_string(),
            level: 10,
            subclass: Some("Life Domain".to_string()),
            hit_dice_type: "d8".to_string(),
            hit_dice_remaining: 10,
        }],
        background: "Acolyte".to_string(),
        alignment: Some("Lawful Good".to_string()),
        abilities: AbilityScores {
            strength: 14,
            dexterity: 10,
            constitution: 14,
            intelligence: 10,
            wisdom: 18,
            charisma: 12,
        },
        max_hp: 73,
        current_hp: 73,
        speed: 30,
        proficiencies: Proficiencies {
            skills: vec![
                "Insight".to_string(),
                "Medicine".to_string(),
                "Persuasion".to_string(),
                "Religion".to_string(),
            ],
            saves: vec!["Wisdom".to_string(), "Charisma".to_string()],
            armor: vec![
                "Light armor".to_string(),
                "Medium armor".to_string(),
                "Heavy armor".to_string(),
                "Shields".to_string(),
            ],
            weapons: vec!["Simple weapons".to_string()],
            tools: vec![],
            languages: vec![
                "Common".to_string(),
                "Celestial".to_string(),
                "Dwarvish".to_string(),
            ],
        },
        class_features: vec![
            FeatureReference::with_subclass("Disciple of Life", "Cleric", "Life", "PHB", 1),
            FeatureReference::new("Channel Divinity", "Cleric", "PHB", 2),
            FeatureReference::with_subclass("Preserve Life", "Cleric", "Life", "PHB", 2),
            FeatureReference::with_subclass("Blessed Healer", "Cleric", "Life", "PHB", 6),
            FeatureReference::with_subclass("Divine Strike", "Cleric", "Life", "PHB", 8),
            FeatureReference::new("Destroy Undead", "Cleric", "PHB", 5),
            FeatureReference::new("Divine Intervention", "Cleric", "PHB", 10),
        ],
        feats: vec!["War Caster".to_string()],
        spells: SpellData {
            cantrips: vec![
                SpellReference::new("Guidance", "PHB"),
                SpellReference::new("Light", "PHB"),
                SpellReference::new("Sacred Flame", "PHB"),
                SpellReference::new("Spare the Dying", "PHB"),
                SpellReference::new("Thaumaturgy", "PHB"),
            ],
            known_spells: vec![
                // Domain spells (always prepared)
                SpellReference::new("Bless", "PHB"),
                SpellReference::new("Cure Wounds", "PHB"),
                SpellReference::new("Lesser Restoration", "PHB"),
                SpellReference::new("Spiritual Weapon", "PHB"),
                SpellReference::new("Beacon of Hope", "PHB"),
                SpellReference::new("Revivify", "PHB"),
                SpellReference::new("Death Ward", "PHB"),
                SpellReference::new("Guardian of Faith", "PHB"),
                SpellReference::new("Mass Cure Wounds", "PHB"),
                SpellReference::new("Raise Dead", "PHB"),
                // Other prepared spells
                SpellReference::new("Healing Word", "PHB"),
                SpellReference::new("Shield of Faith", "PHB"),
                SpellReference::new("Aid", "PHB"),
                SpellReference::new("Prayer of Healing", "PHB"),
                SpellReference::new("Spirit Guardians", "PHB"),
                SpellReference::new("Banishment", "PHB"),
                SpellReference::new("Holy Weapon", "XGE"),
            ],
            prepared_spells: vec![
                SpellReference::new("Bless", "PHB"),
                SpellReference::new("Cure Wounds", "PHB"),
                SpellReference::new("Healing Word", "PHB"),
                SpellReference::new("Shield of Faith", "PHB"),
                SpellReference::new("Lesser Restoration", "PHB"),
                SpellReference::new("Spiritual Weapon", "PHB"),
                SpellReference::new("Aid", "PHB"),
                SpellReference::new("Beacon of Hope", "PHB"),
                SpellReference::new("Revivify", "PHB"),
                SpellReference::new("Spirit Guardians", "PHB"),
                SpellReference::new("Death Ward", "PHB"),
                SpellReference::new("Banishment", "PHB"),
                SpellReference::new("Mass Cure Wounds", "PHB"),
                SpellReference::new("Holy Weapon", "XGE"),
            ],
            spell_slots,
        },
        inventory: vec![
            InventoryItem {
                name: "Plate Armor".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 65.0,
                value: 1500.0,
                notes: Some("AC 18".to_string()),
            },
            InventoryItem {
                name: "Shield".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 6.0,
                value: 10.0,
                notes: Some("+2 AC, holy symbol emblazoned".to_string()),
            },
            InventoryItem {
                name: "Mace".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 4.0,
                value: 5.0,
                notes: Some("1d6 bludgeoning".to_string()),
            },
            InventoryItem {
                name: "Holy Symbol".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 0.0,
                value: 5.0,
                notes: Some("Amulet of Lathander".to_string()),
            },
            InventoryItem {
                name: "Priest's Pack".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 24.0,
                value: 19.0,
                notes: None,
            },
            InventoryItem {
                name: "Diamond".to_string(),
                source: Some("PHB".to_string()),
                quantity: 3,
                weight: 0.0,
                value: 300.0,
                notes: Some("For Revivify spell component".to_string()),
            },
        ],
        currency: Currency {
            copper: 0,
            silver: 0,
            electrum: 0,
            gold: 250,
            platinum: 10,
        },
        equipped: EquippedItems {
            armor: Some("Plate Armor".to_string()),
            shield: Some("Shield".to_string()),
            main_hand: Some("Mace".to_string()),
            off_hand: None,
        },
        personality: Personality {
            traits: Some("I see omens in every event and action. The gods try to speak to us, we just need to listen.".to_string()),
            ideals: Some("Charity. I always try to help those in need, no matter what the personal cost.".to_string()),
            bonds: Some("I will do anything to protect the temple where I served.".to_string()),
            flaws: Some("I put too much trust in those who wield power within my temple's hierarchy.".to_string()),
        },
        player_name: None,
        appearance: Appearance::default(),
        backstory: None,
        background_feature: None,
        roleplay_notes: RoleplayNotes::default(),
        npc_role: None,
        npc_location: None,
        npc_faction: None,
        npc_notes: None,
        legendary_actions: Vec::new(),
        legendary_action_count: None,
    }
}

/// Create Sildar Hallwinter - Human Fighter NPC (Lords' Alliance)
fn create_sildar(created_at: &str) -> CharacterData {
    CharacterData {
        character_name: "Sildar Hallwinter".to_string(),
        player_id: None, // NPC
        level: 5,
        experience_points: 0,
        version: 1,
        snapshot_reason: Some("Dev seed NPC".to_string()),
        created_at: created_at.to_string(),
        race: "Human".to_string(),
        subrace: None,
        classes: vec![ClassLevel {
            class_name: "Fighter".to_string(),
            level: 5,
            subclass: None,
            hit_dice_type: "d10".to_string(),
            hit_dice_remaining: 5,
        }],
        background: "Soldier".to_string(),
        alignment: Some("Lawful Good".to_string()),
        abilities: AbilityScores {
            strength: 14,
            dexterity: 12,
            constitution: 14,
            intelligence: 10,
            wisdom: 11,
            charisma: 12,
        },
        max_hp: 42,
        current_hp: 42,
        speed: 30,
        proficiencies: Proficiencies {
            skills: vec![
                "Athletics".to_string(),
                "Intimidation".to_string(),
                "Perception".to_string(),
            ],
            saves: vec!["Strength".to_string(), "Constitution".to_string()],
            armor: vec!["All armor".to_string(), "Shields".to_string()],
            weapons: vec!["Simple weapons".to_string(), "Martial weapons".to_string()],
            tools: vec![],
            languages: vec!["Common".to_string(), "Orc".to_string()],
        },
        class_features: vec![
            FeatureReference::new("Fighting Style", "Fighter", "PHB", 1),
            FeatureReference::new("Second Wind", "Fighter", "PHB", 1),
            FeatureReference::new("Action Surge", "Fighter", "PHB", 2),
            FeatureReference::new("Extra Attack", "Fighter", "PHB", 5),
        ],
        feats: vec![],
        spells: SpellData::default(),
        inventory: vec![
            InventoryItem {
                name: "Longsword".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 3.0,
                value: 15.0,
                notes: None,
            },
            InventoryItem {
                name: "Chain Mail".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 55.0,
                value: 75.0,
                notes: Some("AC 16".to_string()),
            },
        ],
        currency: Currency {
            copper: 0,
            silver: 0,
            electrum: 0,
            gold: 50,
            platinum: 0,
        },
        equipped: EquippedItems {
            armor: Some("Chain Mail".to_string()),
            shield: None,
            main_hand: Some("Longsword".to_string()),
            off_hand: None,
        },
        personality: Personality {
            traits: Some("I am always calm, no matter what the situation.".to_string()),
            ideals: Some("Honor. I don't lie or cheat. Let me be judged by my actions.".to_string()),
            bonds: Some("My loyalty to the Lords' Alliance is unwavering.".to_string()),
            flaws: Some("I have trouble trusting people outside my organization.".to_string()),
        },
        player_name: None,
        appearance: Appearance::default(),
        backstory: None,
        background_feature: None,
        roleplay_notes: RoleplayNotes::default(),
        npc_role: Some("Lords' Alliance Agent".to_string()),
        npc_location: Some("Phandalin".to_string()),
        npc_faction: Some("Lords' Alliance".to_string()),
        npc_notes: Some("Sildar was escorting Gundren Rockseeker to Phandalin when they were ambushed by goblins. He seeks to establish law and order in Phandalin and find his missing contact, Iarno Albrek.".to_string()),
        legendary_actions: Vec::new(),
        legendary_action_count: None,
    }
}

/// Create Gundren Rockseeker - Dwarf NPC (Quest Giver)
fn create_gundren(created_at: &str) -> CharacterData {
    CharacterData {
        character_name: "Gundren Rockseeker".to_string(),
        player_id: None, // NPC
        level: 3,
        experience_points: 0,
        version: 1,
        snapshot_reason: Some("Dev seed NPC".to_string()),
        created_at: created_at.to_string(),
        race: "Dwarf".to_string(),
        subrace: Some("Hill".to_string()),
        classes: vec![ClassLevel {
            class_name: "Expert".to_string(), // Using generic NPC class
            level: 3,
            subclass: None,
            hit_dice_type: "d8".to_string(),
            hit_dice_remaining: 3,
        }],
        background: "Guild Artisan".to_string(),
        alignment: Some("Neutral Good".to_string()),
        abilities: AbilityScores {
            strength: 12,
            dexterity: 10,
            constitution: 14,
            intelligence: 14,
            wisdom: 12,
            charisma: 13,
        },
        max_hp: 22,
        current_hp: 22,
        speed: 25,
        proficiencies: Proficiencies {
            skills: vec![
                "History".to_string(),
                "Persuasion".to_string(),
                "Investigation".to_string(),
            ],
            saves: vec![],
            armor: vec!["Light armor".to_string()],
            weapons: vec!["Simple weapons".to_string()],
            tools: vec!["Mason's tools".to_string(), "Mining tools".to_string()],
            languages: vec!["Common".to_string(), "Dwarvish".to_string(), "Gnomish".to_string()],
        },
        class_features: vec![],
        feats: vec![],
        spells: SpellData::default(),
        inventory: vec![],
        currency: Currency {
            copper: 0,
            silver: 0,
            electrum: 0,
            gold: 100,
            platinum: 5,
        },
        equipped: EquippedItems::default(),
        personality: Personality {
            traits: Some("I'm full of witty aphorisms and have a proverb for every occasion.".to_string()),
            ideals: Some("Family. Blood runs thicker than water.".to_string()),
            bonds: Some("I will rediscover Wave Echo Cave and restore it to my family's glory.".to_string()),
            flaws: Some("I'm never satisfied with what I have - I always want more.".to_string()),
        },
        player_name: None,
        appearance: Appearance::default(),
        backstory: None,
        background_feature: None,
        roleplay_notes: RoleplayNotes::default(),
        npc_role: Some("Quest Giver / Patron".to_string()),
        npc_location: Some("Captured at Cragmaw Castle".to_string()),
        npc_faction: Some("Rockseeker Clan".to_string()),
        npc_notes: Some("Gundren and his brothers discovered the entrance to Wave Echo Cave, a legendary mine. He hired the party to escort supplies while he traveled ahead with Sildar, but was captured by goblins working for the Black Spider.".to_string()),
        legendary_actions: Vec::new(),
        legendary_action_count: None,
    }
}

/// Create Toblen Stonehill - Human NPC (Innkeeper)
fn create_toblen(created_at: &str) -> CharacterData {
    CharacterData {
        character_name: "Toblen Stonehill".to_string(),
        player_id: None, // NPC
        level: 1,
        experience_points: 0,
        version: 1,
        snapshot_reason: Some("Dev seed NPC".to_string()),
        created_at: created_at.to_string(),
        race: "Human".to_string(),
        subrace: None,
        classes: vec![ClassLevel {
            class_name: "Commoner".to_string(),
            level: 1,
            subclass: None,
            hit_dice_type: "d8".to_string(),
            hit_dice_remaining: 1,
        }],
        background: "Guild Artisan".to_string(),
        alignment: Some("Lawful Good".to_string()),
        abilities: AbilityScores {
            strength: 10,
            dexterity: 10,
            constitution: 12,
            intelligence: 11,
            wisdom: 14,
            charisma: 13,
        },
        max_hp: 8,
        current_hp: 8,
        speed: 30,
        proficiencies: Proficiencies {
            skills: vec!["Insight".to_string(), "Persuasion".to_string()],
            saves: vec![],
            armor: vec![],
            weapons: vec![],
            tools: vec!["Brewer's supplies".to_string(), "Cook's utensils".to_string()],
            languages: vec!["Common".to_string()],
        },
        class_features: vec![],
        feats: vec![],
        spells: SpellData::default(),
        inventory: vec![],
        currency: Currency {
            copper: 50,
            silver: 30,
            electrum: 0,
            gold: 15,
            platinum: 0,
        },
        equipped: EquippedItems::default(),
        personality: Personality {
            traits: Some("I'm friendly and welcoming to travelers and adventurers.".to_string()),
            ideals: Some("Community. We have to take care of each other.".to_string()),
            bonds: Some("I want Phandalin to prosper and be safe for my family.".to_string()),
            flaws: Some("I'm afraid of the Redbrands but feel powerless against them.".to_string()),
        },
        player_name: None,
        appearance: Appearance::default(),
        backstory: None,
        background_feature: None,
        roleplay_notes: RoleplayNotes::default(),
        npc_role: Some("Innkeeper".to_string()),
        npc_location: Some("Stonehill Inn, Phandalin".to_string()),
        npc_faction: None,
        npc_notes: Some("Toblen is a friendly innkeeper who came to Phandalin to prospect but found success running the Stonehill Inn instead. He's a good source of local rumors and information about the Redbrands troubling the town.".to_string()),
        legendary_actions: Vec::new(),
        legendary_action_count: None,
    }
}

/// Create Iarno Albrek (Glasstaff) - Level 5 Human Wizard, Redbrand leader
fn create_iarno(created_at: &str) -> CharacterData {
    let mut spell_slots = HashMap::new();
    spell_slots.insert(1, SpellSlots::new(4));
    spell_slots.insert(2, SpellSlots::new(3));
    spell_slots.insert(3, SpellSlots::new(2));

    CharacterData {
        character_name: "Iarno Albrek".to_string(),
        player_id: None, // NPC
        level: 5,
        experience_points: 0,
        version: 1,
        snapshot_reason: Some("Dev seed NPC".to_string()),
        created_at: created_at.to_string(),
        race: "Human".to_string(),
        subrace: None,
        classes: vec![ClassLevel {
            class_name: "Wizard".to_string(),
            level: 5,
            subclass: Some("School of Enchantment".to_string()),
            hit_dice_type: "d6".to_string(),
            hit_dice_remaining: 5,
        }],
        background: "Sage".to_string(),
        alignment: Some("Lawful Evil".to_string()),
        abilities: AbilityScores {
            strength: 9,
            dexterity: 14,
            constitution: 11,
            intelligence: 17,
            wisdom: 12,
            charisma: 11,
        },
        max_hp: 22,
        current_hp: 22,
        speed: 30,
        proficiencies: Proficiencies {
            skills: vec![
                "Arcana".to_string(),
                "History".to_string(),
                "Deception".to_string(),
                "Insight".to_string(),
            ],
            saves: vec!["Intelligence".to_string(), "Wisdom".to_string()],
            armor: vec![],
            weapons: vec![
                "Daggers".to_string(),
                "Darts".to_string(),
                "Slings".to_string(),
                "Quarterstaffs".to_string(),
                "Light Crossbows".to_string(),
            ],
            tools: vec![],
            languages: vec![
                "Common".to_string(),
                "Elvish".to_string(),
                "Draconic".to_string(),
            ],
        },
        class_features: vec![
            FeatureReference::new("Spellcasting", "Wizard", "PHB", 1),
            FeatureReference::new("Arcane Recovery", "Wizard", "PHB", 1),
            FeatureReference::with_subclass(
                "Hypnotic Gaze",
                "Wizard",
                "School of Enchantment",
                "PHB",
                2,
            ),
        ],
        feats: vec![],
        spells: SpellData {
            cantrips: vec![
                SpellReference::new("Fire Bolt", "PHB"),
                SpellReference::new("Light", "PHB"),
                SpellReference::new("Mage Hand", "PHB"),
                SpellReference::new("Prestidigitation", "PHB"),
            ],
            known_spells: vec![],
            prepared_spells: vec![
                SpellReference::new("Charm Person", "PHB"),
                SpellReference::new("Hold Person", "PHB"),
                SpellReference::new("Magic Missile", "PHB"),
                SpellReference::new("Mage Armor", "PHB"),
                SpellReference::new("Misty Step", "PHB"),
                SpellReference::new("Suggestion", "PHB"),
            ],
            spell_slots,
        },
        inventory: vec![
            InventoryItem {
                name: "Staff of Defense".to_string(),
                source: Some("DMG".to_string()),
                quantity: 1,
                weight: 4.0,
                value: 0.0,
                notes: Some(
                    "Glass staff that can cast Shield (1 charge) and Mage Armor (2 charges)"
                        .to_string(),
                ),
            },
            InventoryItem {
                name: "Spellbook".to_string(),
                source: Some("PHB".to_string()),
                quantity: 1,
                weight: 3.0,
                value: 50.0,
                notes: Some("Contains all prepared spells plus Detect Magic, Identify, Sleep, Scorching Ray".to_string()),
            },
        ],
        currency: Currency {
            copper: 0,
            silver: 0,
            electrum: 0,
            gold: 180,
            platinum: 0,
        },
        equipped: EquippedItems {
            armor: None,
            shield: None,
            main_hand: Some("Staff of Defense".to_string()),
            off_hand: None,
        },
        personality: Personality {
            traits: Some("I speak in a calm, measured tone that belies my ambition and cruelty.".to_string()),
            ideals: Some("Power. Knowledge is the path to power and domination.".to_string()),
            bonds: Some("I am loyal to the Black Spider who promises me arcane secrets.".to_string()),
            flaws: Some("I underestimate my enemies and overestimate my own cunning.".to_string()),
        },
        player_name: None,
        appearance: Appearance::default(),
        backstory: None,
        background_feature: None,
        roleplay_notes: RoleplayNotes::default(),
        npc_role: Some("Antagonist".to_string()),
        npc_location: Some("Tresendar Manor, Phandalin".to_string()),
        npc_faction: Some("Redbrands".to_string()),
        npc_notes: Some("Iarno Albrek, also known as 'Glasstaff', is a former member of the Lords' Alliance who was sent to establish order in Phandalin. Instead, he secretly became the leader of the Redbrands, a gang of ruffians, working for the mysterious Black Spider. He carries a distinctive glass staff and is a capable enchanter.".to_string()),
        legendary_actions: vec![
            LegendaryAction {
                name: "Cantrip".to_string(),
                cost: 1,
                description: "Glasstaff casts a cantrip.".to_string(),
            },
            LegendaryAction {
                name: "Staff Defense".to_string(),
                cost: 1,
                description: "Glasstaff expends one charge from his Staff of Defense to cast Shield as a reaction.".to_string(),
            },
            LegendaryAction {
                name: "Hypnotic Gaze".to_string(),
                cost: 2,
                description: "Glasstaff uses his Hypnotic Gaze feature on a creature within 5 feet. The target must succeed on a DC 14 Wisdom saving throw or be charmed until the end of Glasstaff's next turn.".to_string(),
            },
            LegendaryAction {
                name: "Cast a Spell".to_string(),
                cost: 3,
                description: "Glasstaff casts a spell from his list of prepared spells, using a spell slot as normal.".to_string(),
            },
        ],
        legendary_action_count: Some(3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::establish_connection;
    use crate::seed::template_seeder::seed_templates;

    #[test]
    fn test_seed_dev_data_creates_expected_data() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();
        seed_templates(&mut conn).unwrap();

        // Create temp directories for test campaign files and app data
        let temp_dir = tempfile::tempdir().unwrap();
        let campaigns_dir = temp_dir.path().join("campaigns");
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&campaigns_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        // First seed should create data
        let result = seed_dev_data(
            &mut conn,
            campaigns_dir.to_str().unwrap(),
            data_dir.to_str().unwrap(),
        )
        .unwrap();
        assert!(result, "First seed should return true");

        // Verify campaign created
        let mut repo = CampaignRepository::new(&mut conn);
        let campaigns = repo.list().unwrap();
        assert_eq!(campaigns.len(), 1);
        assert_eq!(campaigns[0].name, TEST_CAMPAIGN_NAME);

        // Verify players created
        use crate::dal::player::PlayerRepository;
        let mut player_repo = PlayerRepository::new(&mut conn);
        let players = player_repo.list().unwrap();
        assert_eq!(players.len(), 4);
        let player_names: Vec<&str> = players.iter().map(|p| p.name.as_str()).collect();
        assert!(player_names.contains(&"Alice"));
        assert!(player_names.contains(&"Bob"));
        assert!(player_names.contains(&"Charlie"));
        assert!(player_names.contains(&"Diana"));

        // Verify characters created (4 PCs + 4 NPCs = 8 total)
        use crate::dal::character::CharacterRepository;
        let mut char_repo = CharacterRepository::new(&mut conn);
        let characters = char_repo.list_all().unwrap();
        assert_eq!(characters.len(), 8);

        // Verify PCs
        let char_names: Vec<&str> = characters
            .iter()
            .map(|c| c.character_name.as_str())
            .collect();
        assert!(char_names.contains(&"Thorin Ironforge"));
        assert!(char_names.contains(&"Elara Moonwhisper"));
        assert!(char_names.contains(&"Finn Lightfoot"));
        assert!(char_names.contains(&"Sister Helena"));

        // Verify NPCs
        assert!(char_names.contains(&"Sildar Hallwinter"));
        assert!(char_names.contains(&"Gundren Rockseeker"));
        assert!(char_names.contains(&"Toblen Stonehill"));
        assert!(char_names.contains(&"Iarno Albrek"));

        // Verify character levels
        let thorin = characters
            .iter()
            .find(|c| c.character_name == "Thorin Ironforge")
            .unwrap();
        assert_eq!(thorin.current_level, 5);
        assert!(!thorin.is_npc()); // PC, not NPC

        let finn = characters
            .iter()
            .find(|c| c.character_name == "Finn Lightfoot")
            .unwrap();
        assert_eq!(finn.current_level, 1);

        let helena = characters
            .iter()
            .find(|c| c.character_name == "Sister Helena")
            .unwrap();
        assert_eq!(helena.current_level, 10);

        // Verify NPCs are marked as such
        let sildar = characters
            .iter()
            .find(|c| c.character_name == "Sildar Hallwinter")
            .unwrap();
        assert!(sildar.is_npc());
        assert_eq!(sildar.player_id, None); // NPCs have no player

        let gundren = characters
            .iter()
            .find(|c| c.character_name == "Gundren Rockseeker")
            .unwrap();
        assert!(gundren.is_npc());
    }

    #[test]
    fn test_seed_dev_data_is_stateful() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();
        seed_templates(&mut conn).unwrap();

        // Create temp directories for test campaign files and app data
        let temp_dir = tempfile::tempdir().unwrap();
        let campaigns_dir = temp_dir.path().join("campaigns");
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&campaigns_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        // First seed should create data
        let first_result = seed_dev_data(
            &mut conn,
            campaigns_dir.to_str().unwrap(),
            data_dir.to_str().unwrap(),
        )
        .unwrap();
        assert!(first_result, "First seed should return true");

        // Second seed should skip (stateful - data already exists)
        let second_result = seed_dev_data(
            &mut conn,
            campaigns_dir.to_str().unwrap(),
            data_dir.to_str().unwrap(),
        )
        .unwrap();
        assert!(!second_result, "Second seed should return false (skipped)");

        // Verify still only one campaign (preserved, not re-created)
        let mut repo = CampaignRepository::new(&mut conn);
        let campaigns = repo.list().unwrap();
        assert_eq!(campaigns.len(), 1, "Should still have only one campaign");
    }

    #[test]
    fn test_is_already_seeded() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        // Initially not seeded
        assert!(!is_already_seeded(&mut conn).unwrap());

        // Manually create a campaign with the test name
        use crate::dal::campaign::campaigns::CampaignRepository;
        use crate::models::campaign::campaigns::NewCampaign;
        let mut repo = CampaignRepository::new(&mut conn);
        repo.create(NewCampaign {
            name: TEST_CAMPAIGN_NAME.to_string(),
            status: "concept".to_string(),
            directory_path: "/tmp/test".to_string(),
        })
        .unwrap();

        // Now should be seeded
        assert!(is_already_seeded(&mut conn).unwrap());
    }
}
