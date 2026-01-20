//! Character seeding for dev data.

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::character::{
    AbilityScores, Character, CharacterData, ClassLevel, Currency, EquippedItems, InventoryItem,
    Proficiencies,
};
use crate::models::player::Player;
use crate::services::{CharacterService, PlayerService};
use chrono::Utc;
use std::collections::HashMap;

/// Seed test players.
pub fn seed_players(conn: &mut DbConnection) -> Result<Vec<Player>> {
    let data = [
        ("Alice", "alice@test.com", "Experienced player"),
        ("Bob", "bob@test.com", "Creative roleplayer"),
        ("Charlie", "charlie@test.com", "New to D&D"),
        ("Diana", "diana@test.com", "Forever DM trying player side"),
    ];

    let mut players = Vec::new();
    for (name, email, notes) in data {
        let mut service = PlayerService::new(conn);
        let player = service.create_player(name, Some(email.into()), Some(notes.into()))?;
        players.push(player);
    }
    Ok(players)
}

/// Seed test characters (PCs and NPCs).
pub fn seed_characters(
    conn: &mut DbConnection,
    campaign_id: i32,
    campaign_dir: &str,
    players: &[Player],
) -> Result<Vec<Character>> {
    let now = Utc::now().to_rfc3339();
    let player_map: HashMap<&str, i32> = players.iter().map(|p| (p.name.as_str(), p.id)).collect();

    let mut characters = Vec::new();

    // PCs with class-appropriate equipment
    let pcs: Vec<(&str, &str, &str, &str, i32, CharacterEquipment)> = vec![
        (
            "Alice",
            "Thorin Ironforge",
            "Dwarf",
            "Fighter",
            5,
            CharacterEquipment::fighter(),
        ),
        (
            "Bob",
            "Elara Moonwhisper",
            "Elf",
            "Wizard",
            5,
            CharacterEquipment::wizard(),
        ),
        (
            "Charlie",
            "Finn Lightfoot",
            "Halfling",
            "Rogue",
            1,
            CharacterEquipment::rogue(),
        ),
        (
            "Diana",
            "Sister Helena",
            "Human",
            "Cleric",
            10,
            CharacterEquipment::cleric(),
        ),
    ];

    for (player_name, char_name, race, class, level, equipment) in pcs {
        if let Some(&player_id) = player_map.get(player_name) {
            let data = make_character(char_name, race, class, level, Some(player_id), &now, equipment);
            let mut service = CharacterService::new(conn);
            let character = service.create_character(
                Some(campaign_id),
                Some(player_id),
                false,
                campaign_dir,
                data,
            )?;
            characters.push(character);
        }
    }

    // NPCs with appropriate equipment
    let npcs: Vec<(&str, &str, &str, i32, CharacterEquipment)> = vec![
        (
            "Sildar Hallwinter",
            "Human",
            "Fighter",
            5,
            CharacterEquipment::npc_fighter(),
        ),
        (
            "Gundren Rockseeker",
            "Dwarf",
            "Fighter",
            3,
            CharacterEquipment::npc_merchant(),
        ),
        (
            "Toblen Stonehill",
            "Human",
            "Commoner",
            1,
            CharacterEquipment::npc_innkeeper(),
        ),
        (
            "Iarno Albrek",
            "Human",
            "Wizard",
            4,
            CharacterEquipment::npc_wizard(),
        ),
    ];

    for (name, race, class, level, equipment) in npcs {
        let data = make_character(name, race, class, level, None, &now, equipment);
        let mut service = CharacterService::new(conn);
        let character = service.create_character(Some(campaign_id), None, true, campaign_dir, data)?;
        characters.push(character);
    }

    Ok(characters)
}

/// Seed a concept character (no campaign association).
/// Simulates a player working on a character concept before joining a campaign.
pub fn seed_concept_character(conn: &mut DbConnection, players: &[Player]) -> Result<Character> {
    let now = Utc::now().to_rfc3339();

    // Find Charlie - the new player experimenting with concepts
    let charlie = players.iter().find(|p| p.name == "Charlie");
    let equipment = CharacterEquipment::concept_sorcerer();

    let data = CharacterData {
        character_name: "Zephyr Stormborn".into(),
        player_name: None,
        player_id: charlie.map(|p| p.id),
        level: 3,
        experience_points: 900,
        version: 1,
        snapshot_reason: Some("Character concept".into()),
        created_at: now,
        race: "Air Genasi".into(),
        subrace: None,
        classes: vec![ClassLevel {
            class_name: "Sorcerer".into(),
            level: 3,
            subclass: Some("Storm Sorcery".into()),
            hit_dice_type: "d6".into(),
            hit_dice_remaining: 3,
        }],
        background: "Sailor".into(),
        alignment: Some("Chaotic Good".into()),
        abilities: AbilityScores {
            strength: 10,
            dexterity: 14,
            constitution: 13,
            intelligence: 12,
            wisdom: 10,
            charisma: 16,
        },
        max_hp: 20,
        current_hp: 20,
        speed: 30,
        proficiencies: Proficiencies::default(),
        class_features: vec![],
        feats: vec![],
        spells: Default::default(),
        inventory: equipment.inventory,
        currency: equipment.currency,
        equipped: equipment.equipped,
        personality: Default::default(),
        appearance: Default::default(),
        backstory: Some("Born during a violent storm at sea, Zephyr has always felt the call of the wind...".into()),
        background_feature: None,
        roleplay_notes: Default::default(),
        npc_role: None,
        npc_location: None,
        npc_faction: None,
        npc_notes: None,
        legendary_action_count: None,
        legendary_actions: vec![],
    };

    let mut service = CharacterService::new(conn);
    // campaign_id = None, player_id from Charlie, is_npc = false, empty campaign_dir
    service.create_character(None, charlie.map(|p| p.id), false, "", data)
}

/// Equipment loadout for a character
struct CharacterEquipment {
    inventory: Vec<InventoryItem>,
    currency: Currency,
    equipped: EquippedItems,
}

impl CharacterEquipment {
    /// Fighter PC: plate armor, shield, longsword, adventuring gear
    fn fighter() -> Self {
        Self {
            inventory: vec![
                item("Plate Armor", "PHB", 1, 65.0, 1500.0),
                item("Shield", "PHB", 1, 6.0, 10.0),
                item("Longsword", "PHB", 1, 3.0, 15.0),
                item("Handaxe", "PHB", 2, 4.0, 10.0),
                item("Backpack", "PHB", 1, 5.0, 2.0),
                item("Bedroll", "PHB", 1, 7.0, 1.0),
                item("Rations (1 day)", "PHB", 10, 20.0, 5.0),
                item("Waterskin", "PHB", 1, 5.0, 0.2),
                item("Rope, Hempen (50 feet)", "PHB", 1, 10.0, 1.0),
                item("Potion of Healing", "DMG", 2, 0.5, 50.0),
            ],
            currency: Currency { gold: 85, silver: 30, copper: 50, ..Default::default() },
            equipped: EquippedItems {
                armor: Some("Plate Armor".into()),
                shield: Some("Shield".into()),
                main_hand: Some("Longsword".into()),
                off_hand: None,
            },
        }
    }

    /// Wizard PC: robes, staff, spellbook, components
    fn wizard() -> Self {
        Self {
            inventory: vec![
                item("Robes", "PHB", 1, 4.0, 1.0),
                item("Quarterstaff", "PHB", 1, 4.0, 0.2),
                item("Spellbook", "PHB", 1, 3.0, 50.0),
                item("Component Pouch", "PHB", 1, 2.0, 25.0),
                item("Scholar's Pack", "PHB", 1, 10.0, 40.0),
                item("Dagger", "PHB", 2, 2.0, 4.0),
                item("Ink (1 ounce bottle)", "PHB", 2, 0.0, 20.0),
                item("Parchment (sheet)", "PHB", 10, 0.0, 1.0),
                item("Scroll of Identify", "DMG", 1, 0.0, 25.0),
                item("Potion of Healing", "DMG", 1, 0.5, 50.0),
            ],
            currency: Currency { gold: 120, silver: 50, copper: 0, ..Default::default() },
            equipped: EquippedItems {
                armor: Some("Robes".into()),
                shield: None,
                main_hand: Some("Quarterstaff".into()),
                off_hand: None,
            },
        }
    }

    /// Rogue PC: leather armor, daggers, thieves' tools
    fn rogue() -> Self {
        Self {
            inventory: vec![
                item("Leather Armor", "PHB", 1, 10.0, 10.0),
                item("Shortsword", "PHB", 1, 2.0, 10.0),
                item("Dagger", "PHB", 3, 3.0, 6.0),
                item("Shortbow", "PHB", 1, 2.0, 25.0),
                item("Arrows", "PHB", 20, 1.0, 1.0),
                item("Thieves' Tools", "PHB", 1, 1.0, 25.0),
                item("Burglar's Pack", "PHB", 1, 10.0, 16.0),
                item("Hooded Lantern", "PHB", 1, 2.0, 5.0),
                item("Oil (flask)", "PHB", 3, 3.0, 0.3),
            ],
            currency: Currency { gold: 25, silver: 80, copper: 45, ..Default::default() },
            equipped: EquippedItems {
                armor: Some("Leather Armor".into()),
                shield: None,
                main_hand: Some("Shortsword".into()),
                off_hand: Some("Dagger".into()),
            },
        }
    }

    /// Cleric PC: chain mail, shield, mace, holy symbol
    fn cleric() -> Self {
        Self {
            inventory: vec![
                item("Chain Mail", "PHB", 1, 55.0, 75.0),
                item("Shield", "PHB", 1, 6.0, 10.0),
                item("Mace", "PHB", 1, 4.0, 5.0),
                item("Holy Symbol (Amulet)", "PHB", 1, 1.0, 5.0),
                item("Priest's Pack", "PHB", 1, 10.0, 19.0),
                item("Healer's Kit", "PHB", 1, 3.0, 5.0),
                item("Potion of Healing", "DMG", 4, 2.0, 200.0),
                item("Scroll of Lesser Restoration", "DMG", 1, 0.0, 60.0),
                item("Holy Water (flask)", "PHB", 2, 2.0, 50.0),
            ],
            currency: Currency { gold: 200, silver: 25, copper: 0, platinum: 5, ..Default::default() },
            equipped: EquippedItems {
                armor: Some("Chain Mail".into()),
                shield: Some("Shield".into()),
                main_hand: Some("Mace".into()),
                off_hand: None,
            },
        }
    }

    /// NPC Fighter: soldier equipment
    fn npc_fighter() -> Self {
        Self {
            inventory: vec![
                item("Chain Mail", "PHB", 1, 55.0, 75.0),
                item("Longsword", "PHB", 1, 3.0, 15.0),
                item("Shield", "PHB", 1, 6.0, 10.0),
                item("Dagger", "PHB", 1, 1.0, 2.0),
                item("Rations (1 day)", "PHB", 5, 10.0, 2.5),
            ],
            currency: Currency { gold: 50, silver: 20, copper: 15, ..Default::default() },
            equipped: EquippedItems {
                armor: Some("Chain Mail".into()),
                shield: Some("Shield".into()),
                main_hand: Some("Longsword".into()),
                off_hand: None,
            },
        }
    }

    /// NPC Merchant: traveling gear, trade goods
    fn npc_merchant() -> Self {
        Self {
            inventory: vec![
                item("Traveler's Clothes", "PHB", 1, 4.0, 2.0),
                item("Miner's Pick", "PHB", 1, 10.0, 2.0),
                item("Pouch", "PHB", 1, 1.0, 0.5),
                item("Map Case", "PHB", 1, 1.0, 1.0),
                item("Gemstone (various)", "DMG", 5, 0.0, 250.0),
            ],
            currency: Currency { gold: 150, silver: 50, copper: 0, platinum: 10, ..Default::default() },
            equipped: EquippedItems::default(),
        }
    }

    /// NPC Innkeeper: simple equipment
    fn npc_innkeeper() -> Self {
        Self {
            inventory: vec![
                item("Common Clothes", "PHB", 1, 3.0, 0.5),
                item("Apron", "PHB", 1, 1.0, 0.1),
                item("Dagger", "PHB", 1, 1.0, 2.0),
                item("Key Ring", "PHB", 1, 0.1, 0.0),
            ],
            currency: Currency { gold: 30, silver: 100, copper: 250, ..Default::default() },
            equipped: EquippedItems::default(),
        }
    }

    /// NPC Wizard: spellcasting equipment
    fn npc_wizard() -> Self {
        Self {
            inventory: vec![
                item("Robes", "PHB", 1, 4.0, 1.0),
                item("Staff of Defense", "DMG", 1, 4.0, 0.0), // Magic item
                item("Spellbook", "PHB", 1, 3.0, 50.0),
                item("Component Pouch", "PHB", 1, 2.0, 25.0),
                item("Dagger", "PHB", 1, 1.0, 2.0),
                item("Potion of Invisibility", "DMG", 1, 0.5, 0.0),
                item("Scroll of Fireball", "DMG", 1, 0.0, 150.0),
            ],
            currency: Currency { gold: 75, silver: 30, copper: 0, ..Default::default() },
            equipped: EquippedItems {
                armor: Some("Robes".into()),
                shield: None,
                main_hand: Some("Staff of Defense".into()),
                off_hand: None,
            },
        }
    }

    /// Concept character: sorcerer with minimal gear
    fn concept_sorcerer() -> Self {
        Self {
            inventory: vec![
                item("Traveler's Clothes", "PHB", 1, 4.0, 2.0),
                item("Dagger", "PHB", 2, 2.0, 4.0),
                item("Arcane Focus (Crystal)", "PHB", 1, 1.0, 10.0),
                item("Backpack", "PHB", 1, 5.0, 2.0),
                item("Bedroll", "PHB", 1, 7.0, 1.0),
                item("Mess Kit", "PHB", 1, 1.0, 0.2),
                item("Tinderbox", "PHB", 1, 1.0, 0.5),
                item("Rations (1 day)", "PHB", 5, 10.0, 2.5),
                item("Waterskin", "PHB", 1, 5.0, 0.2),
            ],
            currency: Currency { gold: 15, silver: 0, copper: 0, ..Default::default() },
            equipped: EquippedItems {
                armor: None,
                shield: None,
                main_hand: Some("Dagger".into()),
                off_hand: None,
            },
        }
    }
}

/// Helper to create inventory items
fn item(name: &str, source: &str, qty: i32, weight: f64, value: f64) -> InventoryItem {
    InventoryItem {
        name: name.into(),
        source: Some(source.into()),
        quantity: qty,
        weight,
        value,
        notes: None,
    }
}

fn make_character(
    name: &str,
    race: &str,
    class: &str,
    level: i32,
    player_id: Option<i32>,
    created_at: &str,
    equipment: CharacterEquipment,
) -> CharacterData {
    CharacterData {
        character_name: name.into(),
        player_name: None,
        player_id,
        level,
        experience_points: 0,
        version: 1,
        snapshot_reason: Some("Dev seed".into()),
        created_at: created_at.into(),
        race: race.into(),
        subrace: None,
        classes: vec![ClassLevel {
            class_name: class.into(),
            level,
            subclass: None,
            hit_dice_type: "d8".into(),
            hit_dice_remaining: level,
        }],
        background: "Folk Hero".into(),
        alignment: Some("Neutral Good".into()),
        abilities: AbilityScores {
            strength: 14,
            dexterity: 14,
            constitution: 14,
            intelligence: 12,
            wisdom: 12,
            charisma: 10,
        },
        max_hp: 10 + (level - 1) * 6,
        current_hp: 10 + (level - 1) * 6,
        speed: 30,
        proficiencies: Proficiencies::default(),
        class_features: vec![],
        feats: vec![],
        spells: Default::default(),
        inventory: equipment.inventory,
        currency: equipment.currency,
        equipped: equipment.equipped,
        personality: Default::default(),
        appearance: Default::default(),
        backstory: None,
        background_feature: None,
        roleplay_notes: Default::default(),
        npc_role: None,
        npc_location: None,
        npc_faction: None,
        npc_notes: None,
        legendary_action_count: None,
        legendary_actions: vec![],
    }
}
