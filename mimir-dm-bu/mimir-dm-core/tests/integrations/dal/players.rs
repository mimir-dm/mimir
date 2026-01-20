//! Integration tests for player DAL

use crate::common::TestDatabase;
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
use mimir_dm_core::dal::player::{CampaignPlayerRepository, PlayerRepository};
use mimir_dm_core::models::campaign::campaigns::NewCampaign;
use mimir_dm_core::models::player::{NewCampaignPlayer, NewPlayer, UpdateCampaignPlayer, UpdatePlayer};
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

// =============================================================================
// PlayerRepository Tests
// =============================================================================

#[test]
fn test_create_player() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = PlayerRepository::new(&mut conn);
    let player = repo
        .create(NewPlayer {
            name: "John Smith".to_string(),
            email: Some("john@example.com".to_string()),
            notes: Some("Prefers roleplay over combat".to_string()),
        })
        .unwrap();

    assert_eq!(player.name, "John Smith");
    assert_eq!(player.email, Some("john@example.com".to_string()));
    assert_eq!(player.notes, Some("Prefers roleplay over combat".to_string()));
}

#[test]
fn test_create_player_minimal() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = PlayerRepository::new(&mut conn);
    let player = repo
        .create(NewPlayer {
            name: "Jane Doe".to_string(),
            email: None,
            notes: None,
        })
        .unwrap();

    assert_eq!(player.name, "Jane Doe");
    assert!(player.email.is_none());
    assert!(player.notes.is_none());
}

#[test]
fn test_find_player_by_id() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = PlayerRepository::new(&mut conn);
    let created = repo
        .create(NewPlayer {
            name: "Test Player".to_string(),
            email: None,
            notes: None,
        })
        .unwrap();

    let found = repo.find_by_id(created.id).unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "Test Player");
}

#[test]
fn test_find_nonexistent_player() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = PlayerRepository::new(&mut conn);
    let result = repo.find_by_id(99999).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_update_player() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = PlayerRepository::new(&mut conn);
    let created = repo
        .create(NewPlayer {
            name: "Old Name".to_string(),
            email: None,
            notes: None,
        })
        .unwrap();

    let updated = repo
        .update(
            created.id,
            UpdatePlayer {
                name: Some("New Name".to_string()),
                email: Some(Some("new@email.com".to_string())),
                notes: Some(Some("Updated notes".to_string())),
            },
        )
        .unwrap();

    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.email, Some("new@email.com".to_string()));
    assert_eq!(updated.notes, Some("Updated notes".to_string()));
}

#[test]
fn test_update_player_clear_optional_fields() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = PlayerRepository::new(&mut conn);
    let created = repo
        .create(NewPlayer {
            name: "Test".to_string(),
            email: Some("test@email.com".to_string()),
            notes: Some("Some notes".to_string()),
        })
        .unwrap();

    // Clear the optional fields
    let updated = repo
        .update(
            created.id,
            UpdatePlayer {
                name: None,
                email: Some(None), // Clear email
                notes: Some(None), // Clear notes
            },
        )
        .unwrap();

    assert_eq!(updated.name, "Test"); // Unchanged
    assert!(updated.email.is_none());
    assert!(updated.notes.is_none());
}

#[test]
fn test_delete_player() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = PlayerRepository::new(&mut conn);
    let created = repo
        .create(NewPlayer {
            name: "To Delete".to_string(),
            email: None,
            notes: None,
        })
        .unwrap();

    repo.delete(created.id).unwrap();

    let found = repo.find_by_id(created.id).unwrap();
    assert!(found.is_none());
}

#[test]
fn test_list_players() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = PlayerRepository::new(&mut conn);

    // Create multiple players
    for name in &["Alice", "Bob", "Charlie"] {
        repo.create(NewPlayer {
            name: name.to_string(),
            email: None,
            notes: None,
        })
        .unwrap();
    }

    let all = repo.list().unwrap();
    assert_eq!(all.len(), 3);

    // Should be ordered by name
    assert_eq!(all[0].name, "Alice");
    assert_eq!(all[1].name, "Bob");
    assert_eq!(all[2].name, "Charlie");
}

// =============================================================================
// CampaignPlayerRepository Tests
// =============================================================================

#[test]
fn test_add_player_to_campaign() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut player_repo = PlayerRepository::new(&mut conn);
    let player = player_repo
        .create(NewPlayer {
            name: "Test Player".to_string(),
            email: None,
            notes: None,
        })
        .unwrap();

    let mut repo = CampaignPlayerRepository::new(&mut conn);
    let association = repo
        .add(NewCampaignPlayer {
            campaign_id,
            player_id: player.id,
        })
        .unwrap();

    assert_eq!(association.campaign_id, campaign_id);
    assert_eq!(association.player_id, player.id);
    assert!(association.active); // Default should be true
}

#[test]
fn test_remove_player_from_campaign() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut player_repo = PlayerRepository::new(&mut conn);
    let player = player_repo
        .create(NewPlayer {
            name: "Test Player".to_string(),
            email: None,
            notes: None,
        })
        .unwrap();

    let mut repo = CampaignPlayerRepository::new(&mut conn);
    repo.add(NewCampaignPlayer {
        campaign_id,
        player_id: player.id,
    })
    .unwrap();

    // Verify player is in campaign
    assert!(repo.is_player_in_campaign(campaign_id, player.id).unwrap());

    // Remove player
    repo.remove(campaign_id, player.id).unwrap();

    // Verify player is no longer in campaign
    assert!(!repo.is_player_in_campaign(campaign_id, player.id).unwrap());
}

#[test]
fn test_update_campaign_player_active_status() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut player_repo = PlayerRepository::new(&mut conn);
    let player = player_repo
        .create(NewPlayer {
            name: "Test Player".to_string(),
            email: None,
            notes: None,
        })
        .unwrap();

    let mut repo = CampaignPlayerRepository::new(&mut conn);
    repo.add(NewCampaignPlayer {
        campaign_id,
        player_id: player.id,
    })
    .unwrap();

    // Set player as inactive
    let updated = repo
        .update(
            campaign_id,
            player.id,
            UpdateCampaignPlayer { active: Some(false) },
        )
        .unwrap();

    assert!(!updated.active);

    // Reactivate
    let reactivated = repo
        .update(
            campaign_id,
            player.id,
            UpdateCampaignPlayer { active: Some(true) },
        )
        .unwrap();

    assert!(reactivated.active);
}

#[test]
fn test_list_players_for_campaign() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    // Create players and collect their IDs
    let mut player_ids = Vec::new();
    {
        let mut player_repo = PlayerRepository::new(&mut conn);
        for name in &["Player 1", "Player 2", "Player 3"] {
            let player = player_repo
                .create(NewPlayer {
                    name: name.to_string(),
                    email: None,
                    notes: None,
                })
                .unwrap();
            player_ids.push(player.id);
        }

        // Create a player not in this campaign
        player_repo
            .create(NewPlayer {
                name: "Not In Campaign".to_string(),
                email: None,
                notes: None,
            })
            .unwrap();
    }

    // Add players to campaign
    let mut campaign_player_repo = CampaignPlayerRepository::new(&mut conn);
    for player_id in player_ids {
        campaign_player_repo
            .add(NewCampaignPlayer {
                campaign_id,
                player_id,
            })
            .unwrap();
    }

    let campaign_players = campaign_player_repo.list_for_campaign(campaign_id).unwrap();
    assert_eq!(campaign_players.len(), 3);
}

#[test]
fn test_list_active_players_for_campaign() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    // Create players and collect their IDs
    let mut active_player_ids = Vec::new();
    let inactive_id;
    {
        let mut player_repo = PlayerRepository::new(&mut conn);

        // Create active players
        for name in &["Active 1", "Active 2"] {
            let player = player_repo
                .create(NewPlayer {
                    name: name.to_string(),
                    email: None,
                    notes: None,
                })
                .unwrap();
            active_player_ids.push(player.id);
        }

        // Create inactive player
        let inactive = player_repo
            .create(NewPlayer {
                name: "Inactive Player".to_string(),
                email: None,
                notes: None,
            })
            .unwrap();
        inactive_id = inactive.id;
    }

    // Add players to campaign
    let mut campaign_player_repo = CampaignPlayerRepository::new(&mut conn);
    for player_id in active_player_ids {
        campaign_player_repo
            .add(NewCampaignPlayer {
                campaign_id,
                player_id,
            })
            .unwrap();
    }

    campaign_player_repo
        .add(NewCampaignPlayer {
            campaign_id,
            player_id: inactive_id,
        })
        .unwrap();

    campaign_player_repo
        .update(
            campaign_id,
            inactive_id,
            UpdateCampaignPlayer { active: Some(false) },
        )
        .unwrap();

    let active_players = campaign_player_repo
        .list_active_for_campaign(campaign_id)
        .unwrap();
    assert_eq!(active_players.len(), 2);
}

#[test]
fn test_is_player_in_campaign() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign_id = setup_campaign(&mut conn, &temp_dir);

    let mut player_repo = PlayerRepository::new(&mut conn);
    let player_in = player_repo
        .create(NewPlayer {
            name: "In Campaign".to_string(),
            email: None,
            notes: None,
        })
        .unwrap();

    let player_out = player_repo
        .create(NewPlayer {
            name: "Not In Campaign".to_string(),
            email: None,
            notes: None,
        })
        .unwrap();

    let mut campaign_player_repo = CampaignPlayerRepository::new(&mut conn);
    campaign_player_repo
        .add(NewCampaignPlayer {
            campaign_id,
            player_id: player_in.id,
        })
        .unwrap();

    assert!(campaign_player_repo
        .is_player_in_campaign(campaign_id, player_in.id)
        .unwrap());
    assert!(!campaign_player_repo
        .is_player_in_campaign(campaign_id, player_out.id)
        .unwrap());
}

#[test]
fn test_player_in_multiple_campaigns() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let campaign1_id = setup_campaign(&mut conn, &temp_dir);

    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign2 = campaign_repo
        .create(NewCampaign {
            name: "Second Campaign".to_string(),
            status: "active".to_string(),
            directory_path: temp_dir.path().join("c2").to_string_lossy().to_string(),
        })
        .unwrap();

    let mut player_repo = PlayerRepository::new(&mut conn);
    let player = player_repo
        .create(NewPlayer {
            name: "Multi-Campaign Player".to_string(),
            email: None,
            notes: None,
        })
        .unwrap();

    let mut campaign_player_repo = CampaignPlayerRepository::new(&mut conn);

    // Add player to both campaigns
    campaign_player_repo
        .add(NewCampaignPlayer {
            campaign_id: campaign1_id,
            player_id: player.id,
        })
        .unwrap();

    campaign_player_repo
        .add(NewCampaignPlayer {
            campaign_id: campaign2.id,
            player_id: player.id,
        })
        .unwrap();

    // Player should be in both campaigns
    assert!(campaign_player_repo
        .is_player_in_campaign(campaign1_id, player.id)
        .unwrap());
    assert!(campaign_player_repo
        .is_player_in_campaign(campaign2.id, player.id)
        .unwrap());
}
