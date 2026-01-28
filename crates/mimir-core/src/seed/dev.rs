//! Development database seeder.
//!
//! Seeds the database with "The Lost Mine of Phandelver" test data.

use diesel::SqliteConnection;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{
    Character, Module, NewCampaignAsset, NewCharacter, NewCharacterClass, NewCharacterInventory,
    NewLightSource, NewMap, NewMapPoi, NewMapTrap, NewModuleMonster, NewModuleNpc,
    NewTokenPlacement,
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

    // Thorin Ironforge - Dwarf Fighter
    let thorin_id = Uuid::new_v4().to_string();
    let thorin = NewCharacter::new_pc(&thorin_id, campaign_id, "Thorin Ironforge", "Alice")
        .with_race("Mountain Dwarf", "PHB")
        .with_background("Soldier", "PHB")
        .with_ability_scores(16, 12, 16, 10, 12, 8)
        .with_currency(0, 30, 0, 85, 0)
        .with_roleplay(
            Some("I face problems head-on. A direct approach is best."),
            Some("My honor is my life. I will protect the innocent."),
            Some("I fight for those who cannot fight for themselves."),
            Some("I have little respect for those who show weakness in battle."),
        );
    dal::insert_character(conn, &thorin)?;

    // Add class
    let thorin_class_id = Uuid::new_v4().to_string();
    let thorin_class = NewCharacterClass::starting(&thorin_class_id, &thorin_id, "Fighter", "PHB")
        .with_level(5)
        .with_subclass("Champion", "PHB");
    dal::insert_character_class(conn, &thorin_class)?;

    // Add inventory
    for (name, qty, is_equipped) in [
        ("Plate Armor", 1, true),
        ("Shield", 1, true),
        ("Longsword", 1, true),
        ("Handaxe", 2, false),
        ("Potion of Healing", 2, false),
        ("Backpack", 1, false),
        ("Bedroll", 1, false),
    ] {
        let item_id = Uuid::new_v4().to_string();
        let mut item = NewCharacterInventory::new(&item_id, &thorin_id, name, "PHB")
            .with_quantity(qty);
        if is_equipped {
            item = item.equipped();
        }
        dal::insert_character_inventory(conn, &item)?;
    }
    characters.push(dal::get_character(conn, &thorin_id)?);

    // Elara Moonwhisper - Elf Wizard
    let elara_id = Uuid::new_v4().to_string();
    let elara = NewCharacter::new_pc(&elara_id, campaign_id, "Elara Moonwhisper", "Bob")
        .with_race("High Elf", "PHB")
        .with_background("Sage", "PHB")
        .with_ability_scores(8, 14, 12, 17, 13, 10)
        .with_currency(0, 50, 0, 120, 0)
        .with_roleplay(
            Some("I use polysyllabic words to impress people."),
            Some("Knowledge is the path to power and self-improvement."),
            Some("I sold my soul for knowledge. I hope to do great deeds to win it back."),
            Some("Most people scream and run when they see a demon. I stop and take notes."),
        );
    dal::insert_character(conn, &elara)?;

    let elara_class_id = Uuid::new_v4().to_string();
    let elara_class = NewCharacterClass::starting(&elara_class_id, &elara_id, "Wizard", "PHB")
        .with_level(5)
        .with_subclass("School of Evocation", "PHB");
    dal::insert_character_class(conn, &elara_class)?;

    for (name, qty, is_equipped) in [
        ("Quarterstaff", 1, true),
        ("Spellbook", 1, false),
        ("Component Pouch", 1, false),
        ("Dagger", 2, false),
        ("Scroll of Identify", 1, false),
        ("Potion of Healing", 1, false),
    ] {
        let item_id = Uuid::new_v4().to_string();
        let mut item = NewCharacterInventory::new(&item_id, &elara_id, name, "PHB")
            .with_quantity(qty);
        if is_equipped {
            item = item.equipped();
        }
        dal::insert_character_inventory(conn, &item)?;
    }
    characters.push(dal::get_character(conn, &elara_id)?);

    // Finn Lightfoot - Halfling Rogue
    let finn_id = Uuid::new_v4().to_string();
    let finn = NewCharacter::new_pc(&finn_id, campaign_id, "Finn Lightfoot", "Charlie")
        .with_race("Lightfoot Halfling", "PHB")
        .with_background("Criminal", "PHB")
        .with_ability_scores(10, 18, 12, 13, 10, 14)
        .with_currency(45, 80, 0, 25, 0)
        .with_roleplay(
            Some("I always have a plan for what to do when things go wrong."),
            Some("Freedom. Chains are meant to be broken."),
            Some("Someone I loved died because of a mistake I made."),
            Some("I turn tail and run when things look bad."),
        );
    dal::insert_character(conn, &finn)?;

    let finn_class_id = Uuid::new_v4().to_string();
    let finn_class = NewCharacterClass::starting(&finn_class_id, &finn_id, "Rogue", "PHB")
        .with_level(3)
        .with_subclass("Thief", "PHB");
    dal::insert_character_class(conn, &finn_class)?;

    for (name, qty, is_equipped) in [
        ("Leather Armor", 1, true),
        ("Shortsword", 1, true),
        ("Dagger", 3, false),
        ("Shortbow", 1, false),
        ("Arrows", 20, false),
        ("Thieves' Tools", 1, false),
    ] {
        let item_id = Uuid::new_v4().to_string();
        let mut item = NewCharacterInventory::new(&item_id, &finn_id, name, "PHB")
            .with_quantity(qty);
        if is_equipped {
            item = item.equipped();
        }
        dal::insert_character_inventory(conn, &item)?;
    }
    characters.push(dal::get_character(conn, &finn_id)?);

    // Sister Helena - Human Cleric
    let helena_id = Uuid::new_v4().to_string();
    let helena = NewCharacter::new_pc(&helena_id, campaign_id, "Sister Helena", "Diana")
        .with_race("Human", "PHB")
        .with_background("Acolyte", "PHB")
        .with_ability_scores(14, 10, 14, 10, 16, 13)
        .with_currency(0, 25, 0, 200, 5)
        .with_roleplay(
            Some("I see omens in every event and action. The gods try to speak to us."),
            Some("Charity. I always try to help those in need."),
            Some("I will do anything to protect the temple where I served."),
            Some("I am suspicious of strangers and expect the worst of them."),
        );
    dal::insert_character(conn, &helena)?;

    let helena_class_id = Uuid::new_v4().to_string();
    let helena_class = NewCharacterClass::starting(&helena_class_id, &helena_id, "Cleric", "PHB")
        .with_level(7)
        .with_subclass("Life Domain", "PHB");
    dal::insert_character_class(conn, &helena_class)?;

    for (name, qty, is_equipped) in [
        ("Chain Mail", 1, true),
        ("Shield", 1, true),
        ("Mace", 1, true),
        ("Holy Symbol", 1, false),
        ("Healer's Kit", 1, false),
        ("Potion of Healing", 4, false),
        ("Holy Water", 2, false),
    ] {
        let item_id = Uuid::new_v4().to_string();
        let mut item = NewCharacterInventory::new(&item_id, &helena_id, name, "PHB")
            .with_quantity(qty);
        if is_equipped {
            item = item.equipped();
        }
        dal::insert_character_inventory(conn, &item)?;
    }
    characters.push(dal::get_character(conn, &helena_id)?);

    // =========================================================================
    // NPCs - Fully seeded with personality and inventory
    // =========================================================================

    // Sildar Hallwinter - Human Fighter (Ally)
    let sildar_id = Uuid::new_v4().to_string();
    let sildar = NewCharacter::new_npc(&sildar_id, campaign_id, "Sildar Hallwinter")
        .with_race("Human", "PHB")
        .with_background("Soldier", "PHB")
        .with_ability_scores(16, 11, 14, 10, 11, 10)
        .with_currency(0, 20, 0, 50, 0)
        .with_npc_info(Some("Ally"), Some("Phandalin"), Some("Lords' Alliance"))
        .with_roleplay(
            Some("I'm always polite and respectful, even to my enemies."),
            Some("Greater Good. Our lot is to protect the civilized lands."),
            Some("I would still lay down my life for the people I served with."),
            Some("I made a terrible mistake in battle that cost many lives."),
        );
    dal::insert_character(conn, &sildar)?;

    let sildar_class_id = Uuid::new_v4().to_string();
    let sildar_class = NewCharacterClass::starting(&sildar_class_id, &sildar_id, "Fighter", "PHB")
        .with_level(5);
    dal::insert_character_class(conn, &sildar_class)?;

    for (name, qty, is_equipped) in [
        ("Chain Mail", 1, true),
        ("Longsword", 1, true),
        ("Shield", 1, true),
        ("Dagger", 1, false),
        ("Potion of Healing", 1, false),
    ] {
        let item_id = Uuid::new_v4().to_string();
        let mut item = NewCharacterInventory::new(&item_id, &sildar_id, name, "PHB")
            .with_quantity(qty);
        if is_equipped {
            item = item.equipped();
        }
        dal::insert_character_inventory(conn, &item)?;
    }
    characters.push(dal::get_character(conn, &sildar_id)?);

    // Gundren Rockseeker - Dwarf Fighter (Quest Giver)
    let gundren_id = Uuid::new_v4().to_string();
    let gundren = NewCharacter::new_npc(&gundren_id, campaign_id, "Gundren Rockseeker")
        .with_race("Hill Dwarf", "PHB")
        .with_background("Guild Artisan", "PHB")
        .with_ability_scores(12, 10, 14, 13, 12, 10)
        .with_currency(0, 50, 0, 150, 10)
        .with_npc_info(Some("Quest Giver"), Some("Wave Echo Cave"), None)
        .with_roleplay(
            Some("I believe that anything worth doing is worth doing right."),
            Some("Aspiration. I work hard to be the best there is at my craft."),
            Some("I owe my guild a great debt for forging me into the person I am."),
            Some("I'm never satisfied with what I have - I always want more."),
        );
    dal::insert_character(conn, &gundren)?;

    let gundren_class_id = Uuid::new_v4().to_string();
    let gundren_class = NewCharacterClass::starting(&gundren_class_id, &gundren_id, "Fighter", "PHB")
        .with_level(3);
    dal::insert_character_class(conn, &gundren_class)?;

    for (name, qty, is_equipped) in [
        ("Traveler's Clothes", 1, true),
        ("Miner's Pick", 1, false),
        ("Map Case", 1, false),
        ("Pouch", 1, false),
    ] {
        let item_id = Uuid::new_v4().to_string();
        let mut item = NewCharacterInventory::new(&item_id, &gundren_id, name, "PHB")
            .with_quantity(qty);
        if is_equipped {
            item = item.equipped();
        }
        dal::insert_character_inventory(conn, &item)?;
    }
    characters.push(dal::get_character(conn, &gundren_id)?);

    // Toblen Stonehill - Human Innkeeper
    let toblen_id = Uuid::new_v4().to_string();
    let toblen = NewCharacter::new_npc(&toblen_id, campaign_id, "Toblen Stonehill")
        .with_race("Human", "PHB")
        .with_background("Folk Hero", "PHB")
        .with_ability_scores(10, 10, 12, 10, 14, 13)
        .with_currency(250, 100, 0, 30, 0)
        .with_npc_info(Some("Innkeeper"), Some("Stonehill Inn, Phandalin"), None)
        .with_roleplay(
            Some("I judge people by their actions, not their words."),
            Some("Sincerity. There's no good in pretending to be something I'm not."),
            Some("I protect those who cannot protect themselves."),
            Some("I have trouble trusting in my allies."),
        );
    dal::insert_character(conn, &toblen)?;

    let toblen_class_id = Uuid::new_v4().to_string();
    let toblen_class = NewCharacterClass::starting(&toblen_class_id, &toblen_id, "Commoner", "PHB")
        .with_level(1);
    dal::insert_character_class(conn, &toblen_class)?;

    for (name, qty, is_equipped) in [
        ("Common Clothes", 1, true),
        ("Apron", 1, true),
        ("Dagger", 1, false),
        ("Key Ring", 1, false),
    ] {
        let item_id = Uuid::new_v4().to_string();
        let mut item = NewCharacterInventory::new(&item_id, &toblen_id, name, "PHB")
            .with_quantity(qty);
        if is_equipped {
            item = item.equipped();
        }
        dal::insert_character_inventory(conn, &item)?;
    }
    characters.push(dal::get_character(conn, &toblen_id)?);

    // Iarno Albrek (Glasstaff) - Human Wizard (Antagonist)
    let iarno_id = Uuid::new_v4().to_string();
    let iarno = NewCharacter::new_npc(&iarno_id, campaign_id, "Iarno Albrek")
        .with_race("Human", "PHB")
        .with_background("Criminal", "PHB")
        .with_ability_scores(10, 12, 10, 16, 14, 14)
        .with_currency(0, 30, 0, 75, 0)
        .with_npc_info(Some("Antagonist"), Some("Tresendar Manor"), Some("Redbrands"))
        .with_roleplay(
            Some("I am incredibly slow to trust. Those who seem fair often have hidden motives."),
            Some("Power. I will do whatever it takes to become powerful."),
            Some("I will become the greatest wizard the world has ever seen."),
            Some("An innocent person is in prison for a crime I committed."),
        );
    dal::insert_character(conn, &iarno)?;

    let iarno_class_id = Uuid::new_v4().to_string();
    let iarno_class = NewCharacterClass::starting(&iarno_class_id, &iarno_id, "Wizard", "PHB")
        .with_level(4);
    dal::insert_character_class(conn, &iarno_class)?;

    for (name, qty, is_equipped) in [
        ("Robes", 1, true),
        ("Staff of Defense", 1, true),
        ("Spellbook", 1, false),
        ("Component Pouch", 1, false),
        ("Dagger", 1, false),
        ("Potion of Invisibility", 1, false),
        ("Scroll of Fireball", 1, false),
    ] {
        let item_id = Uuid::new_v4().to_string();
        let mut item = NewCharacterInventory::new(&item_id, &iarno_id, name, "PHB")
            .with_quantity(qty);
        if is_equipped {
            item = item.equipped();
        }
        dal::insert_character_inventory(conn, &item)?;
    }
    characters.push(dal::get_character(conn, &iarno_id)?);

    // Klarg - Bugbear Boss
    let klarg_id = Uuid::new_v4().to_string();
    let klarg = NewCharacter::new_npc(&klarg_id, campaign_id, "Klarg")
        .with_race("Bugbear", "MM")
        .with_background("Outlander", "PHB")
        .with_ability_scores(15, 14, 13, 8, 11, 9)
        .with_currency(0, 0, 0, 25, 0)
        .with_npc_info(Some("Boss"), Some("Cragmaw Hideout"), Some("Cragmaw Tribe"))
        .with_roleplay(
            Some("I watch over my friends as if they were newborn pups."),
            Some("Might. The strongest are meant to rule."),
            Some("My tribe is the most important thing in my life."),
            Some("Violence is my answer to almost any challenge."),
        );
    dal::insert_character(conn, &klarg)?;

    let klarg_class_id = Uuid::new_v4().to_string();
    let klarg_class = NewCharacterClass::starting(&klarg_class_id, &klarg_id, "Fighter", "PHB")
        .with_level(3);
    dal::insert_character_class(conn, &klarg_class)?;

    for (name, qty, is_equipped) in [
        ("Hide Armor", 1, true),
        ("Morningstar", 1, true),
        ("Javelin", 3, false),
        ("Belt Pouch", 1, false),
    ] {
        let item_id = Uuid::new_v4().to_string();
        let mut item = NewCharacterInventory::new(&item_id, &klarg_id, name, "PHB")
            .with_quantity(qty);
        if is_equipped {
            item = item.equipped();
        }
        dal::insert_character_inventory(conn, &item)?;
    }
    characters.push(dal::get_character(conn, &klarg_id)?);

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
