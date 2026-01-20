//! Integration tests for Document repository

use crate::common::TestDatabase;
use mimir_dm_core::{
    dal::campaign::{
        campaigns::CampaignRepository, documents::DocumentRepository, modules::ModuleRepository,
        template_documents::TemplateRepository,
    },
    models::campaign::{
        campaigns::NewCampaign,
        documents::{DocumentLevel, NewDocument, UpdateDocument},
        modules::NewModule,
        template_documents::NewTemplateDocument,
    },
};

#[test]
fn test_create_document() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create a campaign first
    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            status: "concept".to_string(),
            directory_path: "/test/path".to_string(),
        })
        .expect("Failed to create campaign");

    // Create a template
    let template = TemplateRepository::create(
        &mut conn,
        NewTemplateDocument {
            document_id: "test-template".to_string(),
            version_number: Some(1),
            document_content: "Test content".to_string(),
            content_hash: Some("hash123".to_string()),
            document_type: Some("planning".to_string()),
            document_level: Some("campaign".to_string()),
            purpose: Some("Test purpose".to_string()),
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        },
    )
    .expect("Failed to create template");

    // Create a document
    let new_doc = NewDocument {
        campaign_id: campaign.id,
        module_id: None,
        session_id: None,
        template_id: template.document_id.clone(),
        document_type: "planning".to_string(),
        title: "Test Document".to_string(),
        file_path: "/test/doc.md".to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
    };

    let doc = DocumentRepository::create(&mut conn, new_doc).expect("Failed to create document");

    assert_eq!(doc.campaign_id, campaign.id);
    assert_eq!(doc.title, "Test Document");
    assert_eq!(doc.template_id, template.document_id);
    assert!(doc.completed_at.is_none());
}

#[test]
fn test_find_document_by_id() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create test data
    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            status: "concept".to_string(),
            directory_path: "/test/path".to_string(),
        })
        .expect("Failed to create campaign");

    let template = TemplateRepository::create(
        &mut conn,
        NewTemplateDocument {
            document_id: "test-template-2".to_string(),
            version_number: Some(1),
            document_content: "Content".to_string(),
            content_hash: Some("hash456".to_string()),
            document_type: Some("session".to_string()),
            document_level: Some("session".to_string()),
            purpose: None,
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        },
    )
    .expect("Failed to create template");

    let doc = DocumentRepository::create(
        &mut conn,
        NewDocument {
            campaign_id: campaign.id,
            module_id: None,
            session_id: None,
            template_id: template.document_id,
            document_type: "session".to_string(),
            title: "Find Me".to_string(),
            file_path: "/test/find.md".to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
        },
    )
    .expect("Failed to create document");

    let found = DocumentRepository::find_by_id(&mut conn, doc.id).expect("Failed to find document");

    assert_eq!(found.id, doc.id);
    assert_eq!(found.title, "Find Me");
}

#[test]
fn test_find_documents_by_campaign() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create two campaigns
    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign1 = campaign_repo
        .create(NewCampaign {
            name: "Campaign 1".to_string(),
            status: "concept".to_string(),
            directory_path: "/test/camp1".to_string(),
        })
        .expect("Failed to create campaign 1");

    let campaign2 = campaign_repo
        .create(NewCampaign {
            name: "Campaign 2".to_string(),
            status: "concept".to_string(),
            directory_path: "/test/camp2".to_string(),
        })
        .expect("Failed to create campaign 2");

    // Create template
    let template = TemplateRepository::create(
        &mut conn,
        NewTemplateDocument {
            document_id: "shared-template".to_string(),
            version_number: Some(1),
            document_content: "Content".to_string(),
            content_hash: Some("hash789".to_string()),
            document_type: Some("planning".to_string()),
            document_level: Some("campaign".to_string()),
            purpose: None,
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        },
    )
    .expect("Failed to create template");

    // Create documents for campaign 1
    for i in 1..=3 {
        DocumentRepository::create(
            &mut conn,
            NewDocument {
                campaign_id: campaign1.id,
                module_id: None,
                session_id: None,
                template_id: template.document_id.clone(),
                document_type: "planning".to_string(),
                title: format!("Doc {}", i),
                file_path: format!("/test/doc{}.md", i),
                file_type: "markdown".to_string(),
                is_user_created: false,
            },
        )
        .expect("Failed to create document");
    }

    // Create documents for campaign 2
    for i in 1..=2 {
        DocumentRepository::create(
            &mut conn,
            NewDocument {
                campaign_id: campaign2.id,
                module_id: None,
                session_id: None,
                template_id: template.document_id.clone(),
                document_type: "planning".to_string(),
                title: format!("Other Doc {}", i),
                file_path: format!("/test/other{}.md", i),
                file_type: "markdown".to_string(),
                is_user_created: false,
            },
        )
        .expect("Failed to create document");
    }

    let camp1_docs = DocumentRepository::find_by_campaign(&mut conn, campaign1.id)
        .expect("Failed to find campaign 1 documents");
    let camp2_docs = DocumentRepository::find_by_campaign(&mut conn, campaign2.id)
        .expect("Failed to find campaign 2 documents");

    assert_eq!(camp1_docs.len(), 3);
    assert_eq!(camp2_docs.len(), 2);
}

#[test]
fn test_find_documents_by_module() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create campaign and module
    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Module Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: "/test/modules".to_string(),
        })
        .expect("Failed to create campaign");

    let mut module_repo = ModuleRepository::new(&mut conn);
    let module = module_repo
        .create(NewModule {
            campaign_id: campaign.id,
            name: "Test Module".to_string(),
            module_number: 1,
            status: "planning".to_string(),
            expected_sessions: 5,
        })
        .expect("Failed to create module");

    // Create template
    let template = TemplateRepository::create(
        &mut conn,
        NewTemplateDocument {
            document_id: "module-template".to_string(),
            version_number: Some(1),
            document_content: "Module content".to_string(),
            content_hash: Some("hash_mod".to_string()),
            document_type: Some("module".to_string()),
            document_level: Some("module".to_string()),
            purpose: None,
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        },
    )
    .expect("Failed to create template");

    // Create module documents
    for i in 1..=2 {
        DocumentRepository::create(
            &mut conn,
            NewDocument {
                campaign_id: campaign.id,
                module_id: Some(module.id),
                session_id: None,
                template_id: template.document_id.clone(),
                document_type: "module".to_string(),
                title: format!("Module Doc {}", i),
                file_path: format!("/test/mod_doc{}.md", i),
                file_type: "markdown".to_string(),
                is_user_created: false,
            },
        )
        .expect("Failed to create module document");
    }

    // Create campaign-level document (should not be included)
    DocumentRepository::create(
        &mut conn,
        NewDocument {
            campaign_id: campaign.id,
            module_id: None,
            session_id: None,
            template_id: template.document_id.clone(),
            document_type: "planning".to_string(),
            title: "Campaign Doc".to_string(),
            file_path: "/test/camp_doc.md".to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
        },
    )
    .expect("Failed to create campaign document");

    let module_docs = DocumentRepository::find_by_module(&mut conn, module.id)
        .expect("Failed to find module documents");

    assert_eq!(module_docs.len(), 2);
    assert!(module_docs.iter().all(|d| d.module_id == Some(module.id)));
}

#[test]
fn test_find_handouts() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Handout Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: "/test/handouts".to_string(),
        })
        .expect("Failed to create campaign");

    // Create template for handouts
    let template = TemplateRepository::create(
        &mut conn,
        NewTemplateDocument {
            document_id: "handout-template".to_string(),
            version_number: Some(1),
            document_content: "Handout content".to_string(),
            content_hash: Some("hash_hand".to_string()),
            document_type: Some("handout".to_string()),
            document_level: Some("handout".to_string()),
            purpose: None,
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        },
    )
    .expect("Failed to create template");

    // Create handouts
    for i in 1..=2 {
        DocumentRepository::create(
            &mut conn,
            NewDocument {
                campaign_id: campaign.id,
                module_id: None,
                session_id: None,
                template_id: template.document_id.clone(),
                document_type: "handout".to_string(),
                title: format!("Handout {}", i),
                file_path: format!("/test/handout{}.md", i),
                file_type: "markdown".to_string(),
                is_user_created: false,
            },
        )
        .expect("Failed to create handout");
    }

    // Create non-handout document
    DocumentRepository::create(
        &mut conn,
        NewDocument {
            campaign_id: campaign.id,
            module_id: None,
            session_id: None,
            template_id: template.document_id.clone(),
            document_type: "planning".to_string(),
            title: "Not a handout".to_string(),
            file_path: "/test/not_handout.md".to_string(),
            file_type: "markdown".to_string(),
            is_user_created: false,
        },
    )
    .expect("Failed to create document");

    let handouts = DocumentRepository::find_handouts_by_campaign(&mut conn, campaign.id)
        .expect("Failed to find handouts");

    assert_eq!(handouts.len(), 2);
    assert!(handouts.iter().all(|d| d.document_type == "handout"));
}

#[test]
fn test_update_document() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create test document
    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Update Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: "/test/update".to_string(),
        })
        .expect("Failed to create campaign");

    let template = TemplateRepository::create(
        &mut conn,
        NewTemplateDocument {
            document_id: "update-template".to_string(),
            version_number: Some(1),
            document_content: "Content".to_string(),
            content_hash: Some("hash_upd".to_string()),
            document_type: Some("planning".to_string()),
            document_level: Some("campaign".to_string()),
            purpose: None,
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        },
    )
    .expect("Failed to create template");

    let doc = DocumentRepository::create(
        &mut conn,
        NewDocument {
            campaign_id: campaign.id,
            module_id: None,
            session_id: None,
            template_id: template.document_id,
            document_type: "planning".to_string(),
            title: "Original Title".to_string(),
            file_path: "/test/update.md".to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
        },
    )
    .expect("Failed to create document");

    // Update the document
    let update = UpdateDocument {
        title: Some("Updated Title".to_string()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        completed_at: None,
    };

    let updated =
        DocumentRepository::update(&mut conn, doc.id, update).expect("Failed to update document");

    assert_eq!(updated.title, "Updated Title");
    assert!(updated.completed_at.is_none());
}

#[test]
fn test_mark_completed() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    // Create test document
    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Complete Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: "/test/complete".to_string(),
        })
        .expect("Failed to create campaign");

    let template = TemplateRepository::create(
        &mut conn,
        NewTemplateDocument {
            document_id: "complete-template".to_string(),
            version_number: Some(1),
            document_content: "Content".to_string(),
            content_hash: Some("hash_comp".to_string()),
            document_type: Some("planning".to_string()),
            document_level: Some("campaign".to_string()),
            purpose: None,
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        },
    )
    .expect("Failed to create template");

    let doc = DocumentRepository::create(
        &mut conn,
        NewDocument {
            campaign_id: campaign.id,
            module_id: None,
            session_id: None,
            template_id: template.document_id,
            document_type: "planning".to_string(),
            title: "To Complete".to_string(),
            file_path: "/test/complete.md".to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
        },
    )
    .expect("Failed to create document");

    assert!(doc.completed_at.is_none());

    let completed = DocumentRepository::mark_completed(&mut conn, doc.id)
        .expect("Failed to mark document completed");

    assert!(completed.completed_at.is_some());
    assert!(completed.is_completed());
}

#[test]
fn test_find_incomplete_and_completed() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Status Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: "/test/status".to_string(),
        })
        .expect("Failed to create campaign");

    let template = TemplateRepository::create(
        &mut conn,
        NewTemplateDocument {
            document_id: "status-template".to_string(),
            version_number: Some(1),
            document_content: "Content".to_string(),
            content_hash: Some("hash_stat".to_string()),
            document_type: Some("planning".to_string()),
            document_level: Some("campaign".to_string()),
            purpose: None,
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        },
    )
    .expect("Failed to create template");

    // Create incomplete documents
    for i in 1..=3 {
        DocumentRepository::create(
            &mut conn,
            NewDocument {
                campaign_id: campaign.id,
                module_id: None,
                session_id: None,
                template_id: template.document_id.clone(),
                document_type: "planning".to_string(),
                title: format!("Incomplete {}", i),
                file_path: format!("/test/incomplete{}.md", i),
                file_type: "markdown".to_string(),
                is_user_created: false,
            },
        )
        .expect("Failed to create document");
    }

    // Create and complete some documents
    for i in 1..=2 {
        let doc = DocumentRepository::create(
            &mut conn,
            NewDocument {
                campaign_id: campaign.id,
                module_id: None,
                session_id: None,
                template_id: template.document_id.clone(),
                document_type: "planning".to_string(),
                title: format!("Complete {}", i),
                file_path: format!("/test/complete{}.md", i),
                file_type: "markdown".to_string(),
                is_user_created: false,
            },
        )
        .expect("Failed to create document");

        DocumentRepository::mark_completed(&mut conn, doc.id).expect("Failed to complete document");
    }

    let incomplete = DocumentRepository::find_incomplete_by_campaign(&mut conn, campaign.id)
        .expect("Failed to find incomplete documents");
    let completed = DocumentRepository::find_completed_by_campaign(&mut conn, campaign.id)
        .expect("Failed to find completed documents");

    assert_eq!(incomplete.len(), 3);
    assert_eq!(completed.len(), 2);
    assert!(incomplete.iter().all(|d| !d.is_completed()));
    assert!(completed.iter().all(|d| d.is_completed()));
}

#[test]
fn test_delete_document() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Delete Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: "/test/delete".to_string(),
        })
        .expect("Failed to create campaign");

    let template = TemplateRepository::create(
        &mut conn,
        NewTemplateDocument {
            document_id: "delete-template".to_string(),
            version_number: Some(1),
            document_content: "Content".to_string(),
            content_hash: Some("hash_del".to_string()),
            document_type: Some("planning".to_string()),
            document_level: Some("campaign".to_string()),
            purpose: None,
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        },
    )
    .expect("Failed to create template");

    let doc = DocumentRepository::create(
        &mut conn,
        NewDocument {
            campaign_id: campaign.id,
            module_id: None,
            session_id: None,
            template_id: template.document_id,
            document_type: "planning".to_string(),
            title: "To Delete".to_string(),
            file_path: "/test/delete.md".to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
        },
    )
    .expect("Failed to create document");

    let deleted = DocumentRepository::delete(&mut conn, doc.id).expect("Failed to delete document");

    assert_eq!(deleted, 1);

    // Verify it's gone
    let result = DocumentRepository::find_by_id(&mut conn, doc.id);
    assert!(result.is_err());
}

#[test]
fn test_document_level() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Level Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: "/test/level".to_string(),
        })
        .expect("Failed to create campaign");

    let mut module_repo = ModuleRepository::new(&mut conn);
    let module = module_repo
        .create(NewModule {
            campaign_id: campaign.id,
            name: "Test Module".to_string(),
            module_number: 1,
            status: "active".to_string(),
            expected_sessions: 5,
        })
        .expect("Failed to create module");

    let template = TemplateRepository::create(
        &mut conn,
        NewTemplateDocument {
            document_id: "level-template".to_string(),
            version_number: Some(1),
            document_content: "Content".to_string(),
            content_hash: Some("hash_lev".to_string()),
            document_type: Some("general".to_string()),
            document_level: Some("campaign".to_string()),
            purpose: None,
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        },
    )
    .expect("Failed to create template");

    // Test campaign level
    let camp_doc = DocumentRepository::create(
        &mut conn,
        NewDocument {
            campaign_id: campaign.id,
            module_id: None,
            session_id: None,
            template_id: template.document_id.clone(),
            document_type: "planning".to_string(),
            title: "Campaign Level".to_string(),
            file_path: "/test/camp_level.md".to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
        },
    )
    .expect("Failed to create document");

    assert_eq!(camp_doc.level(), DocumentLevel::Campaign);

    // Test module level
    let mod_doc = DocumentRepository::create(
        &mut conn,
        NewDocument {
            campaign_id: campaign.id,
            module_id: Some(module.id),
            session_id: None,
            template_id: template.document_id.clone(),
            document_type: "module".to_string(),
            title: "Module Level".to_string(),
            file_path: "/test/mod_level.md".to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
        },
    )
    .expect("Failed to create document");

    assert_eq!(mod_doc.level(), DocumentLevel::Module);

    // Test handout level
    let handout_doc = DocumentRepository::create(
        &mut conn,
        NewDocument {
            campaign_id: campaign.id,
            module_id: None,
            session_id: None,
            template_id: template.document_id.clone(),
            document_type: "handout".to_string(),
            title: "Handout Level".to_string(),
            file_path: "/test/handout_level.md".to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
        },
    )
    .expect("Failed to create document");

    assert_eq!(handout_doc.level(), DocumentLevel::Handout);
}

#[test]
fn test_exists_by_path() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Exists Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: "/test/exists".to_string(),
        })
        .expect("Failed to create campaign");

    let template = TemplateRepository::create(
        &mut conn,
        NewTemplateDocument {
            document_id: "exists-template".to_string(),
            version_number: Some(1),
            document_content: "Content".to_string(),
            content_hash: Some("hash_ex".to_string()),
            document_type: Some("planning".to_string()),
            document_level: Some("campaign".to_string()),
            purpose: None,
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        },
    )
    .expect("Failed to create template");

    let _doc = DocumentRepository::create(
        &mut conn,
        NewDocument {
            campaign_id: campaign.id,
            module_id: None,
            session_id: None,
            template_id: template.document_id,
            document_type: "planning".to_string(),
            title: "Check Path".to_string(),
            file_path: "/test/check_path.md".to_string(),
        file_type: "markdown".to_string(),
        is_user_created: false,
        },
    )
    .expect("Failed to create document");

    assert!(
        DocumentRepository::exists_by_path(&mut conn, "/test/check_path.md")
            .expect("Failed to check existence")
    );
    assert!(
        !DocumentRepository::exists_by_path(&mut conn, "/test/nonexistent.md")
            .expect("Failed to check existence")
    );
}
