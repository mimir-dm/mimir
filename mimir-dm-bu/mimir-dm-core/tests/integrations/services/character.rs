//! Integration tests for CharacterService

use diesel::prelude::*;
use mimir_dm_core::services::character::creation::{AbilityScoreMethod, CharacterBuilder};
use mimir_dm_core::services::{CharacterService, PlayerService};
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");

    // Seed PHB class data for testing
    seed_test_class_data(&mut conn);

    (conn, temp_dir)
}

fn seed_test_class_data(conn: &mut SqliteConnection) {
    // Insert Wizard class for spell testing
    let wizard_json = r#"{
        "name": "Wizard",
        "source": "PHB",
        "hd": {"number": 1, "faces": 6},
        "casterProgression": "full",
        "spellcastingAbility": "int",
        "classTableGroups": [{
            "colLabels": ["1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th", "9th"],
            "rowsSpellProgression": [
                [2, 0, 0, 0, 0, 0, 0, 0, 0],
                [3, 0, 0, 0, 0, 0, 0, 0, 0],
                [4, 2, 0, 0, 0, 0, 0, 0, 0]
            ]
        }]
    }"#;

    diesel::sql_query(
        "INSERT INTO catalog_classes (name, source, hit_dice, full_class_json) VALUES (?, ?, ?, ?)",
    )
    .bind::<diesel::sql_types::Text, _>("Wizard")
    .bind::<diesel::sql_types::Text, _>("PHB")
    .bind::<diesel::sql_types::Text, _>("d6")
    .bind::<diesel::sql_types::Text, _>(wizard_json)
    .execute(conn)
    .ok();

    // Insert Fighter class for non-spellcaster testing
    let fighter_json = r#"{
        "name": "Fighter",
        "source": "PHB",
        "hd": {"number": 1, "faces": 10},
        "classTableGroups": []
    }"#;

    diesel::sql_query(
        "INSERT INTO catalog_classes (name, source, hit_dice, full_class_json) VALUES (?, ?, ?, ?)",
    )
    .bind::<diesel::sql_types::Text, _>("Fighter")
    .bind::<diesel::sql_types::Text, _>("PHB")
    .bind::<diesel::sql_types::Text, _>("d10")
    .bind::<diesel::sql_types::Text, _>(fighter_json)
    .execute(conn)
    .ok();

    // Insert Human race
    let human_json = r#"{
        "name": "Human",
        "source": "PHB",
        "size": ["M"],
        "speed": 30,
        "ability": [{"str": 1, "dex": 1, "con": 1, "int": 1, "wis": 1, "cha": 1}]
    }"#;

    diesel::sql_query(
        "INSERT INTO catalog_races (name, source, size, speed, full_race_json) VALUES (?, ?, ?, ?, ?)"
    )
    .bind::<diesel::sql_types::Text, _>("Human")
    .bind::<diesel::sql_types::Text, _>("PHB")
    .bind::<diesel::sql_types::Text, _>("M")
    .bind::<diesel::sql_types::Integer, _>(30)
    .bind::<diesel::sql_types::Text, _>(human_json)
    .execute(conn)
    .ok();

    // Insert Sage background
    let sage_json = r#"{
        "name": "Sage",
        "source": "PHB",
        "skillProficiencies": ["Arcana", "History"]
    }"#;

    diesel::sql_query(
        "INSERT INTO catalog_backgrounds (name, skills, languages, tools, feature, source, full_background_json) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind::<diesel::sql_types::Text, _>("Sage")
    .bind::<diesel::sql_types::Text, _>("Arcana, History")
    .bind::<diesel::sql_types::Text, _>("")
    .bind::<diesel::sql_types::Text, _>("")
    .bind::<diesel::sql_types::Text, _>("Researcher")
    .bind::<diesel::sql_types::Text, _>("PHB")
    .bind::<diesel::sql_types::Text, _>(sage_json)
    .execute(conn)
    .ok();
}

fn create_test_campaign(conn: &mut SqliteConnection, temp_dir: &TempDir) -> i32 {
    use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
    use mimir_dm_core::models::campaign::NewCampaign;

    let mut campaign_repo = CampaignRepository::new(conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            directory_path: temp_dir.path().to_str().unwrap().to_string(),
            status: "concept".to_string(),
        })
        .unwrap();

    campaign.id
}

fn create_test_player(conn: &mut SqliteConnection) -> i32 {
    let mut player_service = PlayerService::new(conn);
    let player = player_service
        .create_player("Test Player", None, None)
        .unwrap();
    player.id
}

#[test]
fn test_create_character_basic() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);
    let player_id = create_test_player(&mut conn);

    let char_data = CharacterBuilder::new(&mut conn)
        .set_identity("Gandalf".to_string(), Some(player_id))
        .set_race("Human", "PHB", None)
        .unwrap()
        .set_class("Wizard", "PHB", None)
        .unwrap()
        .set_background("Sage", "PHB")
        .unwrap()
        .set_ability_scores(AbilityScoreMethod::Manual {
            strength: 10,
            dexterity: 12,
            constitution: 14,
            intelligence: 16,
            wisdom: 13,
            charisma: 8,
        })
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(char_data.character_name, "Gandalf");
    assert_eq!(char_data.level, 1);
    assert_eq!(char_data.abilities.intelligence, 17); // 16 + 1 from Human

    // Store character
    let mut char_service = CharacterService::new(&mut conn);
    let character = char_service
        .create_character(
            Some(campaign_id),
            Some(player_id),
            false,
            temp_dir.path().to_str().unwrap(),
            char_data,
        )
        .unwrap();

    assert_eq!(character.character_name, "Gandalf");
    assert_eq!(character.player_id, Some(player_id));
}

#[test]
fn test_get_character() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);
    let player_id = create_test_player(&mut conn);

    // Create character
    let char_data = CharacterBuilder::new(&mut conn)
        .set_identity("Aragorn".to_string(), Some(player_id))
        .set_race("Human", "PHB", None)
        .unwrap()
        .set_class("Fighter", "PHB", None)
        .unwrap()
        .set_background("Sage", "PHB")
        .unwrap()
        .set_ability_scores(AbilityScoreMethod::Manual {
            strength: 16,
            dexterity: 14,
            constitution: 14,
            intelligence: 10,
            wisdom: 12,
            charisma: 13,
        })
        .unwrap()
        .build()
        .unwrap();

    let mut char_service = CharacterService::new(&mut conn);
    let character = char_service
        .create_character(
            Some(campaign_id),
            Some(player_id),
            false,
            temp_dir.path().to_str().unwrap(),
            char_data,
        )
        .unwrap();

    // Get character
    let (retrieved_char, retrieved_data) = char_service.get_character(character.id).unwrap();

    assert_eq!(retrieved_char.id, character.id);
    assert_eq!(retrieved_data.character_name, "Aragorn");
    assert_eq!(retrieved_data.abilities.strength, 17); // 16 + 1 from Human
}

#[test]
fn test_list_characters_for_campaign() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);
    let player_id = create_test_player(&mut conn);

    // Create multiple characters
    for name in &["Frodo", "Sam", "Merry"] {
        let builder = CharacterBuilder::new(&mut conn);
        let char_data = builder
            .set_identity(name.to_string(), Some(player_id))
            .set_race("Human", "PHB", None)
            .unwrap()
            .set_class("Wizard", "PHB", None)
            .unwrap()
            .set_background("Sage", "PHB")
            .unwrap()
            .set_ability_scores(AbilityScoreMethod::Manual {
                strength: 10,
                dexterity: 12,
                constitution: 14,
                intelligence: 10,
                wisdom: 12,
                charisma: 10,
            })
            .unwrap()
            .build()
            .unwrap();

        let mut char_service = CharacterService::new(&mut conn);
        char_service
            .create_character(
                Some(campaign_id),
                Some(player_id),
                false,
                temp_dir.path().to_str().unwrap(),
                char_data,
            )
            .unwrap();
    }

    let mut char_service = CharacterService::new(&mut conn);
    let characters = char_service
        .list_characters_for_campaign(campaign_id)
        .unwrap();
    assert_eq!(characters.len(), 3);
}

#[test]
fn test_character_versioning() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);
    let player_id = create_test_player(&mut conn);

    // Create character
    let char_data = CharacterBuilder::new(&mut conn)
        .set_identity("Legolas".to_string(), Some(player_id))
        .set_race("Human", "PHB", None)
        .unwrap()
        .set_class("Fighter", "PHB", None)
        .unwrap()
        .set_background("Sage", "PHB")
        .unwrap()
        .set_ability_scores(AbilityScoreMethod::Manual {
            strength: 10,
            dexterity: 18,
            constitution: 12,
            intelligence: 10,
            wisdom: 14,
            charisma: 12,
        })
        .unwrap()
        .build()
        .unwrap();

    let mut char_service = CharacterService::new(&mut conn);
    let character = char_service
        .create_character(
            Some(campaign_id),
            Some(player_id),
            false,
            temp_dir.path().to_str().unwrap(),
            char_data,
        )
        .unwrap();

    // Get initial version
    let versions = char_service.get_character_versions(character.id).unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].version_number, 1);

    // Update character (creates new version)
    let (_, mut char_data) = char_service.get_character(character.id).unwrap();
    char_data.level = 2;
    char_service
        .update_character(character.id, char_data, Some("Leveled up".to_string()))
        .unwrap();

    // Check versions
    let versions = char_service.get_character_versions(character.id).unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[1].version_number, 2);

    // Get specific version
    let version_1_data = char_service.get_character_version(character.id, 1).unwrap();
    assert_eq!(version_1_data.level, 1);

    let version_2_data = char_service.get_character_version(character.id, 2).unwrap();
    assert_eq!(version_2_data.level, 2);
}

#[test]
fn test_delete_character() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);
    let player_id = create_test_player(&mut conn);

    // Create character
    let char_data = CharacterBuilder::new(&mut conn)
        .set_identity("Boromir".to_string(), Some(player_id))
        .set_race("Human", "PHB", None)
        .unwrap()
        .set_class("Fighter", "PHB", None)
        .unwrap()
        .set_background("Sage", "PHB")
        .unwrap()
        .set_ability_scores(AbilityScoreMethod::Manual {
            strength: 16,
            dexterity: 10,
            constitution: 14,
            intelligence: 10,
            wisdom: 10,
            charisma: 12,
        })
        .unwrap()
        .build()
        .unwrap();

    let mut char_service = CharacterService::new(&mut conn);
    let character = char_service
        .create_character(
            Some(campaign_id),
            Some(player_id),
            false,
            temp_dir.path().to_str().unwrap(),
            char_data,
        )
        .unwrap();

    // Delete character
    char_service.delete_character(character.id).unwrap();

    // Try to get character - should fail
    let result = char_service.get_character(character.id);
    assert!(result.is_err());
}

#[test]
fn test_update_character_data() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);
    let player_id = create_test_player(&mut conn);

    // Create character
    let char_data = CharacterBuilder::new(&mut conn)
        .set_identity("Gimli".to_string(), Some(player_id))
        .set_race("Human", "PHB", None)
        .unwrap()
        .set_class("Fighter", "PHB", None)
        .unwrap()
        .set_background("Sage", "PHB")
        .unwrap()
        .set_ability_scores(AbilityScoreMethod::Manual {
            strength: 16,
            dexterity: 10,
            constitution: 16,
            intelligence: 8,
            wisdom: 12,
            charisma: 10,
        })
        .unwrap()
        .build()
        .unwrap();

    let mut char_service = CharacterService::new(&mut conn);
    let character = char_service
        .create_character(
            Some(campaign_id),
            Some(player_id),
            false,
            temp_dir.path().to_str().unwrap(),
            char_data,
        )
        .unwrap();

    // Update character
    let (_, mut char_data) = char_service.get_character(character.id).unwrap();
    char_data.current_hp = 5;
    char_data.max_hp = 20;

    let version = char_service
        .update_character(character.id, char_data, Some("Took damage".to_string()))
        .unwrap();

    assert_eq!(version.snapshot_reason, Some("Took damage".to_string()));

    // Verify update
    let (_, updated_data) = char_service.get_character(character.id).unwrap();
    assert_eq!(updated_data.current_hp, 5);
    assert_eq!(updated_data.max_hp, 20);
}

#[test]
fn test_proficiency_bonus_calculation() {
    let (mut conn, _temp_dir) = setup_test_db();

    let char_data = CharacterBuilder::new(&mut conn)
        .set_identity("Test".to_string(), Some(1))
        .set_race("Human", "PHB", None)
        .unwrap()
        .set_class("Wizard", "PHB", None)
        .unwrap()
        .set_background("Sage", "PHB")
        .unwrap()
        .set_ability_scores(AbilityScoreMethod::Manual {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        })
        .unwrap()
        .build()
        .unwrap();

    // Test proficiency bonus at different levels
    assert_eq!(char_data.proficiency_bonus(), 2); // Level 1

    let mut char_data_level_5 = char_data.clone();
    char_data_level_5.level = 5;
    assert_eq!(char_data_level_5.proficiency_bonus(), 3);

    let mut char_data_level_9 = char_data.clone();
    char_data_level_9.level = 9;
    assert_eq!(char_data_level_9.proficiency_bonus(), 4);

    let mut char_data_level_17 = char_data.clone();
    char_data_level_17.level = 17;
    assert_eq!(char_data_level_17.proficiency_bonus(), 6);
}
