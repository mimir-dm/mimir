//! Module command integration tests.
//!
//! Tests for module CRUD operations including creating, listing,
//! updating, and stage transitions through the service layer.

use super::common::TestEnv;
use mimir_dm_core::services::{CampaignService, ModuleService};

/// Helper to create a test campaign and return its ID
async fn setup_campaign(env: &TestEnv) -> i32 {
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = CampaignService::new(&mut conn);

    service
        .create_campaign(
            "Test Campaign",
            None,
            env.temp_dir.path().to_str().unwrap(),
        )
        .expect("Failed to create campaign")
        .id
}

#[tokio::test]
async fn test_create_module() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let campaign_id = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module_with_documents(campaign_id, "Lost Mines".to_string(), 5, None)
        .expect("Failed to create module");

    assert_eq!(module.name, "Lost Mines");
    assert_eq!(module.campaign_id, campaign_id);
    assert_eq!(module.expected_sessions, 5);
    assert_eq!(module.actual_sessions, 0);
    assert_eq!(module.status, "planning");
}

#[tokio::test]
async fn test_create_module_with_type() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let campaign_id = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module_with_documents(
            campaign_id,
            "Dungeon Crawl".to_string(),
            3,
            Some("dungeon".to_string()),
        )
        .expect("Failed to create module");

    assert_eq!(module.name, "Dungeon Crawl");
    // Module type affects which documents are created
}

#[tokio::test]
async fn test_get_module_by_id() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let campaign_id = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    let created = service
        .create_module_with_documents(campaign_id, "Find Me".to_string(), 4, None)
        .expect("Failed to create module");

    let found = service
        .get_module(created.id)
        .expect("Failed to get module")
        .expect("Module should exist");

    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "Find Me");
}

#[tokio::test]
async fn test_get_nonexistent_module() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");

    let mut service = ModuleService::new(&mut conn);

    let result = service.get_module(99999).expect("Query should succeed");
    assert!(result.is_none(), "Module should not exist");
}

#[tokio::test]
async fn test_list_campaign_modules() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let campaign_id = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    // Create multiple modules
    for i in 1..=3 {
        service
            .create_module_with_documents(
                campaign_id,
                format!("Module {}", i),
                i + 2,
                None,
            )
            .expect("Failed to create module");
    }

    let modules = service
        .list_campaign_modules(campaign_id)
        .expect("Failed to list modules");

    assert_eq!(modules.len(), 3);
}

#[tokio::test]
async fn test_update_module() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let campaign_id = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module_with_documents(campaign_id, "Original Name".to_string(), 3, None)
        .expect("Failed to create module");

    let update = mimir_dm_core::models::campaign::modules::UpdateModule {
        name: Some("Updated Name".to_string()),
        status: None,
        expected_sessions: Some(5),
        actual_sessions: None,
        started_at: None,
        completed_at: None,
    };

    let updated = service
        .update_module(module.id, update)
        .expect("Failed to update module");

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.expected_sessions, 5);
}

#[tokio::test]
async fn test_increment_module_sessions() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let campaign_id = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module_with_documents(campaign_id, "Session Counter".to_string(), 5, None)
        .expect("Failed to create module");

    assert_eq!(module.actual_sessions, 0);

    // Increment sessions
    let updated = service
        .increment_module_sessions(module.id)
        .expect("Failed to increment sessions");
    assert_eq!(updated.actual_sessions, 1);

    // Increment again
    let updated = service
        .increment_module_sessions(module.id)
        .expect("Failed to increment sessions");
    assert_eq!(updated.actual_sessions, 2);
}

#[tokio::test]
async fn test_delete_module() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let campaign_id = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module_with_documents(campaign_id, "Delete Me".to_string(), 2, None)
        .expect("Failed to create module");

    service.delete_module(module.id).expect("Failed to delete module");

    let result = service.get_module(module.id).expect("Query should succeed");
    assert!(result.is_none(), "Module should be deleted");
}

#[tokio::test]
async fn test_transition_module_stage_planning_to_development() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let campaign_id = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module_with_documents(campaign_id, "Stage Test".to_string(), 3, None)
        .expect("Failed to create module");

    assert_eq!(module.status, "planning");

    // Valid transition: planning -> development
    let transitioned = service
        .transition_module_stage(module.id, "development")
        .expect("Failed to transition stage");

    assert_eq!(transitioned.status, "development");
}

#[tokio::test]
async fn test_transition_module_through_all_stages() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let campaign_id = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module_with_documents(campaign_id, "Complete Me".to_string(), 2, None)
        .expect("Failed to create module");

    assert_eq!(module.status, "planning");

    // Transition through all stages in order:
    // planning -> development -> ready -> active -> completed

    let dev = service
        .transition_module_stage(module.id, "development")
        .expect("Failed to transition to development");
    assert_eq!(dev.status, "development");

    let ready = service
        .transition_module_stage(module.id, "ready")
        .expect("Failed to transition to ready");
    assert_eq!(ready.status, "ready");

    let active = service
        .transition_module_stage(module.id, "active")
        .expect("Failed to transition to active");
    assert_eq!(active.status, "active");
    assert!(active.started_at.is_some(), "Started timestamp should be set");

    let completed = service
        .transition_module_stage(module.id, "completed")
        .expect("Failed to transition to completed");
    assert_eq!(completed.status, "completed");
    assert!(completed.completed_at.is_some(), "Completed timestamp should be set");
}

#[tokio::test]
async fn test_get_module_documents() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let campaign_id = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module_with_documents(campaign_id, "Docs Test".to_string(), 3, None)
        .expect("Failed to create module");

    let documents = service
        .get_module_documents(module.id)
        .expect("Failed to get module documents");

    // Module creation should have created some default documents
    assert!(!documents.is_empty(), "Module should have documents");
}

#[tokio::test]
async fn test_check_module_completion() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let campaign_id = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    let module = service
        .create_module_with_documents(campaign_id, "Completion Check".to_string(), 3, None)
        .expect("Failed to create module");

    let status = service
        .check_module_completion(module.id)
        .expect("Failed to check completion");

    // Initially, no documents should be completed
    assert_eq!(status.completed_required_documents, 0);
    assert!(status.total_required_documents >= 0);
}

#[tokio::test]
async fn test_modules_isolated_to_campaigns() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let campaign1_id = setup_campaign(&env).await;

    // Create a second campaign
    let campaign2_id = {
        let mut conn = env.state.db.get_connection().expect("Failed to get connection");
        let mut service = CampaignService::new(&mut conn);
        service
            .create_campaign(
                "Campaign 2",
                None,
                env.temp_dir.path().to_str().unwrap(),
            )
            .expect("Failed to create campaign 2")
            .id
    };

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    // Create modules in different campaigns
    service
        .create_module_with_documents(campaign1_id, "Campaign 1 Module".to_string(), 3, None)
        .expect("Failed to create module");

    service
        .create_module_with_documents(campaign2_id, "Campaign 2 Module".to_string(), 2, None)
        .expect("Failed to create module");

    // Each campaign should only have its own modules
    let c1_modules = service
        .list_campaign_modules(campaign1_id)
        .expect("Failed to list campaign 1 modules");
    let c2_modules = service
        .list_campaign_modules(campaign2_id)
        .expect("Failed to list campaign 2 modules");

    assert_eq!(c1_modules.len(), 1);
    assert_eq!(c1_modules[0].name, "Campaign 1 Module");

    assert_eq!(c2_modules.len(), 1);
    assert_eq!(c2_modules[0].name, "Campaign 2 Module");
}
