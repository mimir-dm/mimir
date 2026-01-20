//! Integration tests for PlayerService

use diesel::prelude::*;
use mimir_dm_core::services::PlayerService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    (conn, temp_dir)
}

#[test]
fn test_create_player() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = PlayerService::new(&mut conn);

    let player = service
        .create_player(
            "Alice",
            Some("alice@example.com".to_string()),
            Some("Test player".to_string()),
        )
        .unwrap();

    assert_eq!(player.name, "Alice");
    assert_eq!(player.email, Some("alice@example.com".to_string()));
    assert_eq!(player.notes, Some("Test player".to_string()));
}

#[test]
fn test_create_player_empty_name() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = PlayerService::new(&mut conn);

    let result = service.create_player("", None, None);
    assert!(result.is_err());
}

#[test]
fn test_get_player() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = PlayerService::new(&mut conn);

    let created = service.create_player("Bob", None, None).unwrap();
    let fetched = service.get_player(created.id).unwrap();

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "Bob");
}

#[test]
fn test_get_nonexistent_player() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = PlayerService::new(&mut conn);

    let result = service.get_player(999);
    assert!(result.is_err());
}

#[test]
fn test_update_player() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = PlayerService::new(&mut conn);

    let player = service.create_player("Charlie", None, None).unwrap();
    let updated = service
        .update_player(player.id, Some("Charles".to_string()), None, None)
        .unwrap();

    assert_eq!(updated.name, "Charles");
}

#[test]
fn test_update_player_email_and_notes() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = PlayerService::new(&mut conn);

    let player = service.create_player("Diana", None, None).unwrap();
    let updated = service
        .update_player(
            player.id,
            None,
            Some(Some("diana@example.com".to_string())),
            Some(Some("Updated notes".to_string())),
        )
        .unwrap();

    assert_eq!(updated.name, "Diana");
    assert_eq!(updated.email, Some("diana@example.com".to_string()));
    assert_eq!(updated.notes, Some("Updated notes".to_string()));
}

#[test]
fn test_delete_player() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = PlayerService::new(&mut conn);

    let player = service.create_player("Eve", None, None).unwrap();
    service.delete_player(player.id).unwrap();

    let result = service.get_player(player.id);
    assert!(result.is_err());
}

#[test]
fn test_list_players() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = PlayerService::new(&mut conn);

    service.create_player("Frank", None, None).unwrap();
    service.create_player("Grace", None, None).unwrap();
    service.create_player("Hank", None, None).unwrap();

    let players = service.list_players().unwrap();
    assert_eq!(players.len(), 3);
}

#[test]
fn test_add_player_to_campaign() {
    let (mut conn, _temp_dir) = setup_test_db();

    // Create a campaign first
    use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
    use mimir_dm_core::models::campaign::NewCampaign;

    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            directory_path: "/test".to_string(),
            status: "concept".to_string(),
        })
        .unwrap();

    let mut player_service = PlayerService::new(&mut conn);
    let player = player_service.create_player("Ivan", None, None).unwrap();

    player_service
        .add_player_to_campaign(campaign.id, player.id)
        .unwrap();

    let players = player_service
        .list_players_for_campaign(campaign.id)
        .unwrap();
    assert_eq!(players.len(), 1);
    assert_eq!(players[0].id, player.id);
}

#[test]
fn test_add_player_to_campaign_duplicate() {
    let (mut conn, _temp_dir) = setup_test_db();

    // Create campaign
    use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
    use mimir_dm_core::models::campaign::NewCampaign;

    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            directory_path: "/test".to_string(),
            status: "concept".to_string(),
        })
        .unwrap();

    let mut player_service = PlayerService::new(&mut conn);
    let player = player_service.create_player("Jane", None, None).unwrap();

    player_service
        .add_player_to_campaign(campaign.id, player.id)
        .unwrap();

    // Try to add again - should fail
    let result = player_service.add_player_to_campaign(campaign.id, player.id);
    assert!(result.is_err());
}

#[test]
fn test_remove_player_from_campaign() {
    let (mut conn, _temp_dir) = setup_test_db();

    // Create campaign
    use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
    use mimir_dm_core::models::campaign::NewCampaign;

    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            directory_path: "/test".to_string(),
            status: "concept".to_string(),
        })
        .unwrap();

    let mut player_service = PlayerService::new(&mut conn);
    let player = player_service.create_player("Karl", None, None).unwrap();

    player_service
        .add_player_to_campaign(campaign.id, player.id)
        .unwrap();
    player_service
        .remove_player_from_campaign(campaign.id, player.id)
        .unwrap();

    let players = player_service
        .list_players_for_campaign(campaign.id)
        .unwrap();
    assert_eq!(players.len(), 0);
}

#[test]
fn test_set_player_active_status() {
    let (mut conn, _temp_dir) = setup_test_db();

    // Create campaign
    use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
    use mimir_dm_core::models::campaign::NewCampaign;

    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            directory_path: "/test".to_string(),
            status: "concept".to_string(),
        })
        .unwrap();

    let mut player_service = PlayerService::new(&mut conn);
    let player = player_service.create_player("Laura", None, None).unwrap();

    player_service
        .add_player_to_campaign(campaign.id, player.id)
        .unwrap();

    // Initially active
    let active_players = player_service
        .list_active_players_for_campaign(campaign.id)
        .unwrap();
    assert_eq!(active_players.len(), 1);

    // Set inactive
    player_service
        .set_player_active_status(campaign.id, player.id, false)
        .unwrap();

    let active_players = player_service
        .list_active_players_for_campaign(campaign.id)
        .unwrap();
    assert_eq!(active_players.len(), 0);

    // Set active again
    player_service
        .set_player_active_status(campaign.id, player.id, true)
        .unwrap();

    let active_players = player_service
        .list_active_players_for_campaign(campaign.id)
        .unwrap();
    assert_eq!(active_players.len(), 1);
}
