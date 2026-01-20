//! Campaign command integration tests.
//!
//! Tests for campaign CRUD operations including creating, listing,
//! archiving, and managing campaigns through the service layer.
//!
//! These tests exercise the same code paths as the Tauri commands
//! by using the services with the test database.

use super::common::TestEnv;
use mimir_dm_core::services::CampaignService;

#[tokio::test]
async fn test_create_campaign() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CampaignService::new(&mut conn);
    let campaign = service
        .create_campaign(
            "Test Campaign",
            Some("A campaign for testing".to_string()),
            env.temp_dir.path().to_str().unwrap(),
        )
        .expect("Failed to create campaign");

    assert_eq!(campaign.name, "Test Campaign");
    assert_eq!(campaign.status, "concept"); // New campaigns start in "concept" status
    assert!(campaign.directory_path.contains("test-campaign")); // kebab-case directory name
}

#[tokio::test]
async fn test_list_active_campaigns() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CampaignService::new(&mut conn);

    // Initially no campaigns
    let campaigns = service.list_active_campaigns().expect("Failed to list campaigns");
    assert!(campaigns.is_empty());

    // Create a campaign
    service
        .create_campaign(
            "Campaign 1",
            None,
            env.temp_dir.path().to_str().unwrap(),
        )
        .expect("Failed to create campaign");

    // Should have one campaign
    let campaigns = service.list_active_campaigns().expect("Failed to list campaigns");
    assert_eq!(campaigns.len(), 1);
    assert_eq!(campaigns[0].name, "Campaign 1");
}

#[tokio::test]
async fn test_get_campaign_by_id() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CampaignService::new(&mut conn);

    let created = service
        .create_campaign(
            "Find Me Campaign",
            None,
            env.temp_dir.path().to_str().unwrap(),
        )
        .expect("Failed to create campaign");

    let found = service
        .get_campaign(created.id)
        .expect("Failed to get campaign")
        .expect("Campaign should exist");

    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "Find Me Campaign");
}

#[tokio::test]
async fn test_get_nonexistent_campaign() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CampaignService::new(&mut conn);

    let result = service.get_campaign(99999).expect("Query should succeed");
    assert!(result.is_none(), "Campaign should not exist");
}

#[tokio::test]
async fn test_archive_campaign() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CampaignService::new(&mut conn);

    let campaign = service
        .create_campaign(
            "To Archive",
            None,
            env.temp_dir.path().to_str().unwrap(),
        )
        .expect("Failed to create campaign");

    let archived = service
        .archive_campaign(campaign.id)
        .expect("Failed to archive campaign");

    // Archive sets archived_at timestamp but preserves original status
    assert!(archived.archived_at.is_some(), "Archived campaign should have archived_at timestamp");

    // Should not appear in active list
    let active = service.list_active_campaigns().expect("Failed to list campaigns");
    assert!(active.is_empty(), "Archived campaign should not be in active list");
}

#[tokio::test]
async fn test_unarchive_campaign() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CampaignService::new(&mut conn);

    // Create and archive
    let campaign = service
        .create_campaign(
            "Unarchive Me",
            None,
            env.temp_dir.path().to_str().unwrap(),
        )
        .expect("Failed to create campaign");

    let original_status = campaign.status.clone();
    service.archive_campaign(campaign.id).expect("Failed to archive");

    // Unarchive
    let unarchived = service
        .unarchive_campaign(campaign.id)
        .expect("Failed to unarchive campaign");

    // Unarchive restores the original status (not necessarily "active")
    assert_eq!(unarchived.status, original_status);

    // Should appear in active list (non-archived campaigns)
    let active = service.list_active_campaigns().expect("Failed to list campaigns");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].name, "Unarchive Me");
}

#[tokio::test]
async fn test_list_archived_campaigns() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CampaignService::new(&mut conn);

    // Create two campaigns
    let campaign1 = service
        .create_campaign("Active One", None, env.temp_dir.path().to_str().unwrap())
        .expect("Failed to create campaign 1");

    let campaign2 = service
        .create_campaign("Archived One", None, env.temp_dir.path().to_str().unwrap())
        .expect("Failed to create campaign 2");

    // Archive one
    service.archive_campaign(campaign2.id).expect("Failed to archive");

    // Check lists
    let active = service.list_active_campaigns().expect("Failed to list active");
    let archived = service.list_archived_campaigns().expect("Failed to list archived");

    assert_eq!(active.len(), 1);
    assert_eq!(active[0].name, "Active One");

    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].name, "Archived One");
}

#[tokio::test]
async fn test_delete_archived_campaign() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CampaignService::new(&mut conn);

    // Create and archive
    let campaign = service
        .create_campaign(
            "Delete Me",
            None,
            env.temp_dir.path().to_str().unwrap(),
        )
        .expect("Failed to create campaign");

    service.archive_campaign(campaign.id).expect("Failed to archive");

    // Delete (without deleting files)
    service
        .delete_campaign(campaign.id, false)
        .expect("Failed to delete campaign");

    // Should be gone
    let result = service.get_campaign(campaign.id).expect("Query should succeed");
    assert!(result.is_none(), "Campaign should be deleted");
}

#[tokio::test]
async fn test_delete_active_campaign_fails() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CampaignService::new(&mut conn);

    let campaign = service
        .create_campaign(
            "Active Campaign",
            None,
            env.temp_dir.path().to_str().unwrap(),
        )
        .expect("Failed to create campaign");

    // Deleting active campaign should fail
    let result = service.delete_campaign(campaign.id, false);
    assert!(result.is_err(), "Should not be able to delete active campaign");
}

#[tokio::test]
async fn test_multiple_campaigns() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CampaignService::new(&mut conn);

    // Create multiple campaigns
    for i in 1..=5 {
        service
            .create_campaign(
                &format!("Campaign {}", i),
                None,
                env.temp_dir.path().to_str().unwrap(),
            )
            .expect("Failed to create campaign");
    }

    let campaigns = service.list_active_campaigns().expect("Failed to list campaigns");
    assert_eq!(campaigns.len(), 5);
}

#[tokio::test]
async fn test_campaign_directory_creation() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = CampaignService::new(&mut conn);

    let campaign = service
        .create_campaign(
            "Directory Test",
            None,
            env.temp_dir.path().to_str().unwrap(),
        )
        .expect("Failed to create campaign");

    // Check that directory was created
    let campaign_dir = std::path::Path::new(&campaign.directory_path);
    assert!(campaign_dir.exists(), "Campaign directory should be created");
    assert!(campaign_dir.is_dir(), "Campaign path should be a directory");
}
