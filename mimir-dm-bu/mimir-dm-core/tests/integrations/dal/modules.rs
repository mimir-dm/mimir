//! Integration tests for module DAL

use crate::common::TestDatabase;
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
use mimir_dm_core::dal::campaign::modules::ModuleRepository;
use mimir_dm_core::models::campaign::campaigns::NewCampaign;
use mimir_dm_core::models::campaign::modules::NewModule;
use tempfile::TempDir;

#[test]
fn test_module_lifecycle() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create a campaign first
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: temp_dir.path().to_string_lossy().to_string(),
        })
        .unwrap();

    let mut module_repo = ModuleRepository::new(&mut conn);

    // Create a new module
    let new_module = NewModule {
        campaign_id: campaign.id,
        name: "The Brittle Steel Mystery".to_string(),
        module_number: 1,
        status: "planning".to_string(),
        expected_sessions: 4,
    };

    let module = module_repo.create(new_module).unwrap();
    assert_eq!(module.name, "The Brittle Steel Mystery");
    assert_eq!(module.status, "planning");
    assert_eq!(module.expected_sessions, 4);
    assert_eq!(module.actual_sessions, 0);

    // Test status transitions
    let updated = module_repo
        .transition_status(module.id, "development")
        .unwrap();
    assert_eq!(updated.status, "development");

    let updated = module_repo.transition_status(module.id, "ready").unwrap();
    assert_eq!(updated.status, "ready");

    let updated = module_repo.transition_status(module.id, "active").unwrap();
    assert_eq!(updated.status, "active");
    assert!(updated.started_at.is_some());

    let updated = module_repo
        .transition_status(module.id, "completed")
        .unwrap();
    assert_eq!(updated.status, "completed");
    assert!(updated.completed_at.is_some());
}

#[test]
fn test_module_session_tracking() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create a campaign
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: temp_dir.path().to_string_lossy().to_string(),
        })
        .unwrap();

    let mut module_repo = ModuleRepository::new(&mut conn);

    // Create a module
    let module = module_repo
        .create(NewModule {
            campaign_id: campaign.id,
            name: "Test Module".to_string(),
            module_number: 1,
            status: "active".to_string(),
            expected_sessions: 5,
        })
        .unwrap();

    // Increment sessions
    let updated = module_repo.increment_sessions(module.id).unwrap();
    assert_eq!(updated.actual_sessions, 1);
    assert!(updated.started_at.is_some()); // Auto-started

    module_repo.increment_sessions(module.id).unwrap();
    let updated = module_repo.increment_sessions(module.id).unwrap();
    assert_eq!(updated.actual_sessions, 3);

    // Check if should trigger next module (60% = 3 sessions)
    let modules_needing_next = module_repo.find_modules_needing_next(campaign.id).unwrap();
    assert_eq!(modules_needing_next.len(), 1);
    assert_eq!(modules_needing_next[0].id, module.id);
}

#[test]
fn test_module_numbering() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create a campaign
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: temp_dir.path().to_string_lossy().to_string(),
        })
        .unwrap();

    let mut module_repo = ModuleRepository::new(&mut conn);

    // Get next module number for empty campaign
    let next_num = module_repo.get_next_module_number(campaign.id).unwrap();
    assert_eq!(next_num, 1);

    // Create some modules
    module_repo
        .create(NewModule {
            campaign_id: campaign.id,
            name: "Module 1".to_string(),
            module_number: 1,
            status: "completed".to_string(),
            expected_sessions: 3,
        })
        .unwrap();

    module_repo
        .create(NewModule {
            campaign_id: campaign.id,
            name: "Module 2".to_string(),
            module_number: 2,
            status: "active".to_string(),
            expected_sessions: 4,
        })
        .unwrap();

    // Get next module number
    let next_num = module_repo.get_next_module_number(campaign.id).unwrap();
    assert_eq!(next_num, 3);
}
