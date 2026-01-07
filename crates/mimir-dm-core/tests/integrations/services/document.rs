//! Integration tests for DocumentService
//!
//! Tests document lifecycle, template creation, and file operations.

use diesel::prelude::*;
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
use mimir_dm_core::models::campaign::campaigns::NewCampaign;
use mimir_dm_core::models::campaign::documents::NewDocument;
use mimir_dm_core::seed::template_seeder::seed_templates;
use mimir_dm_core::services::DocumentService;
use mimir_dm_core::{establish_connection, run_migrations};
use std::fs;
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");

    let mut conn = establish_connection(db_path.to_str().expect("Invalid path"))
        .expect("Failed to establish connection");
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_templates(&mut conn).expect("Failed to seed templates");

    (conn, temp_dir)
}

fn create_test_campaign(conn: &mut SqliteConnection, temp_dir: &TempDir) -> i32 {
    let mut repo = CampaignRepository::new(conn);
    let campaign = repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            status: "concept".to_string(),
            directory_path: temp_dir.path().to_str().expect("Invalid path").to_string(),
        })
        .expect("Failed to create test campaign");
    campaign.id
}

#[test]
fn test_document_service_create_document() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);

    let mut service = DocumentService::new(&mut conn);

    let new_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "test_doc".to_string(),
        document_type: "notes".to_string(),
        title: "Test Document".to_string(),
        file_path: temp_dir.path().join("test_doc.md").to_string_lossy().to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };

    let doc = service.create_document(new_doc).expect("Failed to create document");

    assert_eq!(doc.campaign_id, campaign_id);
    assert_eq!(doc.title, "Test Document");
    assert_eq!(doc.document_type, "notes");
    assert!(doc.completed_at.is_none());
}

#[test]
fn test_document_service_get_campaign_documents() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);

    // Create multiple documents
    let mut service = DocumentService::new(&mut conn);
    for i in 1..=3 {
        let new_doc = NewDocument {
            campaign_id,
            module_id: None,
            session_id: None,
            template_id: format!("doc_{}", i),
            document_type: "notes".to_string(),
            title: format!("Document {}", i),
            file_path: temp_dir.path().join(format!("doc_{}.md", i)).to_string_lossy().to_string(),
            file_type: "markdown".to_string(),
            is_user_created: false,
        };
        service.create_document(new_doc).expect("Failed to create document");
    }

    // Get all documents
    let docs = service
        .get_campaign_documents(campaign_id)
        .expect("Failed to get documents");

    assert_eq!(docs.len(), 3);
}

#[test]
fn test_document_service_get_documents_by_level() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);

    let mut service = DocumentService::new(&mut conn);

    // Create campaign-level document
    let campaign_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "campaign_notes".to_string(),
        document_type: "notes".to_string(),
        title: "Campaign Notes".to_string(),
        file_path: temp_dir.path().join("campaign_notes.md").to_string_lossy().to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };
    service.create_document(campaign_doc).expect("Failed to create campaign doc");

    // Create handout document
    let handout_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "player_handout".to_string(),
        document_type: "handout".to_string(),
        title: "Player Handout".to_string(),
        file_path: temp_dir.path().join("handout.md").to_string_lossy().to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };
    service.create_document(handout_doc).expect("Failed to create handout");

    // Test campaign level filter
    let campaign_docs = service
        .get_documents_by_level(campaign_id, "campaign", None, None)
        .expect("Failed to get campaign docs");
    assert_eq!(campaign_docs.len(), 1);
    assert_eq!(campaign_docs[0].title, "Campaign Notes");

    // Test handout level filter
    let handouts = service
        .get_documents_by_level(campaign_id, "handout", None, None)
        .expect("Failed to get handouts");
    assert_eq!(handouts.len(), 1);
    assert_eq!(handouts[0].title, "Player Handout");

    // Test invalid level
    let result = service.get_documents_by_level(campaign_id, "invalid", None, None);
    assert!(result.is_err());
}

#[test]
fn test_document_service_update_document() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);

    let mut service = DocumentService::new(&mut conn);

    let new_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "update_test".to_string(),
        document_type: "notes".to_string(),
        title: "Original Title".to_string(),
        file_path: temp_dir.path().join("update_test.md").to_string_lossy().to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };
    let doc = service.create_document(new_doc).expect("Failed to create document");

    // Update the document
    let update = mimir_dm_core::models::campaign::documents::UpdateDocument {
        title: Some("Updated Title".to_string()),
        updated_at: None,
        completed_at: None,
    };
    let updated = service
        .update_document(doc.id, update)
        .expect("Failed to update document");

    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.id, doc.id);
}

#[test]
fn test_document_service_complete_document() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);

    let mut service = DocumentService::new(&mut conn);

    let new_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "complete_test".to_string(),
        document_type: "notes".to_string(),
        title: "Complete Test".to_string(),
        file_path: temp_dir.path().join("complete_test.md").to_string_lossy().to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };
    let doc = service.create_document(new_doc).expect("Failed to create document");
    assert!(doc.completed_at.is_none());

    // Complete the document
    let completed = service
        .complete_document(doc.id)
        .expect("Failed to complete document");

    assert!(completed.completed_at.is_some());
}

#[test]
fn test_document_service_delete_document() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);

    let mut service = DocumentService::new(&mut conn);

    let new_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "delete_test".to_string(),
        document_type: "notes".to_string(),
        title: "Delete Test".to_string(),
        file_path: temp_dir.path().join("delete_test.md").to_string_lossy().to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };
    let doc = service.create_document(new_doc).expect("Failed to create document");

    // Delete the document
    service.delete_document(doc.id).expect("Failed to delete document");

    // Verify it's gone
    let docs = service
        .get_campaign_documents(campaign_id)
        .expect("Failed to get documents");
    assert!(docs.is_empty());
}

#[test]
fn test_document_service_incomplete_and_completed_documents() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);

    let mut service = DocumentService::new(&mut conn);

    // Create two documents
    for i in 1..=2 {
        let new_doc = NewDocument {
            campaign_id,
            module_id: None,
            session_id: None,
            template_id: format!("status_test_{}", i),
            document_type: "notes".to_string(),
            title: format!("Status Test {}", i),
            file_path: temp_dir.path().join(format!("status_{}.md", i)).to_string_lossy().to_string(),
            file_type: "markdown".to_string(),
            is_user_created: false,
        };
        service.create_document(new_doc).expect("Failed to create document");
    }

    // Initially all should be incomplete
    let incomplete = service
        .get_incomplete_documents(campaign_id)
        .expect("Failed to get incomplete docs");
    assert_eq!(incomplete.len(), 2);

    let completed = service
        .get_completed_documents(campaign_id)
        .expect("Failed to get completed docs");
    assert!(completed.is_empty());

    // Complete one document
    service
        .complete_document(incomplete[0].id)
        .expect("Failed to complete document");

    // Now should have 1 incomplete and 1 completed
    let incomplete = service
        .get_incomplete_documents(campaign_id)
        .expect("Failed to get incomplete docs");
    assert_eq!(incomplete.len(), 1);

    let completed = service
        .get_completed_documents(campaign_id)
        .expect("Failed to get completed docs");
    assert_eq!(completed.len(), 1);
}

#[test]
fn test_document_service_read_and_save_file() {
    let (mut conn, temp_dir) = setup_test_db();

    let service = DocumentService::new(&mut conn);
    let file_path = temp_dir.path().join("test_file.md");
    let file_path_str = file_path.to_string_lossy().to_string();

    // Save a file
    let content = "# Test Document\n\nThis is test content.";
    service
        .save_document_file(&file_path_str, content)
        .expect("Failed to save file");

    // Verify file exists
    assert!(file_path.exists());

    // Read the file back
    let read_content = service
        .read_document_file(&file_path_str)
        .expect("Failed to read file");

    assert_eq!(read_content, content);
}

#[test]
fn test_document_service_read_nonexistent_file() {
    let (mut conn, _temp_dir) = setup_test_db();

    let service = DocumentService::new(&mut conn);

    let result = service.read_document_file("/nonexistent/path/file.md");
    assert!(result.is_err());
}

#[test]
fn test_document_service_save_creates_parent_directories() {
    let (mut conn, temp_dir) = setup_test_db();

    let service = DocumentService::new(&mut conn);
    let nested_path = temp_dir.path().join("nested/dir/structure/test.md");
    let nested_path_str = nested_path.to_string_lossy().to_string();

    // Save should create parent directories
    service
        .save_document_file(&nested_path_str, "Test content")
        .expect("Failed to save file with nested path");

    assert!(nested_path.exists());
    assert_eq!(fs::read_to_string(&nested_path).unwrap(), "Test content");
}

#[test]
fn test_document_service_create_from_template() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);

    let mut service = DocumentService::new(&mut conn);

    // Create document from template
    let doc = service
        .create_document_from_template(campaign_id, "campaign_pitch")
        .expect("Failed to create document from template");

    assert_eq!(doc.campaign_id, campaign_id);
    assert_eq!(doc.template_id, "campaign_pitch");
    assert!(doc.title.contains("Campaign") || doc.title.contains("Pitch"));

    // Verify the file was created
    assert!(std::path::Path::new(&doc.file_path).exists());

    // Read and verify it has frontmatter
    let content = fs::read_to_string(&doc.file_path).expect("Failed to read created file");
    assert!(content.starts_with("---"));
    assert!(content.contains("title:"));
    assert!(content.contains("type:"));
}

#[test]
fn test_document_service_create_from_template_duplicate_prevention() {
    let (mut conn, temp_dir) = setup_test_db();
    let campaign_id = create_test_campaign(&mut conn, &temp_dir);

    let mut service = DocumentService::new(&mut conn);

    // Create first document from template
    service
        .create_document_from_template(campaign_id, "campaign_pitch")
        .expect("Failed to create first document");

    // Try to create duplicate - should fail
    let result = service.create_document_from_template(campaign_id, "campaign_pitch");
    assert!(result.is_err());
}
