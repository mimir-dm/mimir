//! Integration tests for campaign DAL

use crate::common::TestDatabase;
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
use mimir_dm_core::models::campaign::campaigns::NewCampaign;
use tempfile::TempDir;

#[test]
fn test_campaign_lifecycle() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = CampaignRepository::new(&mut conn);

    // Create a temporary directory for the campaign
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create a new campaign
    let new_campaign = NewCampaign {
        name: "The Imprisoned Corruption".to_string(),
        status: "concept".to_string(),
        directory_path: temp_dir.path().to_string_lossy().to_string(),
    };

    let campaign = repo.create(new_campaign).unwrap();
    assert_eq!(campaign.name, "The Imprisoned Corruption");
    assert_eq!(campaign.status, "concept");

    // Test status transitions through workflow stages
    let updated = repo.transition_status(campaign.id, "session_zero").unwrap();
    assert_eq!(updated.status, "session_zero");

    let updated = repo.transition_status(campaign.id, "integration").unwrap();
    assert_eq!(updated.status, "integration");

    let updated = repo.transition_status(campaign.id, "active").unwrap();
    assert_eq!(updated.status, "active");

    let updated = repo.transition_status(campaign.id, "concluding").unwrap();
    assert_eq!(updated.status, "concluding");
}

#[test]
fn test_invalid_campaign_transitions() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = CampaignRepository::new(&mut conn);

    // Create a temporary directory for the campaign
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create a new campaign
    let new_campaign = NewCampaign {
        name: "Test Campaign".to_string(),
        status: "concept".to_string(),
        directory_path: temp_dir.path().to_string_lossy().to_string(),
    };

    let campaign = repo.create(new_campaign).unwrap();

    // DAL layer doesn't validate transitions - that's the service layer's job
    // This test validates that the DAL allows any status update

    // DAL allows direct status updates (service layer validates transitions)
    let updated = repo.transition_status(campaign.id, "active").unwrap();
    assert_eq!(updated.status, "active");

    // DAL also allows going backwards (service layer would prevent this)
    let updated = repo.transition_status(campaign.id, "concept").unwrap();
    assert_eq!(updated.status, "concept");
}

#[test]
fn test_list_active_campaigns() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = CampaignRepository::new(&mut conn);

    // Create temporary directories for campaigns
    let temp_dir1 = TempDir::new().expect("Failed to create temp directory");
    let temp_dir2 = TempDir::new().expect("Failed to create temp directory");
    let temp_dir3 = TempDir::new().expect("Failed to create temp directory");

    // Create multiple campaigns
    repo.create(NewCampaign {
        name: "Campaign 1".to_string(),
        status: "concept".to_string(),
        directory_path: temp_dir1.path().to_string_lossy().to_string(),
    })
    .unwrap();

    let campaign2 = repo
        .create(NewCampaign {
            name: "Campaign 2".to_string(),
            status: "concept".to_string(),
            directory_path: temp_dir2.path().to_string_lossy().to_string(),
        })
        .unwrap();

    repo.create(NewCampaign {
        name: "Campaign 3".to_string(),
        status: "concluding".to_string(),
        directory_path: temp_dir3.path().to_string_lossy().to_string(),
    })
    .unwrap();

    // Make campaign 2 active through valid stages
    repo.transition_status(campaign2.id, "session_zero")
        .unwrap();
    repo.transition_status(campaign2.id, "integration").unwrap();
    repo.transition_status(campaign2.id, "active").unwrap();

    // List active campaigns (list_active returns all non-archived campaigns)
    let active = repo.list_active().unwrap();
    assert_eq!(active.len(), 3); // All three campaigns are non-archived
    assert!(active
        .iter()
        .any(|c| c.name == "Campaign 1" && c.status == "concept"));
    assert!(active
        .iter()
        .any(|c| c.name == "Campaign 2" && c.status == "active"));
    assert!(active
        .iter()
        .any(|c| c.name == "Campaign 3" && c.status == "concluding"));
}
