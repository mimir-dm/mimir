//! Tests for template duplicate detection using content hashing

use crate::common::TestDatabase;
use mimir_dm_core::dal::campaign::template_documents::TemplateRepository;
use mimir_dm_core::models::campaign::template_documents::{
    NewTemplateDocument, UpdateTemplateDocument,
};

#[test]
fn test_duplicate_content_prevents_new_version() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create first version
    let template_v1 = NewTemplateDocument {
        document_id: "test-duplicate".to_string(),
        version_number: None,
        document_content: "# Test Template\nThis is the content.".to_string(),
        content_hash: None,
        document_type: Some("test".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("Testing duplicates".to_string()),
        variables_schema: None,
        default_values: None,
        is_active: None,
        metadata: None,
    };

    let created_v1 = TemplateRepository::create(&mut conn, template_v1).unwrap();
    assert_eq!(created_v1.version_number, 1);

    // Try to create with same content
    let template_duplicate = NewTemplateDocument {
        document_id: "test-duplicate".to_string(),
        version_number: None,
        document_content: "# Test Template\nThis is the content.".to_string(), // Same content
        content_hash: None,
        document_type: Some("test".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("Testing duplicates - updated purpose".to_string()), // Different metadata
        variables_schema: None,
        default_values: None,
        is_active: None,
        metadata: None,
    };

    let result = TemplateRepository::create(&mut conn, template_duplicate).unwrap();

    // Should return the existing version, not create a new one
    assert_eq!(result.version_number, 1);
    assert_eq!(result.document_id, created_v1.document_id);
    assert_eq!(result.content_hash, created_v1.content_hash);

    // Verify no new version was created
    let all_versions = TemplateRepository::get_all_versions(&mut conn, "test-duplicate").unwrap();
    assert_eq!(all_versions.len(), 1);
}

#[test]
fn test_different_content_creates_new_version() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create first version
    let template_v1 = NewTemplateDocument {
        document_id: "test-versions".to_string(),
        version_number: None,
        document_content: "# Version 1".to_string(),
        content_hash: None,
        document_type: Some("test".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("Testing versions".to_string()),
        variables_schema: None,
        default_values: None,
        is_active: None,
        metadata: None,
    };

    let created_v1 = TemplateRepository::create(&mut conn, template_v1).unwrap();
    assert_eq!(created_v1.version_number, 1);

    // Create with different content
    let template_v2 = NewTemplateDocument {
        document_id: "test-versions".to_string(),
        version_number: None,
        document_content: "# Version 2 - Different content".to_string(), // Different content
        content_hash: None,
        document_type: Some("test".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("Testing versions".to_string()),
        variables_schema: None,
        default_values: None,
        is_active: None,
        metadata: None,
    };

    let created_v2 = TemplateRepository::create(&mut conn, template_v2).unwrap();

    // Should create new version
    assert_eq!(created_v2.version_number, 2);
    assert_ne!(created_v2.content_hash, created_v1.content_hash);

    // Verify two versions exist
    let all_versions = TemplateRepository::get_all_versions(&mut conn, "test-versions").unwrap();
    assert_eq!(all_versions.len(), 2);
}

#[test]
fn test_update_with_same_content_returns_existing() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create initial version
    let template = NewTemplateDocument {
        document_id: "test-update-duplicate".to_string(),
        version_number: None,
        document_content: "# Original Content".to_string(),
        content_hash: None,
        document_type: Some("test".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("Testing update duplicates".to_string()),
        variables_schema: None,
        default_values: None,
        is_active: None,
        metadata: None,
    };

    let created = TemplateRepository::create(&mut conn, template).unwrap();
    assert_eq!(created.version_number, 1);

    // Try to update with same content
    let updates = UpdateTemplateDocument {
        document_content: Some("# Original Content".to_string()), // Same content
        document_type: None,
        document_level: None,
        purpose: Some(Some("Updated purpose".to_string())), // Different metadata
        variables_schema: None,
        default_values: None,
        updated_at: None,
        is_active: None,
        metadata: None,
    };

    let result = TemplateRepository::update(&mut conn, "test-update-duplicate", updates).unwrap();

    // Should return existing version without creating new one
    assert_eq!(result.version_number, 1);
    assert_eq!(result.content_hash, created.content_hash);

    // Verify no new version was created
    let all_versions =
        TemplateRepository::get_all_versions(&mut conn, "test-update-duplicate").unwrap();
    assert_eq!(all_versions.len(), 1);
}

#[test]
fn test_can_revert_to_old_content() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create v1
    let template_v1 = NewTemplateDocument {
        document_id: "test-revert".to_string(),
        version_number: None,
        document_content: "# Version 1 Content".to_string(),
        content_hash: None,
        document_type: Some("test".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("Testing revert".to_string()),
        variables_schema: None,
        default_values: None,
        is_active: None,
        metadata: None,
    };

    let v1 = TemplateRepository::create(&mut conn, template_v1).unwrap();
    let v1_hash = v1.content_hash.clone();

    // Create v2 with different content
    let template_v2 = NewTemplateDocument {
        document_id: "test-revert".to_string(),
        version_number: None,
        document_content: "# Version 2 Content - Different".to_string(),
        content_hash: None,
        document_type: Some("test".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("Testing revert".to_string()),
        variables_schema: None,
        default_values: None,
        is_active: None,
        metadata: None,
    };

    let v2 = TemplateRepository::create(&mut conn, template_v2).unwrap();
    assert_eq!(v2.version_number, 2);
    assert_ne!(v2.content_hash, v1_hash);

    // Now try to create v3 with v1's content (reverting)
    let template_v3 = NewTemplateDocument {
        document_id: "test-revert".to_string(),
        version_number: None,
        document_content: "# Version 1 Content".to_string(), // Same as v1
        content_hash: None,
        document_type: Some("test".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("Testing revert".to_string()),
        variables_schema: None,
        default_values: None,
        is_active: None,
        metadata: None,
    };

    let v3 = TemplateRepository::create(&mut conn, template_v3).unwrap();

    // Should create v3 because content differs from v2 (the latest)
    assert_eq!(v3.version_number, 3);
    assert_eq!(v3.content_hash, v1_hash); // Same hash as v1

    // All three versions should exist
    let all_versions = TemplateRepository::get_all_versions(&mut conn, "test-revert").unwrap();
    assert_eq!(all_versions.len(), 3);
}
