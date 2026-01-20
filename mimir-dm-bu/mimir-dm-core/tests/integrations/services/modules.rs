//! Integration tests for module service

use mimir_dm_core::establish_connection;
use mimir_dm_core::models::campaign::modules::UpdateModule;
use mimir_dm_core::run_migrations;
use mimir_dm_core::services::CampaignService;
use mimir_dm_core::services::ModuleService;
use tempfile::TempDir;

fn setup_test_db() -> mimir_dm_core::connection::DbConnection {
    let mut conn = establish_connection(":memory:").unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");

    // Seed templates
    mimir_dm_core::seed::template_seeder::seed_templates(&mut conn).unwrap();

    conn
}

fn create_test_campaign(conn: &mut mimir_dm_core::connection::DbConnection) -> (i32, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir_path = temp_dir.path().to_string_lossy().to_string();

    let mut campaign_service = CampaignService::new(conn);
    let campaign = campaign_service
        .create_campaign(
            "Test Campaign",
            Some("Test campaign for module tests".to_string()),
            &dir_path,
        )
        .unwrap();

    // Keep temp_dir alive by leaking it - in tests this is okay
    std::mem::forget(temp_dir);

    (campaign.id, dir_path)
}

#[test]
fn test_create_module() {
    let mut conn = setup_test_db();
    let (campaign_id, _) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module(campaign_id, "Test Module".to_string(), 4)
        .unwrap();

    assert_eq!(module.name, "Test Module");
    assert_eq!(module.campaign_id, campaign_id);
    assert_eq!(module.module_number, 1);
    assert_eq!(module.status, "planning");
    assert_eq!(module.expected_sessions, 4);
}

#[test]
fn test_module_numbering() {
    let mut conn = setup_test_db();
    let (campaign_id, _) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    // Create first module
    let module1 = service
        .create_module(campaign_id, "Module 1".to_string(), 3)
        .unwrap();
    assert_eq!(module1.module_number, 1);

    // Create second module
    let module2 = service
        .create_module(campaign_id, "Module 2".to_string(), 4)
        .unwrap();
    assert_eq!(module2.module_number, 2);

    // Create third module
    let module3 = service
        .create_module(campaign_id, "Module 3".to_string(), 5)
        .unwrap();
    assert_eq!(module3.module_number, 3);
}

#[test]
fn test_get_module() {
    let mut conn = setup_test_db();
    let (campaign_id, _) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    let created = service
        .create_module(campaign_id, "Test Module".to_string(), 4)
        .unwrap();

    let fetched = service.get_module(created.id).unwrap().unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "Test Module");
}

#[test]
fn test_list_campaign_modules() {
    let mut conn = setup_test_db();
    let (campaign_id, _) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    // Create multiple modules
    service
        .create_module(campaign_id, "Module 1".to_string(), 3)
        .unwrap();
    service
        .create_module(campaign_id, "Module 2".to_string(), 4)
        .unwrap();
    service
        .create_module(campaign_id, "Module 3".to_string(), 5)
        .unwrap();

    let modules = service.list_campaign_modules(campaign_id).unwrap();
    assert_eq!(modules.len(), 3);

    // Should be ordered by module number
    assert_eq!(modules[0].name, "Module 1");
    assert_eq!(modules[1].name, "Module 2");
    assert_eq!(modules[2].name, "Module 3");
}

#[test]
fn test_transition_module_stage() {
    let mut conn = setup_test_db();
    let (campaign_id, _) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module(campaign_id, "Test Module".to_string(), 4)
        .unwrap();

    // Valid transition: planning -> development
    let updated = service
        .transition_module_stage(module.id, "development")
        .unwrap();
    assert_eq!(updated.status, "development");

    // Valid transition: development -> ready
    let updated = service.transition_module_stage(module.id, "ready").unwrap();
    assert_eq!(updated.status, "ready");

    // Valid transition: ready -> active
    let updated = service
        .transition_module_stage(module.id, "active")
        .unwrap();
    assert_eq!(updated.status, "active");
    assert!(updated.started_at.is_some());

    // Valid transition: active -> completed
    let updated = service
        .transition_module_stage(module.id, "completed")
        .unwrap();
    assert_eq!(updated.status, "completed");
    assert!(updated.completed_at.is_some());
}

#[test]
fn test_invalid_transition() {
    let mut conn = setup_test_db();
    let (campaign_id, _) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module(campaign_id, "Test Module".to_string(), 4)
        .unwrap();

    // Invalid transition: planning -> ready (skipping stages)
    let result = service.transition_module_stage(module.id, "ready");
    assert!(result.is_err());
}

#[test]
fn test_backward_transitions() {
    let mut conn = setup_test_db();
    let (campaign_id, _) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module(campaign_id, "Test Module".to_string(), 4)
        .unwrap();

    // Move to development
    service
        .transition_module_stage(module.id, "development")
        .unwrap();

    // Can move back to planning
    let updated = service
        .transition_module_stage(module.id, "planning")
        .unwrap();
    assert_eq!(updated.status, "planning");
}

#[test]
fn test_update_module() {
    let mut conn = setup_test_db();
    let (campaign_id, _) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module(campaign_id, "Original Name".to_string(), 4)
        .unwrap();

    let update = UpdateModule {
        name: Some("Updated Name".to_string()),
        expected_sessions: Some(6),
        ..Default::default()
    };

    let updated = service.update_module(module.id, update).unwrap();
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.expected_sessions, 6);
}

#[test]
fn test_initialize_module_documents() {
    let mut conn = setup_test_db();
    let (campaign_id, dir_path) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module(campaign_id, "Test Module".to_string(), 4)
        .unwrap();

    // Module starts in planning, so already has required documents

    // Initialize documents
    let created_files = service
        .initialize_module_documents(module.id, &dir_path)
        .unwrap();

    // Should create module_overview for planning stage
    assert!(created_files.contains(&"module-overview.md".to_string()));

    // Verify files exist on disk
    let module_dir = std::path::PathBuf::from(&dir_path)
        .join("modules")
        .join(format!("module_{:02}", module.module_number));
    assert!(module_dir.join("module-overview.md").exists());

    // Verify documents in database
    let documents = service.get_module_documents(module.id).unwrap();
    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0].template_id, "module_overview");
}

#[test]
fn test_check_module_completion() {
    let mut conn = setup_test_db();
    let (campaign_id, dir_path) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module(campaign_id, "Test Module".to_string(), 4)
        .unwrap();

    // Module starts in planning state by default

    // Check completion before documents
    let status = service.check_module_completion(module.id).unwrap();
    assert_eq!(status.current_stage, "planning");
    assert_eq!(status.total_required_documents, 1); // module_overview
    assert_eq!(status.completed_required_documents, 0);
    assert!(!status.is_stage_complete);
    assert!(!status.can_progress);

    // Initialize documents
    service
        .initialize_module_documents(module.id, &dir_path)
        .unwrap();

    // Mark document as complete
    let documents = service.get_module_documents(module.id).unwrap();
    let doc_id = documents[0].id;

    // Release service to allow direct connection access
    let _ = service;

    mimir_dm_core::dal::campaign::documents::DocumentRepository::update(
        &mut conn,
        doc_id,
        mimir_dm_core::models::campaign::documents::UpdateDocument {
            title: None,
            updated_at: None,
            completed_at: Some(chrono::Utc::now().to_rfc3339()),
        },
    )
    .unwrap();

    // Recreate service to check completion
    let mut service = ModuleService::new(&mut conn);
    let status = service.check_module_completion(module.id).unwrap();
    assert_eq!(status.completed_required_documents, 1);
    assert!(status.is_stage_complete);
    assert!(status.can_progress);
    assert_eq!(status.next_stage, Some("development".to_string()));
}

#[test]
fn test_increment_module_sessions() {
    let mut conn = setup_test_db();
    let (campaign_id, _) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module(campaign_id, "Test Module".to_string(), 4)
        .unwrap();

    assert_eq!(module.actual_sessions, 0);

    // Increment sessions
    let updated = service.increment_module_sessions(module.id).unwrap();
    assert_eq!(updated.actual_sessions, 1);
    assert!(updated.started_at.is_some()); // Should auto-start

    // Increment again
    let updated = service.increment_module_sessions(module.id).unwrap();
    assert_eq!(updated.actual_sessions, 2);
}

#[test]
fn test_find_modules_needing_next() {
    let mut conn = setup_test_db();
    let (campaign_id, _) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    // Create an active module
    let module = service
        .create_module(
            campaign_id,
            "Active Module".to_string(),
            5, // Expected 5 sessions
        )
        .unwrap();

    // Module starts in planning state, transition to active
    service
        .transition_module_stage(module.id, "development")
        .unwrap();
    service.transition_module_stage(module.id, "ready").unwrap();
    service
        .transition_module_stage(module.id, "active")
        .unwrap();

    // Initially no modules need next
    let needing_next = service.find_modules_needing_next(campaign_id).unwrap();
    assert_eq!(needing_next.len(), 0);

    // Increment sessions to 60% (3 out of 5)
    service.increment_module_sessions(module.id).unwrap();
    service.increment_module_sessions(module.id).unwrap();
    service.increment_module_sessions(module.id).unwrap();

    // Now should trigger next module planning
    let needing_next = service.find_modules_needing_next(campaign_id).unwrap();
    assert_eq!(needing_next.len(), 1);
    assert_eq!(needing_next[0].id, module.id);
}

#[test]
fn test_delete_module() {
    let mut conn = setup_test_db();
    let (campaign_id, _) = create_test_campaign(&mut conn);

    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module(campaign_id, "To Delete".to_string(), 3)
        .unwrap();

    // Verify it exists
    assert!(service.get_module(module.id).unwrap().is_some());

    // Delete it
    service.delete_module(module.id).unwrap();

    // Verify it's gone
    assert!(service.get_module(module.id).unwrap().is_none());
}
