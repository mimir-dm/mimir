//! Character command integration tests.
//!
//! Tests for character CRUD operations including creating PCs and NPCs,
//! listing characters, campaign assignment, and character management.

use super::common::TestEnv;
use mimir_dm_core::models::character::data::{
    AbilityScores, Appearance, CharacterData, ClassLevel, Currency, EquippedItems, Personality,
    Proficiencies, RoleplayNotes, SpellData,
};
use mimir_dm_core::services::{CampaignService, CharacterService};

/// Helper to create a test campaign and return its ID and directory
async fn setup_campaign(env: &TestEnv) -> (i32, String) {
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = CampaignService::new(&mut conn);

    let campaign = service
        .create_campaign(
            "Test Campaign",
            None,
            env.temp_dir.path().to_str().unwrap(),
        )
        .expect("Failed to create campaign");

    (campaign.id, campaign.directory_path)
}

/// Helper to create minimal character data
fn create_minimal_character_data(name: &str) -> CharacterData {
    CharacterData {
        character_name: name.to_string(),
        player_id: None,
        level: 1,
        experience_points: 0,
        version: 1,
        snapshot_reason: Some("Test character".to_string()),
        created_at: chrono::Utc::now().to_rfc3339(),
        race: "Human".to_string(),
        subrace: None,
        classes: vec![ClassLevel {
            class_name: "Fighter".to_string(),
            level: 1,
            subclass: None,
            hit_dice_type: "d10".to_string(),
            hit_dice_remaining: 1,
        }],
        background: "Soldier".to_string(),
        alignment: Some("Neutral".to_string()),
        abilities: AbilityScores {
            strength: 16,
            dexterity: 14,
            constitution: 14,
            intelligence: 10,
            wisdom: 12,
            charisma: 8,
        },
        max_hp: 12,
        current_hp: 12,
        speed: 30,
        proficiencies: Proficiencies {
            skills: vec!["Athletics".to_string(), "Intimidation".to_string()],
            saves: vec!["Strength".to_string(), "Constitution".to_string()],
            armor: vec!["Light".to_string(), "Medium".to_string(), "Heavy".to_string(), "Shields".to_string()],
            weapons: vec!["Simple".to_string(), "Martial".to_string()],
            tools: vec![],
            languages: vec!["Common".to_string()],
        },
        class_features: vec![],
        feats: vec![],
        spells: SpellData::default(),
        inventory: vec![],
        currency: Currency::default(),
        equipped: EquippedItems::default(),
        personality: Personality::default(),
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

#[tokio::test]
async fn test_create_character() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CharacterService::new(&mut conn);
    let char_data = create_minimal_character_data("Test Fighter");

    let character = service
        .create_character(None, None, false, "", char_data)
        .expect("Failed to create character");

    assert_eq!(character.character_name, "Test Fighter");
    assert!(!character.is_npc);
}

#[tokio::test]
async fn test_create_npc() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, campaign_dir) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = CharacterService::new(&mut conn);

    let mut char_data = create_minimal_character_data("Shopkeeper Bob");
    char_data.npc_role = Some("Merchant".to_string());
    char_data.npc_location = Some("Market Square".to_string());

    let character = service
        .create_character(Some(campaign_id), None, true, &campaign_dir, char_data)
        .expect("Failed to create NPC");

    assert_eq!(character.character_name, "Shopkeeper Bob");
    assert!(character.is_npc);
    assert_eq!(character.campaign_id, Some(campaign_id));
}

#[tokio::test]
async fn test_get_character() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CharacterService::new(&mut conn);
    let char_data = create_minimal_character_data("Find Me");

    let created = service
        .create_character(None, None, false, "", char_data)
        .expect("Failed to create character");

    let (character, data) = service
        .get_character(created.id)
        .expect("Failed to get character");

    assert_eq!(character.id, created.id);
    assert_eq!(data.character_name, "Find Me");
    assert_eq!(data.race, "Human");
    assert_eq!(data.classes[0].class_name, "Fighter");
}

#[tokio::test]
async fn test_list_all_characters() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CharacterService::new(&mut conn);

    // Initially empty
    let chars = service.list_all_characters().expect("Failed to list characters");
    assert!(chars.is_empty());

    // Create some characters
    for i in 1..=3 {
        let char_data = create_minimal_character_data(&format!("Character {}", i));
        service
            .create_character(None, None, false, "", char_data)
            .expect("Failed to create character");
    }

    let chars = service.list_all_characters().expect("Failed to list characters");
    assert_eq!(chars.len(), 3);
}

#[tokio::test]
async fn test_list_characters_for_campaign() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, campaign_dir) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = CharacterService::new(&mut conn);

    // Create character in campaign
    let in_campaign = create_minimal_character_data("In Campaign");
    service
        .create_character(Some(campaign_id), None, false, &campaign_dir, in_campaign)
        .expect("Failed to create character");

    // Create character not in campaign
    let not_in_campaign = create_minimal_character_data("Not In Campaign");
    service
        .create_character(None, None, false, "", not_in_campaign)
        .expect("Failed to create character");

    let campaign_chars = service
        .list_characters_for_campaign(campaign_id)
        .expect("Failed to list campaign characters");

    assert_eq!(campaign_chars.len(), 1);
    assert_eq!(campaign_chars[0].character_name, "In Campaign");
}

#[tokio::test]
async fn test_list_npcs_for_campaign() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, campaign_dir) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = CharacterService::new(&mut conn);

    // Create NPC
    let npc_data = create_minimal_character_data("Guard Captain");
    service
        .create_character(Some(campaign_id), None, true, &campaign_dir, npc_data)
        .expect("Failed to create NPC");

    // Create PC
    let pc_data = create_minimal_character_data("Player Hero");
    service
        .create_character(Some(campaign_id), None, false, &campaign_dir, pc_data)
        .expect("Failed to create PC");

    let npcs = service
        .list_npcs_for_campaign(campaign_id)
        .expect("Failed to list NPCs");

    assert_eq!(npcs.len(), 1);
    assert_eq!(npcs[0].character_name, "Guard Captain");
    assert!(npcs[0].is_npc);
}

#[tokio::test]
async fn test_list_pcs_for_campaign() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, campaign_dir) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = CharacterService::new(&mut conn);

    // Create NPC
    let npc_data = create_minimal_character_data("Innkeeper");
    service
        .create_character(Some(campaign_id), None, true, &campaign_dir, npc_data)
        .expect("Failed to create NPC");

    // Create PCs
    let pc1_data = create_minimal_character_data("Hero One");
    service
        .create_character(Some(campaign_id), None, false, &campaign_dir, pc1_data)
        .expect("Failed to create PC");

    let pc2_data = create_minimal_character_data("Hero Two");
    service
        .create_character(Some(campaign_id), None, false, &campaign_dir, pc2_data)
        .expect("Failed to create PC");

    let pcs = service
        .list_pcs_for_campaign(campaign_id)
        .expect("Failed to list PCs");

    assert_eq!(pcs.len(), 2);
    assert!(pcs.iter().all(|c| !c.is_npc));
}

#[tokio::test]
async fn test_delete_character() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CharacterService::new(&mut conn);
    let char_data = create_minimal_character_data("Delete Me");

    let created = service
        .create_character(None, None, false, "", char_data)
        .expect("Failed to create character");

    service
        .delete_character(created.id)
        .expect("Failed to delete character");

    // Verify deleted
    let chars = service.list_all_characters().expect("Failed to list");
    assert!(chars.is_empty());
}

#[tokio::test]
async fn test_update_character() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CharacterService::new(&mut conn);
    let char_data = create_minimal_character_data("Update Me");

    let created = service
        .create_character(None, None, false, "", char_data)
        .expect("Failed to create character");

    // Update the character
    let (_, mut current_data) = service.get_character(created.id).expect("Failed to get");
    current_data.current_hp = 8; // Took some damage
    current_data.experience_points = 300;

    let new_version = service
        .update_character(created.id, current_data, Some("Took damage".to_string()))
        .expect("Failed to update character");

    assert_eq!(new_version.version_number, 2);

    // Verify update
    let (_, updated_data) = service.get_character(created.id).expect("Failed to get");
    assert_eq!(updated_data.current_hp, 8);
    assert_eq!(updated_data.experience_points, 300);
}

#[tokio::test]
async fn test_character_versions() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CharacterService::new(&mut conn);
    let char_data = create_minimal_character_data("Version Test");

    let created = service
        .create_character(None, None, false, "", char_data)
        .expect("Failed to create character");

    // Make several updates
    for i in 2..=4 {
        let (_, mut data) = service.get_character(created.id).expect("Failed to get");
        data.experience_points = i * 100;
        service
            .update_character(created.id, data, Some(format!("Update {}", i)))
            .expect("Failed to update");
    }

    let versions = service
        .get_character_versions(created.id)
        .expect("Failed to get versions");

    assert_eq!(versions.len(), 4);
}

#[tokio::test]
async fn test_get_specific_character_version() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CharacterService::new(&mut conn);
    let char_data = create_minimal_character_data("Version History");

    let created = service
        .create_character(None, None, false, "", char_data)
        .expect("Failed to create character");

    // Update to create version 2
    let (_, mut data) = service.get_character(created.id).expect("Failed to get");
    data.level = 2;
    data.max_hp = 22;
    data.current_hp = 22;
    service
        .update_character(created.id, data, Some("Level up".to_string()))
        .expect("Failed to update");

    // Get version 1 (original)
    let v1_data = service
        .get_character_version(created.id, 1)
        .expect("Failed to get version 1");

    assert_eq!(v1_data.level, 1);
    assert_eq!(v1_data.max_hp, 12);

    // Get version 2 (after level up)
    let v2_data = service
        .get_character_version(created.id, 2)
        .expect("Failed to get version 2");

    assert_eq!(v2_data.level, 2);
    assert_eq!(v2_data.max_hp, 22);
}

#[tokio::test]
async fn test_assign_character_to_campaign() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, campaign_dir) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = CharacterService::new(&mut conn);

    // Create unassigned character
    let char_data = create_minimal_character_data("Unassigned");
    let created = service
        .create_character(None, None, false, "", char_data)
        .expect("Failed to create character");

    assert!(created.campaign_id.is_none());

    // Assign to campaign
    let assigned = service
        .assign_to_campaign(created.id, campaign_id, &campaign_dir)
        .expect("Failed to assign to campaign");

    assert_eq!(assigned.campaign_id, Some(campaign_id));
}

#[tokio::test]
async fn test_add_item_to_inventory() {
    use mimir_dm_core::models::character::data::InventoryItem;

    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CharacterService::new(&mut conn);
    let char_data = create_minimal_character_data("Item Collector");

    let created = service
        .create_character(None, None, false, "", char_data)
        .expect("Failed to create character");

    // Add item via update_character (doesn't require catalog lookup)
    let (_, mut data) = service.get_character(created.id).expect("Failed to get");
    data.inventory.push(InventoryItem {
        name: "Longsword".to_string(),
        source: Some("PHB".to_string()),
        quantity: 1,
        weight: 3.0,
        value: 15.0,
        notes: None,
    });
    service
        .update_character(created.id, data, Some("Added longsword".to_string()))
        .expect("Failed to update character");

    // Verify item was added
    let (_, data) = service.get_character(created.id).expect("Failed to get");
    assert_eq!(data.inventory.len(), 1);
    assert_eq!(data.inventory[0].name, "Longsword");
    assert_eq!(data.inventory[0].quantity, 1);
}

#[tokio::test]
async fn test_remove_item_from_inventory() {
    use mimir_dm_core::models::character::data::InventoryItem;

    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CharacterService::new(&mut conn);
    let char_data = create_minimal_character_data("Item Manager");

    let created = service
        .create_character(None, None, false, "", char_data)
        .expect("Failed to create character");

    // Add items via update_character (doesn't require catalog lookup)
    let (_, mut data) = service.get_character(created.id).expect("Failed to get");
    data.inventory.push(InventoryItem {
        name: "Potion of Healing".to_string(),
        source: Some("PHB".to_string()),
        quantity: 5,
        weight: 0.5,
        value: 50.0,
        notes: None,
    });
    service
        .update_character(created.id, data, Some("Added potions".to_string()))
        .expect("Failed to update character");

    // Remove some via remove_item
    service
        .remove_item(created.id, "Potion of Healing", 2)
        .expect("Failed to remove item");

    // Verify
    let (_, data) = service.get_character(created.id).expect("Failed to get");
    assert_eq!(data.inventory[0].quantity, 3);
}

#[tokio::test]
async fn test_update_currency() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CharacterService::new(&mut conn);
    let char_data = create_minimal_character_data("Wealthy");

    let created = service
        .create_character(None, None, false, "", char_data)
        .expect("Failed to create character");

    // Update currency
    service
        .update_currency(created.id, 50, 20, 0, 100, 5)
        .expect("Failed to update currency");

    // Verify
    let (_, data) = service.get_character(created.id).expect("Failed to get");
    assert_eq!(data.currency.copper, 50);
    assert_eq!(data.currency.silver, 20);
    assert_eq!(data.currency.gold, 100);
    assert_eq!(data.currency.platinum, 5);
}

#[tokio::test]
async fn test_update_equipped() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CharacterService::new(&mut conn);
    let char_data = create_minimal_character_data("Equipped");

    let created = service
        .create_character(None, None, false, "", char_data)
        .expect("Failed to create character");

    // Update equipped items
    service
        .update_equipped(
            created.id,
            Some("Chain Mail".to_string()),
            Some("Shield".to_string()),
            Some("Longsword".to_string()),
            None,
        )
        .expect("Failed to update equipped");

    // Verify
    let (_, data) = service.get_character(created.id).expect("Failed to get");
    assert_eq!(data.equipped.armor, Some("Chain Mail".to_string()));
    assert_eq!(data.equipped.shield, Some("Shield".to_string()));
    assert_eq!(data.equipped.main_hand, Some("Longsword".to_string()));
}
