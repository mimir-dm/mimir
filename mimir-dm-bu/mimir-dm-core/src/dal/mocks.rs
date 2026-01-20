//! Mock implementations of repository traits for testing.
//!
//! This module provides in-memory mock implementations of repository traits
//! that can be used for unit testing services without requiring a database.
//!
//! # Usage
//!
//! ```ignore
//! use mimir_dm_core::dal::mocks::MockCampaignRepository;
//! use mimir_dm_core::dal::traits::CampaignRepositoryTrait;
//!
//! let mut mock = MockCampaignRepository::new();
//!
//! // Seed with test data
//! mock.seed_campaign(Campaign { id: 1, name: "Test".into(), ... });
//!
//! // Use in tests
//! let campaign = mock.find_by_id(1).unwrap();
//! assert_eq!(campaign.unwrap().name, "Test");
//! ```

use crate::dal::traits::{CampaignRepositoryTrait, DocumentRepositoryTrait, ModuleRepositoryTrait};
use crate::error::{DbError, Result};
use crate::models::campaign::campaigns::{Campaign, NewCampaign, UpdateCampaign};
use crate::models::campaign::documents::{Document, NewDocument, UpdateDocument};
use crate::models::campaign::modules::{Module, NewModule, UpdateModule};
use std::cell::RefCell;
use std::collections::HashMap;

// =============================================================================
// Mock Campaign Repository
// =============================================================================

/// In-memory mock implementation of [`CampaignRepositoryTrait`].
///
/// Stores campaigns in a HashMap for testing. Provides methods to seed
/// test data and configure behavior.
#[derive(Default)]
pub struct MockCampaignRepository {
    campaigns: RefCell<HashMap<i32, Campaign>>,
    next_id: RefCell<i32>,
    /// If set, all operations will return this error message
    force_error_msg: Option<String>,
}

impl MockCampaignRepository {
    /// Create a new empty mock repository
    pub fn new() -> Self {
        Self {
            campaigns: RefCell::new(HashMap::new()),
            next_id: RefCell::new(1),
            force_error_msg: None,
        }
    }

    /// Seed the mock with a campaign
    pub fn seed_campaign(&mut self, campaign: Campaign) {
        let id = campaign.id;
        self.campaigns.borrow_mut().insert(id, campaign);
        let mut next = self.next_id.borrow_mut();
        if id >= *next {
            *next = id + 1;
        }
    }

    /// Create a mock repository pre-seeded with campaigns
    pub fn with_campaigns(campaigns: Vec<Campaign>) -> Self {
        let mut mock = Self::new();
        for campaign in campaigns {
            mock.seed_campaign(campaign);
        }
        mock
    }

    /// Configure the mock to return an error for all operations
    pub fn with_error(mut self, error_msg: &str) -> Self {
        self.force_error_msg = Some(error_msg.to_string());
        self
    }

    fn check_error(&self) -> Result<()> {
        if let Some(ref msg) = self.force_error_msg {
            Err(DbError::InvalidData(msg.clone()))
        } else {
            Ok(())
        }
    }
}

impl CampaignRepositoryTrait for MockCampaignRepository {
    fn create(&mut self, new_campaign: NewCampaign) -> Result<Campaign> {
        self.check_error()?;
        let id = *self.next_id.borrow();
        *self.next_id.borrow_mut() += 1;

        let now = chrono::Utc::now().to_rfc3339();
        let campaign = Campaign {
            id,
            name: new_campaign.name,
            status: new_campaign.status,
            directory_path: new_campaign.directory_path,
            created_at: now.clone(),
            session_zero_date: None,
            first_session_date: None,
            last_activity_at: now,
            archived_at: None,
        };

        self.campaigns.borrow_mut().insert(id, campaign.clone());
        Ok(campaign)
    }

    fn find_by_id(&mut self, id: i32) -> Result<Option<Campaign>> {
        self.check_error()?;
        Ok(self.campaigns.borrow().get(&id).cloned())
    }

    fn update(&mut self, id: i32, update: UpdateCampaign) -> Result<Campaign> {
        self.check_error()?;
        let mut campaigns = self.campaigns.borrow_mut();
        let campaign = campaigns.get_mut(&id).ok_or_else(|| DbError::NotFound {
            entity_type: "Campaign".to_string(),
            id: id.to_string(),
        })?;

        if let Some(name) = update.name {
            campaign.name = name;
        }
        if let Some(status) = update.status {
            campaign.status = status;
        }
        if let Some(archived_at) = update.archived_at {
            campaign.archived_at = archived_at;
        }
        if let Some(last_activity_at) = update.last_activity_at {
            campaign.last_activity_at = last_activity_at;
        }

        Ok(campaign.clone())
    }

    fn transition_status(&mut self, id: i32, new_status: &str) -> Result<Campaign> {
        self.update(
            id,
            UpdateCampaign {
                status: Some(new_status.to_string()),
                last_activity_at: Some(chrono::Utc::now().to_rfc3339()),
                ..Default::default()
            },
        )
    }

    fn delete(&mut self, id: i32) -> Result<()> {
        self.check_error()?;
        self.campaigns.borrow_mut().remove(&id);
        Ok(())
    }

    fn list(&mut self) -> Result<Vec<Campaign>> {
        self.check_error()?;
        Ok(self.campaigns.borrow().values().cloned().collect())
    }

    fn list_by_status(&mut self, status: &str) -> Result<Vec<Campaign>> {
        self.check_error()?;
        Ok(self
            .campaigns
            .borrow()
            .values()
            .filter(|c| c.status == status)
            .cloned()
            .collect())
    }

    fn list_active(&mut self) -> Result<Vec<Campaign>> {
        self.check_error()?;
        Ok(self
            .campaigns
            .borrow()
            .values()
            .filter(|c| c.archived_at.is_none())
            .cloned()
            .collect())
    }

    fn list_archived(&mut self) -> Result<Vec<Campaign>> {
        self.check_error()?;
        Ok(self
            .campaigns
            .borrow()
            .values()
            .filter(|c| c.archived_at.is_some())
            .cloned()
            .collect())
    }

    fn archive(&mut self, id: i32) -> Result<Campaign> {
        self.update(
            id,
            UpdateCampaign {
                archived_at: Some(Some(chrono::Utc::now().to_rfc3339())),
                last_activity_at: Some(chrono::Utc::now().to_rfc3339()),
                ..Default::default()
            },
        )
    }

    fn unarchive(&mut self, id: i32) -> Result<Campaign> {
        self.update(
            id,
            UpdateCampaign {
                archived_at: Some(None),
                last_activity_at: Some(chrono::Utc::now().to_rfc3339()),
                ..Default::default()
            },
        )
    }
}

// =============================================================================
// Mock Module Repository
// =============================================================================

/// In-memory mock implementation of [`ModuleRepositoryTrait`].
#[derive(Default)]
pub struct MockModuleRepository {
    modules: RefCell<HashMap<i32, Module>>,
    next_id: RefCell<i32>,
    force_error_msg: Option<String>,
}

impl MockModuleRepository {
    pub fn new() -> Self {
        Self {
            modules: RefCell::new(HashMap::new()),
            next_id: RefCell::new(1),
            force_error_msg: None,
        }
    }

    pub fn seed_module(&mut self, module: Module) {
        let id = module.id;
        self.modules.borrow_mut().insert(id, module);
        let mut next = self.next_id.borrow_mut();
        if id >= *next {
            *next = id + 1;
        }
    }

    pub fn with_modules(modules: Vec<Module>) -> Self {
        let mut mock = Self::new();
        for module in modules {
            mock.seed_module(module);
        }
        mock
    }

    pub fn with_error(mut self, error_msg: &str) -> Self {
        self.force_error_msg = Some(error_msg.to_string());
        self
    }

    fn check_error(&self) -> Result<()> {
        if let Some(ref msg) = self.force_error_msg {
            Err(DbError::InvalidData(msg.clone()))
        } else {
            Ok(())
        }
    }
}

impl ModuleRepositoryTrait for MockModuleRepository {
    fn create(&mut self, new_module: NewModule) -> Result<Module> {
        self.check_error()?;
        let id = *self.next_id.borrow();
        *self.next_id.borrow_mut() += 1;

        let now = chrono::Utc::now().to_rfc3339();
        let module = Module {
            id,
            campaign_id: new_module.campaign_id,
            name: new_module.name,
            module_number: new_module.module_number,
            status: new_module.status,
            expected_sessions: new_module.expected_sessions,
            actual_sessions: 0,
            started_at: None,
            completed_at: None,
            created_at: now,
        };

        self.modules.borrow_mut().insert(id, module.clone());
        Ok(module)
    }

    fn find_by_id(&mut self, id: i32) -> Result<Option<Module>> {
        self.check_error()?;
        Ok(self.modules.borrow().get(&id).cloned())
    }

    fn update(&mut self, id: i32, update: UpdateModule) -> Result<Module> {
        self.check_error()?;
        let mut modules = self.modules.borrow_mut();
        let module = modules.get_mut(&id).ok_or_else(|| DbError::NotFound {
            entity_type: "Module".to_string(),
            id: id.to_string(),
        })?;

        if let Some(name) = update.name {
            module.name = name;
        }
        if let Some(status) = update.status {
            module.status = status;
        }
        if let Some(actual_sessions) = update.actual_sessions {
            module.actual_sessions = actual_sessions;
        }
        if let Some(started_at) = update.started_at {
            module.started_at = started_at;
        }
        if let Some(completed_at) = update.completed_at {
            module.completed_at = completed_at;
        }

        Ok(module.clone())
    }

    fn transition_status(&mut self, id: i32, new_status: &str) -> Result<Module> {
        let mut update = UpdateModule {
            status: Some(new_status.to_string()),
            ..Default::default()
        };

        if new_status == "active" {
            let module = self.find_by_id(id)?.ok_or_else(|| DbError::NotFound {
                entity_type: "Module".to_string(),
                id: id.to_string(),
            })?;
            if module.started_at.is_none() {
                update.started_at = Some(Some(chrono::Utc::now().to_rfc3339()));
            }
        } else if new_status == "completed" {
            update.completed_at = Some(Some(chrono::Utc::now().to_rfc3339()));
        }

        self.update(id, update)
    }

    fn increment_sessions(&mut self, id: i32) -> Result<Module> {
        self.check_error()?;
        let module = self.find_by_id(id)?.ok_or_else(|| DbError::NotFound {
            entity_type: "Module".to_string(),
            id: id.to_string(),
        })?;

        let mut update = UpdateModule {
            actual_sessions: Some(module.actual_sessions + 1),
            ..Default::default()
        };

        if module.actual_sessions == 0 && module.started_at.is_none() {
            update.started_at = Some(Some(chrono::Utc::now().to_rfc3339()));
        }

        self.update(id, update)
    }

    fn delete(&mut self, id: i32) -> Result<()> {
        self.check_error()?;
        self.modules.borrow_mut().remove(&id);
        Ok(())
    }

    fn list_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Module>> {
        self.check_error()?;
        let mut modules: Vec<_> = self
            .modules
            .borrow()
            .values()
            .filter(|m| m.campaign_id == campaign_id)
            .cloned()
            .collect();
        modules.sort_by_key(|m| m.module_number);
        Ok(modules)
    }

    fn list_by_campaign_and_status(
        &mut self,
        campaign_id: i32,
        status: &str,
    ) -> Result<Vec<Module>> {
        self.check_error()?;
        let mut modules: Vec<_> = self
            .modules
            .borrow()
            .values()
            .filter(|m| m.campaign_id == campaign_id && m.status == status)
            .cloned()
            .collect();
        modules.sort_by_key(|m| m.module_number);
        Ok(modules)
    }

    fn find_modules_needing_next(&mut self, campaign_id: i32) -> Result<Vec<Module>> {
        self.check_error()?;
        Ok(self
            .modules
            .borrow()
            .values()
            .filter(|m| m.campaign_id == campaign_id && m.status == "active")
            .filter(|m| m.should_trigger_next_module())
            .cloned()
            .collect())
    }

    fn get_next_module_number(&mut self, campaign_id: i32) -> Result<i32> {
        self.check_error()?;
        let max = self
            .modules
            .borrow()
            .values()
            .filter(|m| m.campaign_id == campaign_id)
            .map(|m| m.module_number)
            .max()
            .unwrap_or(0);
        Ok(max + 1)
    }
}

// =============================================================================
// Mock Document Repository
// =============================================================================

/// In-memory mock implementation of [`DocumentRepositoryTrait`].
#[derive(Default)]
pub struct MockDocumentRepository {
    documents: RefCell<HashMap<i32, Document>>,
    next_id: RefCell<i32>,
    force_error_msg: Option<String>,
}

impl MockDocumentRepository {
    pub fn new() -> Self {
        Self {
            documents: RefCell::new(HashMap::new()),
            next_id: RefCell::new(1),
            force_error_msg: None,
        }
    }

    pub fn seed_document(&mut self, document: Document) {
        let id = document.id;
        self.documents.borrow_mut().insert(id, document);
        let mut next = self.next_id.borrow_mut();
        if id >= *next {
            *next = id + 1;
        }
    }

    pub fn with_documents(documents: Vec<Document>) -> Self {
        let mut mock = Self::new();
        for doc in documents {
            mock.seed_document(doc);
        }
        mock
    }

    pub fn with_error(mut self, error_msg: &str) -> Self {
        self.force_error_msg = Some(error_msg.to_string());
        self
    }

    fn check_error(&self) -> Result<()> {
        if let Some(ref msg) = self.force_error_msg {
            Err(DbError::InvalidData(msg.clone()))
        } else {
            Ok(())
        }
    }
}

impl DocumentRepositoryTrait for MockDocumentRepository {
    fn create(&mut self, new_document: NewDocument) -> Result<Document> {
        self.check_error()?;
        let id = *self.next_id.borrow();
        *self.next_id.borrow_mut() += 1;

        let now = chrono::Utc::now().to_rfc3339();
        let document = Document {
            id,
            campaign_id: new_document.campaign_id,
            module_id: new_document.module_id,
            session_id: new_document.session_id,
            template_id: new_document.template_id,
            document_type: new_document.document_type,
            title: new_document.title,
            file_path: new_document.file_path,
            file_type: new_document.file_type,
            is_user_created: new_document.is_user_created,
            created_at: now.clone(),
            updated_at: now,
            completed_at: None,
        };

        self.documents.borrow_mut().insert(id, document.clone());
        Ok(document)
    }

    fn find_by_id(&mut self, document_id: i32) -> Result<Document> {
        self.check_error()?;
        self.documents
            .borrow()
            .get(&document_id)
            .cloned()
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Document".to_string(),
                id: document_id.to_string(),
            })
    }

    fn find_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Document>> {
        self.check_error()?;
        Ok(self
            .documents
            .borrow()
            .values()
            .filter(|d| d.campaign_id == campaign_id)
            .cloned()
            .collect())
    }

    fn find_by_module(&mut self, module_id: i32) -> Result<Vec<Document>> {
        self.check_error()?;
        Ok(self
            .documents
            .borrow()
            .values()
            .filter(|d| d.module_id == Some(module_id))
            .cloned()
            .collect())
    }

    fn find_by_module_and_template(
        &mut self,
        module_id: i32,
        template_id: &str,
    ) -> Result<Option<Document>> {
        self.check_error()?;
        Ok(self
            .documents
            .borrow()
            .values()
            .find(|d| d.module_id == Some(module_id) && d.template_id == template_id)
            .cloned())
    }

    fn find_by_session(&mut self, session_id: i32) -> Result<Vec<Document>> {
        self.check_error()?;
        Ok(self
            .documents
            .borrow()
            .values()
            .filter(|d| d.session_id == Some(session_id))
            .cloned()
            .collect())
    }

    fn find_by_template(&mut self, template_id: &str) -> Result<Vec<Document>> {
        self.check_error()?;
        Ok(self
            .documents
            .borrow()
            .values()
            .filter(|d| d.template_id == template_id)
            .cloned()
            .collect())
    }

    fn find_incomplete_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Document>> {
        self.check_error()?;
        Ok(self
            .documents
            .borrow()
            .values()
            .filter(|d| d.campaign_id == campaign_id && d.completed_at.is_none())
            .cloned()
            .collect())
    }

    fn find_completed_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Document>> {
        self.check_error()?;
        Ok(self
            .documents
            .borrow()
            .values()
            .filter(|d| d.campaign_id == campaign_id && d.completed_at.is_some())
            .cloned()
            .collect())
    }

    fn update(&mut self, document_id: i32, update: UpdateDocument) -> Result<Document> {
        self.check_error()?;
        let mut documents = self.documents.borrow_mut();
        let document = documents
            .get_mut(&document_id)
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Document".to_string(),
                id: document_id.to_string(),
            })?;

        if let Some(title) = update.title {
            document.title = title;
        }
        if let Some(completed_at) = update.completed_at {
            document.completed_at = Some(completed_at);
        }
        if let Some(updated_at) = update.updated_at {
            document.updated_at = updated_at;
        }

        Ok(document.clone())
    }

    fn mark_completed(&mut self, document_id: i32) -> Result<Document> {
        self.update(
            document_id,
            UpdateDocument {
                title: None,
                updated_at: Some(chrono::Utc::now().to_rfc3339()),
                completed_at: Some(chrono::Utc::now().to_rfc3339()),
            },
        )
    }

    fn delete(&mut self, document_id: i32) -> Result<usize> {
        self.check_error()?;
        if self.documents.borrow_mut().remove(&document_id).is_some() {
            Ok(1)
        } else {
            Ok(0)
        }
    }

    fn exists_by_path(&mut self, file_path: &str) -> Result<bool> {
        self.check_error()?;
        Ok(self
            .documents
            .borrow()
            .values()
            .any(|d| d.file_path == file_path))
    }

    fn find_handouts_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Document>> {
        self.check_error()?;
        Ok(self
            .documents
            .borrow()
            .values()
            .filter(|d| d.campaign_id == campaign_id && d.document_type == "handout")
            .cloned()
            .collect())
    }
}

// =============================================================================
// Test Fixtures - Helper functions to create test data
// =============================================================================

/// Create a test campaign with default values
pub fn create_test_campaign(id: i32, name: &str) -> Campaign {
    let now = chrono::Utc::now().to_rfc3339();
    Campaign {
        id,
        name: name.to_string(),
        status: "concept".to_string(),
        directory_path: format!("/tmp/test-campaigns/{}", name.to_lowercase().replace(' ', "-")),
        created_at: now.clone(),
        session_zero_date: None,
        first_session_date: None,
        last_activity_at: now,
        archived_at: None,
    }
}

/// Create a test module with default values
pub fn create_test_module(id: i32, campaign_id: i32, name: &str, module_number: i32) -> Module {
    let now = chrono::Utc::now().to_rfc3339();
    Module {
        id,
        campaign_id,
        name: name.to_string(),
        module_number,
        status: "planned".to_string(),
        expected_sessions: 4,
        actual_sessions: 0,
        started_at: None,
        completed_at: None,
        created_at: now,
    }
}

/// Create a test document with default values
pub fn create_test_document(id: i32, campaign_id: i32, title: &str) -> Document {
    let now = chrono::Utc::now().to_rfc3339();
    Document {
        id,
        campaign_id,
        module_id: None,
        session_id: None,
        template_id: "campaign_pitch".to_string(),
        document_type: "campaign_pitch".to_string(),
        title: title.to_string(),
        file_path: format!("/tmp/test-docs/{}.md", title.to_lowercase().replace(' ', "-")),
        file_type: "markdown".to_string(),
        is_user_created: false,
        created_at: now.clone(),
        updated_at: now,
        completed_at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_campaign_create() {
        let mut mock = MockCampaignRepository::new();

        let new_campaign = NewCampaign {
            name: "Test Campaign".to_string(),
            status: "concept".to_string(),
            directory_path: "/tmp/test".to_string(),
        };

        let campaign = mock.create(new_campaign).unwrap();
        assert_eq!(campaign.id, 1);
        assert_eq!(campaign.name, "Test Campaign");
        assert_eq!(campaign.status, "concept");
    }

    #[test]
    fn test_mock_campaign_find_by_id() {
        let mut mock = MockCampaignRepository::new();
        mock.seed_campaign(create_test_campaign(1, "Test"));

        let found = mock.find_by_id(1).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test");

        let not_found = mock.find_by_id(999).unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_mock_campaign_list_active() {
        let mut mock = MockCampaignRepository::new();

        let active = create_test_campaign(1, "Active");
        mock.seed_campaign(active);

        let mut archived = create_test_campaign(2, "Archived");
        archived.archived_at = Some(chrono::Utc::now().to_rfc3339());
        mock.seed_campaign(archived);

        let active_list = mock.list_active().unwrap();
        assert_eq!(active_list.len(), 1);
        assert_eq!(active_list[0].name, "Active");
    }

    #[test]
    fn test_mock_campaign_error_injection() {
        let mut mock = MockCampaignRepository::new().with_error("Forced error");

        let result = mock.list();
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_module_create_and_list() {
        let mut mock = MockModuleRepository::new();

        let new_module = NewModule {
            campaign_id: 1,
            name: "Test Module".to_string(),
            module_number: 1,
            status: "planned".to_string(),
            expected_sessions: 4,
        };

        let module = mock.create(new_module).unwrap();
        assert_eq!(module.id, 1);

        let modules = mock.list_by_campaign(1).unwrap();
        assert_eq!(modules.len(), 1);
    }

    #[test]
    fn test_mock_document_operations() {
        let mut mock = MockDocumentRepository::new();

        let new_doc = NewDocument {
            campaign_id: 1,
            module_id: None,
            session_id: None,
            template_id: "pitch".to_string(),
            document_type: "campaign_pitch".to_string(),
            title: "Test Doc".to_string(),
            file_path: "/tmp/test.md".to_string(),
            file_type: "markdown".to_string(),
            is_user_created: false,
        };

        let doc = mock.create(new_doc).unwrap();
        assert_eq!(doc.id, 1);

        // Test exists_by_path
        assert!(mock.exists_by_path("/tmp/test.md").unwrap());
        assert!(!mock.exists_by_path("/tmp/nonexistent.md").unwrap());

        // Test mark_completed
        let completed = mock.mark_completed(1).unwrap();
        assert!(completed.completed_at.is_some());
    }
}
