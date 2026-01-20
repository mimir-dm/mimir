//! Integration tests for template documents DAL

use crate::common::TestDatabase;
use mimir_dm_core::dal::campaign::template_documents::TemplateRepository;
use mimir_dm_core::models::campaign::template_documents::{
    NewTemplateDocument, UpdateTemplateDocument,
};

#[test]
fn test_create_template_with_auto_versioning() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create first version
    let template_v1 = NewTemplateDocument {
        document_id: "campaign-pitch".to_string(),
        version_number: None, // Should auto-assign to 1
        document_content: "# Campaign Pitch v1".to_string(),
        content_hash: None, // Will be computed
        document_type: Some("campaign_pitch".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("Create a pitch for players".to_string()),
        variables_schema: None,
        default_values: None,
        is_active: None, // Should auto-assign to true
        metadata: None,
    };

    let created_v1 = TemplateRepository::create(&mut conn, template_v1).unwrap();
    assert_eq!(created_v1.version_number, 1);
    assert!(created_v1.is_active);

    // Create second version
    let template_v2 = NewTemplateDocument {
        document_id: "campaign-pitch".to_string(),
        version_number: None, // Should auto-assign to 2
        document_content: "# Campaign Pitch v2".to_string(),
        content_hash: None, // Will be computed
        document_type: Some("campaign_pitch".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("Create an improved pitch for players".to_string()),
        variables_schema: None,
        default_values: None,
        is_active: None,
        metadata: None,
    };

    let created_v2 = TemplateRepository::create(&mut conn, template_v2).unwrap();
    assert_eq!(created_v2.version_number, 2);
    assert!(created_v2.is_active);

    // Check that v1 is no longer active
    let v1_check = TemplateRepository::get_version(&mut conn, "campaign-pitch", 1).unwrap();
    assert!(!v1_check.is_active);
}

#[test]
fn test_get_latest_template() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create three versions
    for i in 1..=3 {
        let template = NewTemplateDocument {
            document_id: "module-overview".to_string(),
            version_number: None,
            document_content: format!("# Module Overview v{}", i),
            content_hash: None,
            document_type: Some("module_overview".to_string()),
            document_level: Some("module".to_string()),
            purpose: Some("Overview template".to_string()),
            variables_schema: None,
            default_values: None,
            is_active: None,
            metadata: None,
        };
        TemplateRepository::create(&mut conn, template).unwrap();
    }

    // Get latest should return v3
    let latest = TemplateRepository::get_latest(&mut conn, "module-overview").unwrap();
    assert_eq!(latest.version_number, 3);
    assert!(latest.is_active);
    assert_eq!(latest.document_content, "# Module Overview v3");
}

#[test]
fn test_get_specific_version() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create two versions
    for i in 1..=2 {
        let template = NewTemplateDocument {
            document_id: "session-outline".to_string(),
            version_number: None,
            document_content: format!("# Session Outline v{}", i),
            content_hash: None,
            document_type: Some("session_outline".to_string()),
            document_level: Some("session".to_string()),
            purpose: Some("Session planning".to_string()),
            variables_schema: None,
            default_values: None,
            is_active: None,
            metadata: None,
        };
        TemplateRepository::create(&mut conn, template).unwrap();
    }

    // Get specific version
    let v1 = TemplateRepository::get_version(&mut conn, "session-outline", 1).unwrap();
    assert_eq!(v1.version_number, 1);
    assert_eq!(v1.document_content, "# Session Outline v1");
    assert!(!v1.is_active); // Should be inactive since v2 exists
}

#[test]
fn test_get_all_versions() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create three versions
    for i in 1..=3 {
        let template = NewTemplateDocument {
            document_id: "npc-tracker".to_string(),
            version_number: None,
            document_content: format!("# NPC Tracker v{}", i),
            content_hash: None,
            document_type: Some("major_npc_tracker".to_string()),
            document_level: Some("handout".to_string()),
            purpose: Some("Track NPCs".to_string()),
            variables_schema: None,
            default_values: None,
            is_active: None,
            metadata: None,
        };
        TemplateRepository::create(&mut conn, template).unwrap();
    }

    let all_versions = TemplateRepository::get_all_versions(&mut conn, "npc-tracker").unwrap();
    assert_eq!(all_versions.len(), 3);

    // Should be ordered by version descending
    assert_eq!(all_versions[0].version_number, 3);
    assert_eq!(all_versions[1].version_number, 2);
    assert_eq!(all_versions[2].version_number, 1);

    // Only latest should be active
    assert!(all_versions[0].is_active);
    assert!(!all_versions[1].is_active);
    assert!(!all_versions[2].is_active);
}

#[test]
fn test_get_by_level() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create templates at different levels
    let campaign_template = NewTemplateDocument {
        document_id: "campaign-bible".to_string(),
        version_number: None,
        document_content: "# Campaign Bible".to_string(),
        content_hash: None,
        document_type: Some("campaign_bible".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("Complete campaign reference".to_string()),
        variables_schema: None,
        default_values: None,
        is_active: None,
        metadata: None,
    };
    TemplateRepository::create(&mut conn, campaign_template).unwrap();

    let module_template = NewTemplateDocument {
        document_id: "module-dungeon".to_string(),
        version_number: None,
        document_content: "# Dungeon Module".to_string(),
        content_hash: None,
        document_type: Some("module_dungeon".to_string()),
        document_level: Some("module".to_string()),
        purpose: Some("Dungeon crawl template".to_string()),
        variables_schema: None,
        default_values: None,
        is_active: None,
        metadata: None,
    };
    TemplateRepository::create(&mut conn, module_template).unwrap();

    // Get campaign level templates
    let campaign_templates = TemplateRepository::get_by_level(&mut conn, "campaign").unwrap();
    assert_eq!(campaign_templates.len(), 1);
    assert_eq!(campaign_templates[0].document_id, "campaign-bible");

    // Get module level templates
    let module_templates = TemplateRepository::get_by_level(&mut conn, "module").unwrap();
    assert_eq!(module_templates.len(), 1);
    assert_eq!(module_templates[0].document_id, "module-dungeon");
}

#[test]
fn test_update_creates_new_version() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create initial version
    let template = NewTemplateDocument {
        document_id: "world-overview".to_string(),
        version_number: None,
        document_content: "# World Overview v1".to_string(),
        content_hash: None,
        document_type: Some("world_overview".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("World building".to_string()),
        variables_schema: Some(r#"[{"name": "world_name", "type": "string"}]"#.to_string()),
        default_values: Some(r#"{"world_name": "My World"}"#.to_string()),
        is_active: None,
        metadata: None,
    };
    let v1 = TemplateRepository::create(&mut conn, template).unwrap();
    assert_eq!(v1.version_number, 1);

    // Update the template
    let updates = UpdateTemplateDocument {
        document_content: Some("# World Overview v2 - Improved".to_string()),
        document_type: None,
        document_level: None,
        purpose: Some(Some("Enhanced world building template".to_string())),
        variables_schema: Some(Some(
            r#"[{"name": "world_name", "type": "string"}, {"name": "genre", "type": "string"}]"#
                .to_string(),
        )),
        default_values: None,
        updated_at: None,
        is_active: None,
        metadata: None,
    };

    let v2 = TemplateRepository::update(&mut conn, "world-overview", updates).unwrap();
    assert_eq!(v2.version_number, 2);
    assert!(v2.is_active);
    assert_eq!(v2.document_content, "# World Overview v2 - Improved");
    assert_eq!(v2.purpose.unwrap(), "Enhanced world building template");

    // Check v1 is inactive
    let v1_check = TemplateRepository::get_version(&mut conn, "world-overview", 1).unwrap();
    assert!(!v1_check.is_active);
}

#[test]
fn test_set_active_version() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create three versions
    for i in 1..=3 {
        let template = NewTemplateDocument {
            document_id: "faction-template".to_string(),
            version_number: None,
            document_content: format!("# Faction Template v{}", i),
            content_hash: None,
            document_type: Some("faction_template".to_string()),
            document_level: Some("campaign".to_string()),
            purpose: Some("Faction tracking".to_string()),
            variables_schema: None,
            default_values: None,
            is_active: None,
            metadata: None,
        };
        TemplateRepository::create(&mut conn, template).unwrap();
    }

    // Current active should be v3
    let latest = TemplateRepository::get_latest(&mut conn, "faction-template").unwrap();
    assert_eq!(latest.version_number, 3);

    // Set v2 as active
    TemplateRepository::set_active_version(&mut conn, "faction-template", 2).unwrap();

    // Now latest should return v2
    let latest = TemplateRepository::get_latest(&mut conn, "faction-template").unwrap();
    assert_eq!(latest.version_number, 2);
    assert!(latest.is_active);

    // Check all versions - only v2 should be active
    let all_versions = TemplateRepository::get_all_versions(&mut conn, "faction-template").unwrap();
    for version in all_versions {
        assert_eq!(version.is_active, version.version_number == 2);
    }
}

#[test]
fn test_delete_specific_version() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create two versions
    for i in 1..=2 {
        let template = NewTemplateDocument {
            document_id: "clue-tracker".to_string(),
            version_number: None,
            document_content: format!("# Clue Tracker v{}", i),
            content_hash: None,
            document_type: Some("clue_tracker".to_string()),
            document_level: Some("session".to_string()),
            purpose: Some("Track clues".to_string()),
            variables_schema: None,
            default_values: None,
            is_active: None,
            metadata: None,
        };
        TemplateRepository::create(&mut conn, template).unwrap();
    }

    // Delete v1
    let deleted = TemplateRepository::delete_version(&mut conn, "clue-tracker", 1).unwrap();
    assert_eq!(deleted, 1);

    // v1 should be gone
    let v1_result = TemplateRepository::get_version(&mut conn, "clue-tracker", 1);
    assert!(v1_result.is_err());

    // v2 should still exist and be active
    let v2 = TemplateRepository::get_version(&mut conn, "clue-tracker", 2).unwrap();
    assert_eq!(v2.version_number, 2);
    assert!(v2.is_active);
}

#[test]
fn test_delete_all_versions() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create three versions
    for i in 1..=3 {
        let template = NewTemplateDocument {
            document_id: "document-tracker".to_string(),
            version_number: None,
            document_content: format!("# Document Tracker v{}", i),
            content_hash: None,
            document_type: Some("document_tracker".to_string()),
            document_level: Some("handout".to_string()),
            purpose: Some("Track documents".to_string()),
            variables_schema: None,
            default_values: None,
            is_active: None,
            metadata: None,
        };
        TemplateRepository::create(&mut conn, template).unwrap();
    }

    // Delete all versions
    let deleted = TemplateRepository::delete_all_versions(&mut conn, "document-tracker").unwrap();
    assert_eq!(deleted, 3);

    // No versions should exist
    let all_versions = TemplateRepository::get_all_versions(&mut conn, "document-tracker").unwrap();
    assert_eq!(all_versions.len(), 0);
}

#[test]
fn test_template_with_variables_and_defaults() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let variables_schema = serde_json::json!([
        {
            "name": "campaign_name",
            "type": "string",
            "description": "The name of your campaign",
            "default": "[Campaign Name]",
            "required": true
        },
        {
            "name": "genre",
            "type": "string",
            "description": "Primary genre and tone",
            "default": "Fantasy Adventure",
            "required": true
        }
    ]);

    let defaults = serde_json::json!({
        "campaign_name": "[Campaign Name]",
        "genre": "Fantasy Adventure"
    });

    let template = NewTemplateDocument {
        document_id: "campaign-pitch-advanced".to_string(),
        version_number: None,
        document_content: "# {{campaign_name}}\n\nGenre: {{genre}}".to_string(),
        content_hash: None,
        document_type: Some("campaign_pitch".to_string()),
        document_level: Some("campaign".to_string()),
        purpose: Some("Create a pitch with variables".to_string()),
        variables_schema: Some(variables_schema.to_string()),
        default_values: Some(defaults.to_string()),
        is_active: None,
        metadata: None,
    };

    let created = TemplateRepository::create(&mut conn, template).unwrap();
    assert!(created.variables_schema.is_some());
    assert!(created.default_values.is_some());

    // Parse and verify the JSON
    let parsed_vars = created.parse_variables_schema().unwrap();
    assert!(parsed_vars.is_array());

    let parsed_defaults = created.parse_default_values().unwrap();
    assert_eq!(parsed_defaults["campaign_name"], "[Campaign Name]");
    assert_eq!(parsed_defaults["genre"], "Fantasy Adventure");
}
