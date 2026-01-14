//! Integration tests for character DAL

use crate::common::TestDatabase;
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
use mimir_dm_core::dal::character::{CharacterRepository, CharacterVersionRepository};
use mimir_dm_core::dal::player::PlayerRepository;
use mimir_dm_core::models::campaign::campaigns::NewCampaign;
use mimir_dm_core::models::character::{NewCharacter, NewCharacterVersion, UpdateCharacter};
use mimir_dm_core::models::player::NewPlayer;
use tempfile::TempDir;

/// Helper to create a test campaign
fn setup_campaign(conn: &mut diesel::SqliteConnection, temp_dir: &TempDir) -> i32 {
    let mut campaign_repo = CampaignRepository::new(conn);
    campaign_repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: temp_dir.path().to_string_lossy().to_string(),
        })
        .unwrap()
        .id
}

/// Helper to create a test player
fn setup_player(conn: &mut diesel::SqliteConnection) -> i32 {
    let mut player_repo = PlayerRepository::new(conn);
    player_repo
        .create(NewPlayer {
            name: "Test Player".to_string(),
            email: None,
            notes: None,
        })
        .unwrap()
        .id
}

// =============================================================================
// CharacterRepository Tests
// =============================================================================

#[test]
fn test_create_character() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut repo = CharacterRepository::new(&mut conn);
    let character = repo
        .create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: "Thorin Ironforge".to_string(),
            is_npc: Some(false),
            directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
            class: Some("Fighter".to_string()),
            race: Some("Dwarf".to_string()),
        })
        .unwrap();

    assert_eq!(character.character_name, "Thorin Ironforge");
    assert_eq!(character.campaign_id, Some(campaign_id));
    assert!(!character.is_npc);
    assert_eq!(character.class, Some("Fighter".to_string()));
    assert_eq!(character.race, Some("Dwarf".to_string()));
}

#[test]
fn test_create_npc() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut repo = CharacterRepository::new(&mut conn);
    let npc = repo
        .create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: "Gundren Rockseeker".to_string(),
            is_npc: Some(true),
            directory_path: temp_dir.path().join("npcs").to_string_lossy().to_string(),
            class: None,
            race: Some("Dwarf".to_string()),
        })
        .unwrap();

    assert_eq!(npc.character_name, "Gundren Rockseeker");
    assert!(npc.is_npc);
}

#[test]
fn test_find_character_by_id() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut repo = CharacterRepository::new(&mut conn);
    let created = repo
        .create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: "Elara Nightwhisper".to_string(),
            is_npc: Some(false),
            directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
            class: Some("Rogue".to_string()),
            race: Some("Elf".to_string()),
        })
        .unwrap();

    let found = repo.find_by_id(created.id).unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.character_name, "Elara Nightwhisper");
}

#[test]
fn test_find_nonexistent_character() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = CharacterRepository::new(&mut conn);
    let result = repo.find_by_id(99999).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_update_character() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut repo = CharacterRepository::new(&mut conn);
    let created = repo
        .create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: "Test Character".to_string(),
            is_npc: Some(false),
            directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
            class: Some("Wizard".to_string()),
            race: None,
        })
        .unwrap();

    let updated = repo
        .update(
            created.id,
            UpdateCharacter {
                character_name: Some("Gandalf the Grey".to_string()),
                is_npc: None,
                current_level: Some(10),
                current_version: None,
                updated_at: None,
                campaign_id: None,
                directory_path: None,
            },
        )
        .unwrap();

    assert_eq!(updated.character_name, "Gandalf the Grey");
    assert_eq!(updated.current_level, 10);
}

#[test]
fn test_delete_character() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut repo = CharacterRepository::new(&mut conn);
    let created = repo
        .create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: "Expendable NPC".to_string(),
            is_npc: Some(true),
            directory_path: temp_dir.path().join("npcs").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap();

    repo.delete(created.id).unwrap();

    let found = repo.find_by_id(created.id).unwrap();
    assert!(found.is_none());
}

#[test]
fn test_list_all_characters() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut repo = CharacterRepository::new(&mut conn);

    // Create some characters
    for name in &["Alice", "Bob", "Charlie"] {
        repo.create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: name.to_string(),
            is_npc: Some(false),
            directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap();
    }

    let all = repo.list_all().unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn test_list_characters_for_campaign() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let other_campaign = campaign_repo
        .create(NewCampaign {
            name: "Other Campaign".to_string(),
            status: "active".to_string(),
            directory_path: temp_dir.path().join("other").to_string_lossy().to_string(),
        })
        .unwrap();

    let mut repo = CharacterRepository::new(&mut conn);

    // Create characters in first campaign
    for name in &["PC1", "PC2"] {
        repo.create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: name.to_string(),
            is_npc: Some(false),
            directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap();
    }

    // Create character in other campaign
    repo.create(NewCharacter {
        campaign_id: Some(other_campaign.id),
        player_id: None,
        character_name: "Other PC".to_string(),
        is_npc: Some(false),
        directory_path: temp_dir.path().join("other/chars").to_string_lossy().to_string(),
        class: None,
        race: None,
    })
    .unwrap();

    let campaign_chars = repo.list_for_campaign(campaign_id).unwrap();
    assert_eq!(campaign_chars.len(), 2);
}

#[test]
fn test_list_characters_for_player() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);
    let player_id = setup_player(&mut conn);

    let mut repo = CharacterRepository::new(&mut conn);

    // Create characters for the player
    repo.create(NewCharacter {
        campaign_id: Some(campaign_id),
        player_id: Some(player_id),
        character_name: "Player's First Character".to_string(),
        is_npc: Some(false),
        directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
        class: None,
        race: None,
    })
    .unwrap();

    repo.create(NewCharacter {
        campaign_id: Some(campaign_id),
        player_id: Some(player_id),
        character_name: "Player's Second Character".to_string(),
        is_npc: Some(false),
        directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
        class: None,
        race: None,
    })
    .unwrap();

    // Create character for another player (none)
    repo.create(NewCharacter {
        campaign_id: Some(campaign_id),
        player_id: None,
        character_name: "Unassigned Character".to_string(),
        is_npc: Some(false),
        directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
        class: None,
        race: None,
    })
    .unwrap();

    let player_chars = repo.list_for_player(campaign_id, player_id).unwrap();
    assert_eq!(player_chars.len(), 2);
}

#[test]
fn test_list_npcs() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut repo = CharacterRepository::new(&mut conn);

    // Create NPCs
    for name in &["NPC1", "NPC2", "NPC3"] {
        repo.create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: name.to_string(),
            is_npc: Some(true),
            directory_path: temp_dir.path().join("npcs").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap();
    }

    // Create a PC
    repo.create(NewCharacter {
        campaign_id: Some(campaign_id),
        player_id: None,
        character_name: "Player Character".to_string(),
        is_npc: Some(false),
        directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
        class: None,
        race: None,
    })
    .unwrap();

    let npcs = repo.list_npcs(campaign_id).unwrap();
    assert_eq!(npcs.len(), 3);
}

#[test]
fn test_list_pcs() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut repo = CharacterRepository::new(&mut conn);

    // Create PCs
    for name in &["PC1", "PC2"] {
        repo.create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: name.to_string(),
            is_npc: Some(false),
            directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap();
    }

    // Create an NPC
    repo.create(NewCharacter {
        campaign_id: Some(campaign_id),
        player_id: None,
        character_name: "NPC".to_string(),
        is_npc: Some(true),
        directory_path: temp_dir.path().join("npcs").to_string_lossy().to_string(),
        class: None,
        race: None,
    })
    .unwrap();

    let pcs = repo.list_pcs(campaign_id).unwrap();
    assert_eq!(pcs.len(), 2);
}

#[test]
fn test_find_character_by_name_in_campaign() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut repo = CharacterRepository::new(&mut conn);

    repo.create(NewCharacter {
        campaign_id: Some(campaign_id),
        player_id: None,
        character_name: "Unique Name".to_string(),
        is_npc: Some(false),
        directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
        class: None,
        race: None,
    })
    .unwrap();

    let found = repo
        .find_by_name_in_campaign(campaign_id, "Unique Name")
        .unwrap();
    assert!(found.is_some());

    let not_found = repo
        .find_by_name_in_campaign(campaign_id, "Nonexistent")
        .unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_find_npc_by_name_in_campaign() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut repo = CharacterRepository::new(&mut conn);

    // Create an NPC
    repo.create(NewCharacter {
        campaign_id: Some(campaign_id),
        player_id: None,
        character_name: "Barkeep".to_string(),
        is_npc: Some(true),
        directory_path: temp_dir.path().join("npcs").to_string_lossy().to_string(),
        class: None,
        race: None,
    })
    .unwrap();

    // Create a PC with same name
    repo.create(NewCharacter {
        campaign_id: Some(campaign_id),
        player_id: None,
        character_name: "Hero".to_string(),
        is_npc: Some(false),
        directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
        class: None,
        race: None,
    })
    .unwrap();

    // Should find NPC
    let found = repo
        .find_npc_by_name_in_campaign(campaign_id, "Barkeep")
        .unwrap();
    assert!(found.is_some());

    // Should not find PC as NPC
    let not_found = repo
        .find_npc_by_name_in_campaign(campaign_id, "Hero")
        .unwrap();
    assert!(not_found.is_none());
}

// =============================================================================
// CharacterVersionRepository Tests
// =============================================================================

#[test]
fn test_create_character_version() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut char_repo = CharacterRepository::new(&mut conn);
    let character = char_repo
        .create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: "Test Character".to_string(),
            is_npc: Some(false),
            directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap();

    let mut version_repo = CharacterVersionRepository::new(&mut conn);
    let version = version_repo
        .create(NewCharacterVersion {
            character_id: character.id,
            version_number: 1,
            file_path: "/path/to/char_v1.yaml".to_string(),
            character_data: r#"{"name": "Test Character", "level": 1}"#.to_string(),
            snapshot_reason: Some("Initial creation".to_string()),
            level: 1,
        })
        .unwrap();

    assert_eq!(version.character_id, character.id);
    assert_eq!(version.version_number, 1);
    assert_eq!(version.level, 1);
    assert_eq!(version.snapshot_reason, Some("Initial creation".to_string()));
}

#[test]
fn test_find_character_version() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut char_repo = CharacterRepository::new(&mut conn);
    let character = char_repo
        .create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: "Test Character".to_string(),
            is_npc: Some(false),
            directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap();

    let mut version_repo = CharacterVersionRepository::new(&mut conn);

    // Create two versions
    version_repo
        .create(NewCharacterVersion {
            character_id: character.id,
            version_number: 1,
            file_path: "/path/to/char_v1.yaml".to_string(),
            character_data: r#"{"level": 1}"#.to_string(),
            snapshot_reason: None,
            level: 1,
        })
        .unwrap();

    version_repo
        .create(NewCharacterVersion {
            character_id: character.id,
            version_number: 2,
            file_path: "/path/to/char_v2.yaml".to_string(),
            character_data: r#"{"level": 2}"#.to_string(),
            snapshot_reason: Some("Level up".to_string()),
            level: 2,
        })
        .unwrap();

    // Find specific version
    let found = version_repo
        .find_by_character_and_version(character.id, 1)
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().level, 1);

    let found_v2 = version_repo
        .find_by_character_and_version(character.id, 2)
        .unwrap();
    assert!(found_v2.is_some());
    assert_eq!(found_v2.unwrap().level, 2);
}

#[test]
fn test_find_latest_version() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut char_repo = CharacterRepository::new(&mut conn);
    let character = char_repo
        .create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: "Test Character".to_string(),
            is_npc: Some(false),
            directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap();

    let mut version_repo = CharacterVersionRepository::new(&mut conn);

    // Create multiple versions
    for v in 1..=5 {
        version_repo
            .create(NewCharacterVersion {
                character_id: character.id,
                version_number: v,
                file_path: format!("/path/to/char_v{}.yaml", v),
                character_data: format!(r#"{{"level": {}}}"#, v),
                snapshot_reason: None,
                level: v,
            })
            .unwrap();
    }

    let latest = version_repo.find_latest(character.id).unwrap();
    assert!(latest.is_some());
    assert_eq!(latest.unwrap().version_number, 5);
}

#[test]
fn test_list_versions_for_character() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut char_repo = CharacterRepository::new(&mut conn);
    let character = char_repo
        .create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: "Test Character".to_string(),
            is_npc: Some(false),
            directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap();

    let mut version_repo = CharacterVersionRepository::new(&mut conn);

    // Create 3 versions
    for v in 1..=3 {
        version_repo
            .create(NewCharacterVersion {
                character_id: character.id,
                version_number: v,
                file_path: format!("/path/to/char_v{}.yaml", v),
                character_data: format!(r#"{{"level": {}}}"#, v),
                snapshot_reason: None,
                level: v,
            })
            .unwrap();
    }

    let versions = version_repo.list_for_character(character.id).unwrap();
    assert_eq!(versions.len(), 3);

    // Should be ordered ascending
    assert_eq!(versions[0].version_number, 1);
    assert_eq!(versions[1].version_number, 2);
    assert_eq!(versions[2].version_number, 3);
}

#[test]
fn test_get_next_version_number() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut char_repo = CharacterRepository::new(&mut conn);
    let character = char_repo
        .create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: "Test Character".to_string(),
            is_npc: Some(false),
            directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap();

    let mut version_repo = CharacterVersionRepository::new(&mut conn);

    // No versions yet - should return 1
    let next = version_repo.get_next_version_number(character.id).unwrap();
    assert_eq!(next, 1);

    // Create a version
    version_repo
        .create(NewCharacterVersion {
            character_id: character.id,
            version_number: 1,
            file_path: "/path/to/char_v1.yaml".to_string(),
            character_data: r#"{}"#.to_string(),
            snapshot_reason: None,
            level: 1,
        })
        .unwrap();

    // Next should be 2
    let next = version_repo.get_next_version_number(character.id).unwrap();
    assert_eq!(next, 2);
}

#[test]
fn test_update_version_file_path() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut char_repo = CharacterRepository::new(&mut conn);
    let character = char_repo
        .create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: "Test Character".to_string(),
            is_npc: Some(false),
            directory_path: temp_dir.path().join("characters").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap();

    let mut version_repo = CharacterVersionRepository::new(&mut conn);
    let version = version_repo
        .create(NewCharacterVersion {
            character_id: character.id,
            version_number: 1,
            file_path: "/old/path.yaml".to_string(),
            character_data: r#"{}"#.to_string(),
            snapshot_reason: None,
            level: 1,
        })
        .unwrap();

    version_repo
        .update_file_path(version.id, "/new/path.yaml".to_string())
        .unwrap();

    let updated = version_repo
        .find_by_character_and_version(character.id, 1)
        .unwrap()
        .unwrap();
    assert_eq!(updated.file_path, "/new/path.yaml");
}
