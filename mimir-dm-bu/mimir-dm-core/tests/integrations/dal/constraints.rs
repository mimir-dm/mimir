//! Integration tests for database constraint handling
//!
//! Tests for:
//! - Foreign key constraint violations
//! - Unique constraint violations
//! - Cascade delete behavior

use crate::common::TestDatabase;
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
use mimir_dm_core::dal::campaign::documents::DocumentRepository;
use mimir_dm_core::dal::campaign::module_monsters::ModuleMonsterRepository;
use mimir_dm_core::dal::campaign::modules::ModuleRepository;
use mimir_dm_core::dal::character::CharacterRepository;
use mimir_dm_core::dal::player::{CampaignPlayerRepository, PlayerRepository};
use mimir_dm_core::models::campaign::campaigns::NewCampaign;
use mimir_dm_core::models::campaign::documents::NewDocument;
use mimir_dm_core::models::campaign::module_monsters::NewModuleMonster;
use mimir_dm_core::models::campaign::modules::NewModule;
use mimir_dm_core::models::character::NewCharacter;
use mimir_dm_core::models::player::{NewCampaignPlayer, NewPlayer};
use tempfile::TempDir;

// =============================================================================
// Foreign Key Constraint Tests
// =============================================================================

#[test]
fn test_create_module_with_invalid_campaign_id_fails() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = ModuleRepository::new(&mut conn);
    let result = repo.create(NewModule {
        campaign_id: 99999, // Non-existent campaign
        name: "Invalid Module".to_string(),
        module_number: 1,
        status: "planning".to_string(),
        expected_sessions: 4,
    });

    assert!(result.is_err(), "Should fail due to FK constraint");
}

#[test]
fn test_create_document_with_invalid_campaign_id_fails() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let result = DocumentRepository::create(
        &mut conn,
        NewDocument {
            campaign_id: 99999, // Non-existent campaign
            module_id: None,
            session_id: None,
            template_id: "test".to_string(),
            file_path: "/test/path.md".to_string(),
            title: "Test Document".to_string(),
            document_type: "module_overview".to_string(),
            file_type: "markdown".to_string(),
            is_user_created: false,
        },
    );

    assert!(result.is_err(), "Should fail due to FK constraint");
}

#[test]
fn test_create_module_monster_with_invalid_module_id_fails() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = ModuleMonsterRepository::new(&mut conn);
    let result = repo.create(NewModuleMonster {
        module_id: 99999, // Non-existent module
        monster_name: "Goblin".to_string(),
        monster_source: "MM".to_string(),
        quantity: 4,
        encounter_tag: None,
    });

    assert!(result.is_err(), "Should fail due to FK constraint");
}

#[test]
fn test_create_character_with_invalid_campaign_id_fails() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = CharacterRepository::new(&mut conn);
    let result = repo.create(NewCharacter {
        campaign_id: Some(99999), // Non-existent campaign
        player_id: None,
        character_name: "Invalid Character".to_string(),
        is_npc: Some(false),
        directory_path: "/test/path".to_string(),
        class: None,
        race: None,
    });

    assert!(result.is_err(), "Should fail due to FK constraint");
}

#[test]
fn test_add_player_to_invalid_campaign_fails() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create a valid player first
    let player_id;
    {
        let mut player_repo = PlayerRepository::new(&mut conn);
        let player = player_repo
            .create(NewPlayer {
                name: "Test Player".to_string(),
                email: None,
                notes: None,
            })
            .unwrap();
        player_id = player.id;
    }

    let mut repo = CampaignPlayerRepository::new(&mut conn);
    let result = repo.add(NewCampaignPlayer {
        campaign_id: 99999, // Non-existent campaign
        player_id,
    });

    assert!(result.is_err(), "Should fail due to FK constraint");
}

#[test]
fn test_add_invalid_player_to_campaign_fails() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create a valid campaign
    let campaign_id;
    {
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .create(NewCampaign {
                name: "Test Campaign".to_string(),
                status: "active".to_string(),
                directory_path: temp_dir.path().to_string_lossy().to_string(),
            })
            .unwrap();
        campaign_id = campaign.id;
    }

    let mut repo = CampaignPlayerRepository::new(&mut conn);
    let result = repo.add(NewCampaignPlayer {
        campaign_id,
        player_id: 99999, // Non-existent player
    });

    assert!(result.is_err(), "Should fail due to FK constraint");
}

// =============================================================================
// Cascade Delete Tests
// =============================================================================

#[test]
fn test_cascade_delete_campaign_removes_modules() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create campaign
    let campaign_id;
    {
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .create(NewCampaign {
                name: "Test Campaign".to_string(),
                status: "active".to_string(),
                directory_path: temp_dir.path().to_string_lossy().to_string(),
            })
            .unwrap();
        campaign_id = campaign.id;
    }

    // Create modules
    let module_ids: Vec<i32>;
    {
        let mut module_repo = ModuleRepository::new(&mut conn);
        module_ids = (1..=3)
            .map(|i| {
                module_repo
                    .create(NewModule {
                        campaign_id,
                        name: format!("Module {}", i),
                        module_number: i,
                        status: "planning".to_string(),
                        expected_sessions: 4,
                    })
                    .unwrap()
                    .id
            })
            .collect();
    }

    // Verify modules exist
    {
        let mut module_repo = ModuleRepository::new(&mut conn);
        let modules = module_repo.list_by_campaign(campaign_id).unwrap();
        assert_eq!(modules.len(), 3);
    }

    // Delete campaign
    {
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        campaign_repo.delete(campaign_id).unwrap();
    }

    // Verify modules are also deleted
    {
        let mut module_repo = ModuleRepository::new(&mut conn);
        for module_id in module_ids {
            let result = module_repo.find_by_id(module_id).unwrap();
            assert!(
                result.is_none(),
                "Module should be deleted by cascade delete"
            );
        }
    }
}

#[test]
fn test_cascade_delete_module_removes_monsters() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create campaign and module
    let module_id;
    {
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .create(NewCampaign {
                name: "Test Campaign".to_string(),
                status: "active".to_string(),
                directory_path: temp_dir.path().to_string_lossy().to_string(),
            })
            .unwrap();

        let mut module_repo = ModuleRepository::new(&mut conn);
        let module = module_repo
            .create(NewModule {
                campaign_id: campaign.id,
                name: "Test Module".to_string(),
                module_number: 1,
                status: "planning".to_string(),
                expected_sessions: 4,
            })
            .unwrap();
        module_id = module.id;
    }

    // Create monsters in the module
    let monster_ids: Vec<i32>;
    {
        let mut monster_repo = ModuleMonsterRepository::new(&mut conn);
        monster_ids = vec!["Goblin", "Orc", "Troll"]
            .into_iter()
            .map(|name| {
                monster_repo
                    .create(NewModuleMonster {
                        module_id,
                        monster_name: name.to_string(),
                        monster_source: "MM".to_string(),
                        quantity: 1,
                        encounter_tag: None,
                    })
                    .unwrap()
                    .id
            })
            .collect();
    }

    // Verify monsters exist
    {
        let mut monster_repo = ModuleMonsterRepository::new(&mut conn);
        let monsters = monster_repo.list_by_module(module_id).unwrap();
        assert_eq!(monsters.len(), 3);
    }

    // Delete module
    {
        let mut module_repo = ModuleRepository::new(&mut conn);
        module_repo.delete(module_id).unwrap();
    }

    // Verify monsters are also deleted
    {
        let mut monster_repo = ModuleMonsterRepository::new(&mut conn);
        for monster_id in monster_ids {
            let result = monster_repo.find_by_id(monster_id).unwrap();
            assert!(
                result.is_none(),
                "Monster should be deleted by cascade delete"
            );
        }
    }
}

#[test]
fn test_cascade_delete_campaign_removes_documents() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create campaign
    let campaign_id;
    {
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .create(NewCampaign {
                name: "Test Campaign".to_string(),
                status: "active".to_string(),
                directory_path: temp_dir.path().to_string_lossy().to_string(),
            })
            .unwrap();
        campaign_id = campaign.id;
    }

    // Create documents
    let doc_ids: Vec<i32>;
    {
        doc_ids = (1..=3)
            .map(|i| {
                DocumentRepository::create(
                    &mut conn,
                    NewDocument {
                        campaign_id,
                        module_id: None,
                        session_id: None,
                        template_id: format!("template_{}", i),
                        file_path: format!("/test/doc_{}.md", i),
                        title: format!("Document {}", i),
                        document_type: "module_overview".to_string(),
                        file_type: "markdown".to_string(),
                        is_user_created: false,
                    },
                )
                .unwrap()
                .id
            })
            .collect();
    }

    // Verify documents exist
    {
        let docs = DocumentRepository::find_by_campaign(&mut conn, campaign_id).unwrap();
        assert_eq!(docs.len(), 3);
    }

    // Delete campaign
    {
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        campaign_repo.delete(campaign_id).unwrap();
    }

    // Verify documents are also deleted
    for doc_id in doc_ids {
        let result = DocumentRepository::find_by_id(&mut conn, doc_id);
        assert!(
            result.is_err(),
            "Document should be deleted by cascade delete"
        );
    }
}

#[test]
fn test_cascade_delete_campaign_removes_characters() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create campaign
    let campaign_id;
    {
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .create(NewCampaign {
                name: "Test Campaign".to_string(),
                status: "active".to_string(),
                directory_path: temp_dir.path().to_string_lossy().to_string(),
            })
            .unwrap();
        campaign_id = campaign.id;
    }

    // Create characters
    let char_ids: Vec<i32>;
    {
        let mut char_repo = CharacterRepository::new(&mut conn);
        char_ids = vec!["Hero", "Sidekick", "NPC"]
            .into_iter()
            .map(|name| {
                char_repo
                    .create(NewCharacter {
                        campaign_id: Some(campaign_id),
                        player_id: None,
                        character_name: name.to_string(),
                        is_npc: Some(false),
                        directory_path: temp_dir
                            .path()
                            .join("characters")
                            .to_string_lossy()
                            .to_string(),
                        class: None,
                        race: None,
                    })
                    .unwrap()
                    .id
            })
            .collect();
    }

    // Verify characters exist
    {
        let mut char_repo = CharacterRepository::new(&mut conn);
        let chars = char_repo.list_for_campaign(campaign_id).unwrap();
        assert_eq!(chars.len(), 3);
    }

    // Delete campaign
    {
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        campaign_repo.delete(campaign_id).unwrap();
    }

    // Verify characters are also deleted
    {
        let mut char_repo = CharacterRepository::new(&mut conn);
        for char_id in char_ids {
            let result = char_repo.find_by_id(char_id).unwrap();
            assert!(
                result.is_none(),
                "Character should be deleted by cascade delete"
            );
        }
    }
}

#[test]
fn test_cascade_delete_player_removes_campaign_associations() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create campaign and player
    let campaign_id;
    let player_id;
    {
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .create(NewCampaign {
                name: "Test Campaign".to_string(),
                status: "active".to_string(),
                directory_path: temp_dir.path().to_string_lossy().to_string(),
            })
            .unwrap();
        campaign_id = campaign.id;

        let mut player_repo = PlayerRepository::new(&mut conn);
        let player = player_repo
            .create(NewPlayer {
                name: "Test Player".to_string(),
                email: None,
                notes: None,
            })
            .unwrap();
        player_id = player.id;
    }

    // Add player to campaign
    {
        let mut cp_repo = CampaignPlayerRepository::new(&mut conn);
        cp_repo
            .add(NewCampaignPlayer {
                campaign_id,
                player_id,
            })
            .unwrap();

        // Verify association exists
        assert!(cp_repo.is_player_in_campaign(campaign_id, player_id).unwrap());
    }

    // Delete player
    {
        let mut player_repo = PlayerRepository::new(&mut conn);
        player_repo.delete(player_id).unwrap();
    }

    // Verify association is removed
    {
        let mut cp_repo = CampaignPlayerRepository::new(&mut conn);
        assert!(!cp_repo.is_player_in_campaign(campaign_id, player_id).unwrap());
    }
}

// =============================================================================
// Unique Constraint Tests
// =============================================================================

#[test]
fn test_duplicate_campaign_player_association_handled() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create campaign and player
    let campaign_id;
    let player_id;
    {
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .create(NewCampaign {
                name: "Test Campaign".to_string(),
                status: "active".to_string(),
                directory_path: temp_dir.path().to_string_lossy().to_string(),
            })
            .unwrap();
        campaign_id = campaign.id;

        let mut player_repo = PlayerRepository::new(&mut conn);
        let player = player_repo
            .create(NewPlayer {
                name: "Test Player".to_string(),
                email: None,
                notes: None,
            })
            .unwrap();
        player_id = player.id;
    }

    // Add player to campaign
    let mut cp_repo = CampaignPlayerRepository::new(&mut conn);
    cp_repo
        .add(NewCampaignPlayer {
            campaign_id,
            player_id,
        })
        .unwrap();

    // Try to add same player again - should fail due to unique constraint
    let result = cp_repo.add(NewCampaignPlayer {
        campaign_id,
        player_id,
    });

    assert!(
        result.is_err(),
        "Should fail due to unique constraint on campaign_player association"
    );
}
