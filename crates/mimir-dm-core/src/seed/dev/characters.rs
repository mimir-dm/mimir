//! Character seeding for dev data.

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::character::{
    AbilityScores, Character, CharacterData, ClassLevel, Proficiencies,
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

    // PCs
    let pcs = [
        ("Alice", "Thorin Ironforge", "Dwarf", "Fighter", 5),
        ("Bob", "Elara Moonwhisper", "Elf", "Wizard", 5),
        ("Charlie", "Finn Lightfoot", "Halfling", "Rogue", 1),
        ("Diana", "Sister Helena", "Human", "Cleric", 10),
    ];

    for (player_name, char_name, race, class, level) in pcs {
        if let Some(&player_id) = player_map.get(player_name) {
            let data = make_character(char_name, race, class, level, Some(player_id), &now);
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

    // NPCs
    let npcs = [
        ("Sildar Hallwinter", "Human", "Fighter", 5),
        ("Gundren Rockseeker", "Dwarf", "Fighter", 3),
        ("Toblen Stonehill", "Human", "Commoner", 1),
        ("Iarno Albrek", "Human", "Wizard", 4),
    ];

    for (name, race, class, level) in npcs {
        let data = make_character(name, race, class, level, None, &now);
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
        inventory: vec![],
        currency: Default::default(),
        equipped: Default::default(),
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

fn make_character(
    name: &str,
    race: &str,
    class: &str,
    level: i32,
    player_id: Option<i32>,
    created_at: &str,
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
        inventory: vec![],
        currency: Default::default(),
        equipped: Default::default(),
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
