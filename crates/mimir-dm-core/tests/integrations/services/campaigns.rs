//! Integration tests for campaign service and related functionality

use diesel::prelude::*;
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
use mimir_dm_core::dal::campaign::documents::DocumentRepository;
use mimir_dm_core::dal::campaign::template_documents::TemplateRepository;
use mimir_dm_core::models::campaign::documents::{NewDocument, UpdateDocument};
use mimir_dm_core::seed::template_seeder::seed_templates;
use mimir_dm_core::services::campaign_service::CampaignService;
use mimir_dm_core::services::template_service::TemplateService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    // Create a temporary directory for test databases
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");

    // Create connection
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();

    // Run migrations and seed templates
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_templates(&mut conn).unwrap();

    (conn, temp_dir)
}

#[test]
fn test_campaign_service_create_and_get() {
    let (mut conn, temp_dir) = setup_test_db();

    // Create campaign using the service
    let mut service = CampaignService::new(&mut conn);
    let campaign = service
        .create_campaign(
            "Test Campaign",
            Some("A test campaign".to_string()),
            temp_dir.path().to_str().unwrap(),
        )
        .unwrap();

    assert_eq!(campaign.name, "Test Campaign");
    assert_eq!(campaign.status, "concept");

    // Get campaign using repository
    let mut repo = CampaignRepository::new(&mut conn);
    let retrieved = repo.find_by_id(campaign.id).unwrap().unwrap();
    assert_eq!(retrieved.id, campaign.id);
    assert_eq!(retrieved.name, campaign.name);
}

#[test]
fn test_campaign_service_list_and_update() {
    let (mut conn, temp_dir) = setup_test_db();

    // Create multiple campaigns
    let mut service = CampaignService::new(&mut conn);
    for i in 1..=3 {
        service
            .create_campaign(
                &format!("Campaign {}", i),
                None,
                temp_dir.path().to_str().unwrap(),
            )
            .unwrap();
    }

    // List campaigns using repository
    let mut repo = CampaignRepository::new(&mut conn);
    let campaigns = repo.list().unwrap();
    assert!(campaigns.len() >= 3);

    // Update campaign status using service
    let campaign = &campaigns[0];
    let mut service = CampaignService::new(&mut conn);
    let updated = service
        .transition_campaign_stage(campaign.id, "session_zero")
        .unwrap();
    assert_eq!(updated.status, "session_zero");
}

#[test]
fn test_document_lifecycle() {
    let (mut conn, temp_dir) = setup_test_db();

    // Create campaign first
    let mut service = CampaignService::new(&mut conn);
    let campaign = service
        .create_campaign("Doc Test Campaign", None, temp_dir.path().to_str().unwrap())
        .unwrap();

    // Create document using repository
    let new_doc = NewDocument {
        campaign_id: campaign.id,
        template_id: "campaign_pitch".to_string(),
        document_type: "campaign_pitch".to_string(),
        title: "Campaign Pitch".to_string(),
        file_path: format!("{}/campaign_pitch.md", campaign.directory_path),
        module_id: None,
        session_id: None,
        file_type: "markdown".to_string(),
        is_user_created: false,
    };

    let doc = DocumentRepository::create(&mut conn, new_doc).unwrap();
    assert_eq!(doc.title, "Campaign Pitch");
    assert_eq!(doc.template_id, "campaign_pitch");

    // Get document
    let retrieved = DocumentRepository::find_by_id(&mut conn, doc.id).unwrap();
    assert_eq!(retrieved.id, doc.id);

    // Update document
    let update = UpdateDocument {
        title: Some("Updated Pitch".to_string()),
        completed_at: Some("2024-01-01T00:00:00".to_string()),
        updated_at: Some("2024-01-01T00:00:00".to_string()),
    };

    let updated = DocumentRepository::update(&mut conn, doc.id, update).unwrap();
    assert_eq!(updated.title, "Updated Pitch");
    assert!(updated.completed_at.is_some());
}

#[test]
fn test_document_list_by_campaign() {
    let (mut conn, temp_dir) = setup_test_db();

    // Create campaign
    let mut service = CampaignService::new(&mut conn);
    let campaign = service
        .create_campaign(
            "Multi Doc Campaign",
            None,
            temp_dir.path().to_str().unwrap(),
        )
        .unwrap();

    // Create multiple documents
    let templates = vec!["campaign_pitch", "starting_scenario", "world_primer"];
    for template_id in &templates {
        let new_doc = NewDocument {
            campaign_id: campaign.id,
            template_id: template_id.to_string(),
            document_type: template_id.to_string(),
            title: template_id.replace('_', " "),
            file_path: format!("{}/{}.md", campaign.directory_path, template_id),
            module_id: None,
            session_id: None,
            file_type: "markdown".to_string(),
            is_user_created: false,
        };
        DocumentRepository::create(&mut conn, new_doc).unwrap();
    }

    // List documents
    let docs = DocumentRepository::find_by_campaign(&mut conn, campaign.id).unwrap();
    assert!(docs.len() >= templates.len());
}

#[test]
fn test_template_get_and_render() {
    let (mut conn, _temp_dir) = setup_test_db();

    // Get template using repository
    let template = TemplateRepository::get_latest(&mut conn, "campaign_pitch").unwrap();
    assert_eq!(template.document_type, Some("campaign_pitch".to_string()));
    assert!(!template.document_content.is_empty());

    // Verify template service can list templates
    let mut service = TemplateService::new(&mut conn);
    let templates = service.list_templates().unwrap();
    assert!(!templates.is_empty());

    // Verify we can get a specific template
    let campaign_pitch = service.get_template("campaign_pitch").unwrap();
    assert_eq!(
        campaign_pitch.document_type,
        Some("campaign_pitch".to_string())
    );
}

#[test]
fn test_campaign_stage_progression() {
    let (mut conn, temp_dir) = setup_test_db();

    // Create campaign
    let mut service = CampaignService::new(&mut conn);
    let campaign = service
        .create_campaign(
            "Stage Test Campaign",
            None,
            temp_dir.path().to_str().unwrap(),
        )
        .unwrap();

    // Get initial status
    assert_eq!(campaign.status, "concept");

    // Create and complete required document
    let new_doc = NewDocument {
        campaign_id: campaign.id,
        template_id: "campaign_pitch".to_string(),
        document_type: "campaign_pitch".to_string(),
        title: "Campaign Pitch".to_string(),
        file_path: format!("{}/campaign_pitch.md", campaign.directory_path),
        module_id: None,
        session_id: None,
        file_type: "markdown".to_string(),
        is_user_created: false,
    };
    let doc = DocumentRepository::create(&mut conn, new_doc).unwrap();

    // Mark document as complete
    let update = UpdateDocument {
        completed_at: Some("2024-01-01T00:00:00".to_string()),
        title: None,
        updated_at: Some("2024-01-01T00:00:00".to_string()),
    };
    DocumentRepository::update(&mut conn, doc.id, update).unwrap();

    // Progress to next stage
    let mut service = CampaignService::new(&mut conn);
    let updated = service
        .transition_campaign_stage(campaign.id, "session_zero")
        .unwrap();
    assert_eq!(updated.status, "session_zero");
}

#[test]
fn test_board_configuration() {
    let (_conn, _temp_dir) = setup_test_db();

    // Get board configuration from registry
    use mimir_dm_core::domain::BoardRegistry;
    let registry = BoardRegistry::new();
    let board = registry.get("campaign").unwrap();

    assert_eq!(board.board_type(), "campaign");
    let stages = board.stages();
    assert!(!stages.is_empty());

    // Verify stages have metadata
    for stage in stages {
        let metadata = board.stage_metadata(stage);
        assert!(!metadata.display_name.is_empty());
        assert!(!metadata.description.is_empty());
    }
}

#[test]
fn test_document_check_exists() {
    let (mut conn, temp_dir) = setup_test_db();

    // Create campaign
    let mut service = CampaignService::new(&mut conn);
    let campaign = service
        .create_campaign(
            "File Check Campaign",
            None,
            temp_dir.path().to_str().unwrap(),
        )
        .unwrap();

    // Check if document exists (should not exist initially)
    let docs = DocumentRepository::find_by_campaign(&mut conn, campaign.id).unwrap();
    let initial_count = docs
        .iter()
        .filter(|d| d.template_id == "campaign_pitch")
        .count();

    // Create document
    let new_doc = NewDocument {
        campaign_id: campaign.id,
        template_id: "campaign_pitch".to_string(),
        document_type: "campaign_pitch".to_string(),
        title: "Campaign Pitch".to_string(),
        file_path: format!("{}/campaign_pitch.md", campaign.directory_path),
        module_id: None,
        session_id: None,
        file_type: "markdown".to_string(),
        is_user_created: false,
    };
    DocumentRepository::create(&mut conn, new_doc).unwrap();

    // Check again (should exist now)
    let docs = DocumentRepository::find_by_campaign(&mut conn, campaign.id).unwrap();
    let final_count = docs
        .iter()
        .filter(|d| d.template_id == "campaign_pitch")
        .count();
    assert_eq!(final_count, initial_count + 1);
}
