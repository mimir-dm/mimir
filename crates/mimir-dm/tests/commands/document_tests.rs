//! Document command integration tests.
//!
//! Tests for document CRUD operations including creating, reading,
//! updating, and managing documents through the service layer.

use super::common::TestEnv;
use mimir_dm_core::models::campaign::documents::{NewDocument, UpdateDocument};
use mimir_dm_core::services::{CampaignService, DocumentService, ModuleService};

/// Helper to create a test campaign and return its ID and directory path
async fn setup_campaign(env: &TestEnv) -> (i32, String) {
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = CampaignService::new(&mut conn);

    let campaign = service
        .create_campaign(
            "Test Campaign",
            None,
            env.temp_dir.path().to_str().unwrap(),
        )
        .expect("Failed to create campaign");

    (campaign.id, campaign.directory_path)
}

/// Helper to create a test module and return its ID
async fn setup_module(env: &TestEnv, campaign_id: i32) -> i32 {
    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = ModuleService::new(&mut conn);

    service
        .create_module_with_documents(campaign_id, "Test Module".to_string(), 3, None)
        .expect("Failed to create module")
        .id
}

#[tokio::test]
async fn test_create_document() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, directory_path) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = DocumentService::new(&mut conn);

    let new_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "custom".to_string(),
        document_type: "notes".to_string(),
        title: "Test Document".to_string(),
        file_path: format!("{}/test-doc.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: true,
    };

    let document = service
        .create_document(new_doc)
        .expect("Failed to create document");

    assert_eq!(document.title, "Test Document");
    assert_eq!(document.document_type, "notes");
    assert!(!document.is_completed());
}

#[tokio::test]
async fn test_get_campaign_documents() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, directory_path) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = DocumentService::new(&mut conn);

    // Create some documents
    for i in 1..=3 {
        let new_doc = NewDocument {
            campaign_id,
            module_id: None,
            session_id: None,
            template_id: "custom".to_string(),
            document_type: "notes".to_string(),
            title: format!("Document {}", i),
            file_path: format!("{}/doc-{}.md", directory_path, i),
            file_type: "markdown".to_string(),
            is_user_created: true,
        };
        service.create_document(new_doc).expect("Failed to create document");
    }

    let documents = service
        .get_campaign_documents(campaign_id)
        .expect("Failed to get documents");

    // Campaign creation also creates some initial documents, so we check our 3 are included
    assert!(documents.len() >= 3, "Should have at least our 3 created documents");
    assert!(documents.iter().any(|d| d.title == "Document 1"));
    assert!(documents.iter().any(|d| d.title == "Document 2"));
    assert!(documents.iter().any(|d| d.title == "Document 3"));
}

#[tokio::test]
async fn test_get_documents_by_level() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, directory_path) = setup_campaign(&env).await;
    let module_id = setup_module(&env, campaign_id).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = DocumentService::new(&mut conn);

    // Create campaign-level document (no module_id)
    let campaign_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "custom".to_string(),
        document_type: "notes".to_string(),
        title: "Campaign Doc".to_string(),
        file_path: format!("{}/campaign-doc.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: true,
    };
    service.create_document(campaign_doc).expect("Failed to create doc");

    // Create module-level document (has module_id)
    let module_doc = NewDocument {
        campaign_id,
        module_id: Some(module_id),
        session_id: None,
        template_id: "custom".to_string(),
        document_type: "notes".to_string(),
        title: "Module Doc".to_string(),
        file_path: format!("{}/module-doc.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: true,
    };
    service.create_document(module_doc).expect("Failed to create doc");

    // Get campaign-level documents
    let campaign_docs = service
        .get_documents_by_level(campaign_id, "campaign", None, None)
        .expect("Failed to get campaign docs");

    assert!(campaign_docs.iter().any(|d| d.title == "Campaign Doc"));

    // Get module-level documents
    let module_docs = service
        .get_documents_by_level(campaign_id, "module", Some(module_id), None)
        .expect("Failed to get module docs");

    // Module docs may include auto-generated ones plus our manual one
    assert!(module_docs.iter().any(|d| d.title == "Module Doc"));
}

#[tokio::test]
async fn test_update_document() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, directory_path) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = DocumentService::new(&mut conn);

    let new_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "custom".to_string(),
        document_type: "notes".to_string(),
        title: "Original Title".to_string(),
        file_path: format!("{}/update-test.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: true,
    };

    let document = service.create_document(new_doc).expect("Failed to create doc");

    let update = UpdateDocument {
        title: Some("Updated Title".to_string()),
        updated_at: None,
        completed_at: None,
    };

    let updated = service
        .update_document(document.id, update)
        .expect("Failed to update document");

    assert_eq!(updated.title, "Updated Title");
}

#[tokio::test]
async fn test_complete_document() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, directory_path) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = DocumentService::new(&mut conn);

    let new_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "session_notes".to_string(),
        document_type: "notes".to_string(),
        title: "Complete Me".to_string(),
        file_path: format!("{}/complete-test.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };

    let document = service.create_document(new_doc).expect("Failed to create doc");
    assert!(!document.is_completed());

    let completed = service
        .complete_document(document.id)
        .expect("Failed to complete document");

    assert!(completed.is_completed());
    assert!(completed.completed_at.is_some());
}

#[tokio::test]
async fn test_delete_document() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, directory_path) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = DocumentService::new(&mut conn);

    let new_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "custom".to_string(),
        document_type: "notes".to_string(),
        title: "Delete Me".to_string(),
        file_path: format!("{}/delete-test.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: true,
    };

    let document = service.create_document(new_doc).expect("Failed to create doc");

    service.delete_document(document.id).expect("Failed to delete document");

    // Verify deleted by listing
    let documents = service
        .get_campaign_documents(campaign_id)
        .expect("Failed to get documents");

    assert!(
        !documents.iter().any(|d| d.id == document.id),
        "Document should be deleted"
    );
}

#[tokio::test]
async fn test_get_incomplete_documents() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, directory_path) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = DocumentService::new(&mut conn);

    // Create completed doc
    let completed_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "session_notes".to_string(),
        document_type: "notes".to_string(),
        title: "Completed Doc".to_string(),
        file_path: format!("{}/completed.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };
    let doc1 = service.create_document(completed_doc).expect("Failed to create doc");
    service.complete_document(doc1.id).expect("Failed to complete doc");

    // Create incomplete doc
    let incomplete_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "module_overview".to_string(),
        document_type: "notes".to_string(),
        title: "Incomplete Doc".to_string(),
        file_path: format!("{}/incomplete.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };
    service.create_document(incomplete_doc).expect("Failed to create doc");

    let incomplete = service
        .get_incomplete_documents(campaign_id)
        .expect("Failed to get incomplete docs");

    assert!(incomplete.iter().any(|d| d.title == "Incomplete Doc"));
    assert!(!incomplete.iter().any(|d| d.title == "Completed Doc"));
}

#[tokio::test]
async fn test_get_completed_documents() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, directory_path) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = DocumentService::new(&mut conn);

    // Create and complete a document
    let doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "session_notes".to_string(),
        document_type: "notes".to_string(),
        title: "Will Complete".to_string(),
        file_path: format!("{}/will-complete.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };
    let created = service.create_document(doc).expect("Failed to create doc");
    service.complete_document(created.id).expect("Failed to complete doc");

    let completed = service
        .get_completed_documents(campaign_id)
        .expect("Failed to get completed docs");

    assert!(completed.iter().any(|d| d.title == "Will Complete"));
}

#[tokio::test]
async fn test_create_user_document() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, _) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = DocumentService::new(&mut conn);

    let document = service
        .create_user_document(campaign_id, None, "My Notes", Some("Initial content"))
        .expect("Failed to create user document");

    assert_eq!(document.title, "My Notes");
    assert!(document.is_user_created);
    assert!(document.file_path.contains("My Notes") || document.file_path.contains("my-notes"));
}

#[tokio::test]
async fn test_get_user_documents() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, directory_path) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = DocumentService::new(&mut conn);

    // Create a user document
    service
        .create_user_document(campaign_id, None, "User Notes", None)
        .expect("Failed to create user document");

    // Create a system document
    let system_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "campaign_summary".to_string(),
        document_type: "notes".to_string(),
        title: "System Doc".to_string(),
        file_path: format!("{}/system.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };
    service.create_document(system_doc).expect("Failed to create doc");

    let user_docs = service
        .get_user_documents(campaign_id, None)
        .expect("Failed to get user docs");

    assert!(user_docs.iter().any(|d| d.title == "User Notes"));
    assert!(!user_docs.iter().any(|d| d.title == "System Doc"));
}

#[tokio::test]
async fn test_read_and_save_document_file() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (_, directory_path) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let service = DocumentService::new(&mut conn);

    let file_path = format!("{}/test-file.md", directory_path);

    // Save content
    let content = "# Test Document\n\nThis is test content.";
    service
        .save_document_file(&file_path, content)
        .expect("Failed to save document file");

    // Read it back
    let read_content = service
        .read_document_file(&file_path)
        .expect("Failed to read document file");

    assert_eq!(read_content, content);
}

#[tokio::test]
async fn test_document_file_not_found() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let service = DocumentService::new(&mut conn);

    let result = service.read_document_file("/nonexistent/path.md");
    assert!(result.is_err(), "Reading nonexistent file should fail");
}

#[tokio::test]
async fn test_system_vs_user_documents() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, directory_path) = setup_campaign(&env).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = DocumentService::new(&mut conn);

    // Create system document (template-based)
    let system_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "campaign_summary".to_string(),
        document_type: "notes".to_string(),
        title: "System Doc".to_string(),
        file_path: format!("{}/system.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };
    let sys = service.create_document(system_doc).expect("Failed to create doc");

    // Create user document
    let user_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "custom".to_string(),
        document_type: "notes".to_string(),
        title: "User Doc".to_string(),
        file_path: format!("{}/user.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: true,
    };
    let usr = service.create_document(user_doc).expect("Failed to create doc");

    assert!(!sys.is_user_created);
    assert!(usr.is_user_created);
}

#[tokio::test]
async fn test_document_levels() {
    let env = TestEnv::new().await.expect("Failed to create test environment");
    let (campaign_id, directory_path) = setup_campaign(&env).await;
    let module_id = setup_module(&env, campaign_id).await;

    let mut conn = env.state.db.get_connection().expect("Failed to get connection");
    let mut service = DocumentService::new(&mut conn);

    // Campaign level (no module_id, no session_id)
    let campaign_doc = NewDocument {
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "custom".to_string(),
        document_type: "notes".to_string(),
        title: "Campaign Level".to_string(),
        file_path: format!("{}/campaign-level.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: true,
    };
    let campaign_level = service.create_document(campaign_doc).expect("Failed to create");
    assert!(campaign_level.module_id.is_none());
    assert!(campaign_level.session_id.is_none());

    // Module level (has module_id, no session_id)
    let module_doc = NewDocument {
        campaign_id,
        module_id: Some(module_id),
        session_id: None,
        template_id: "custom".to_string(),
        document_type: "notes".to_string(),
        title: "Module Level".to_string(),
        file_path: format!("{}/module-level.md", directory_path),
        file_type: "markdown".to_string(),
        is_user_created: true,
    };
    let module_level = service.create_document(module_doc).expect("Failed to create");
    assert_eq!(module_level.module_id, Some(module_id));
    assert!(module_level.session_id.is_none());
}
