//! Document Service
//!
//! Business logic for document management (campaign and module markdown content).

use chrono::Utc;
use diesel::SqliteConnection;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{Document, NewDocument, UpdateDocument as DalUpdateDocument};
use crate::services::{ServiceError, ServiceResult};

/// Input for creating a blank document.
#[derive(Debug, Clone)]
pub struct CreateDocumentInput {
    /// Campaign this document belongs to
    pub campaign_id: String,
    /// Module this document belongs to (optional - None for campaign-level)
    pub module_id: Option<String>,
    /// Document title
    pub title: String,
    /// Document type (defaults to "user_document")
    pub doc_type: Option<String>,
    /// Initial content (defaults to empty)
    pub content: Option<String>,
}

impl CreateDocumentInput {
    /// Create input for a campaign-level document.
    pub fn for_campaign(campaign_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            campaign_id: campaign_id.into(),
            module_id: None,
            title: title.into(),
            doc_type: None,
            content: None,
        }
    }

    /// Create input for a module-level document.
    pub fn for_module(
        campaign_id: impl Into<String>,
        module_id: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            campaign_id: campaign_id.into(),
            module_id: Some(module_id.into()),
            title: title.into(),
            doc_type: None,
            content: None,
        }
    }

    /// Set the document type.
    pub fn with_type(mut self, doc_type: impl Into<String>) -> Self {
        self.doc_type = Some(doc_type.into());
        self
    }

    /// Set initial content.
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }
}

/// Input for updating a document.
#[derive(Debug, Clone, Default)]
pub struct UpdateDocumentInput {
    /// New title (if changing)
    pub title: Option<String>,
    /// New content (if changing)
    pub content: Option<String>,
    /// New document type (if changing)
    pub doc_type: Option<String>,
}

impl UpdateDocumentInput {
    /// Create an update to change the title.
    pub fn set_title(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            content: None,
            doc_type: None,
        }
    }

    /// Create an update to change the content.
    pub fn set_content(content: impl Into<String>) -> Self {
        Self {
            title: None,
            content: Some(content.into()),
            doc_type: None,
        }
    }

    /// Create an update to change both title and content.
    pub fn set_title_and_content(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            content: Some(content.into()),
            doc_type: None,
        }
    }
}

/// Service for document management.
///
/// Handles document CRUD operations for campaign and module markdown content.
pub struct DocumentService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> DocumentService<'a> {
    /// Create a new document service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Create a new document.
    ///
    /// Use this for creating ad-hoc user documents. Template-based documents
    /// are created automatically by CampaignService and ModuleService.
    pub fn create(&mut self, input: CreateDocumentInput) -> ServiceResult<Document> {
        let doc_id = Uuid::new_v4().to_string();
        let doc_type = input.doc_type.as_deref().unwrap_or("user_document");
        let content = input.content.as_deref().unwrap_or("");

        let new_doc = if let Some(ref module_id) = input.module_id {
            NewDocument::for_module(&doc_id, &input.campaign_id, module_id, &input.title, doc_type)
                .with_content(content)
        } else {
            NewDocument::for_campaign(&doc_id, &input.campaign_id, &input.title, doc_type)
                .with_content(content)
        };

        dal::insert_document(self.conn, &new_doc)?;
        dal::get_document(self.conn, &doc_id).map_err(ServiceError::from)
    }

    /// List all campaign-level documents (not in any module).
    pub fn list_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<Vec<Document>> {
        dal::list_campaign_level_documents(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// List all documents for a specific module.
    pub fn list_for_module(&mut self, module_id: &str) -> ServiceResult<Vec<Document>> {
        dal::list_module_documents(self.conn, module_id).map_err(ServiceError::from)
    }

    /// List documents by type for a campaign.
    pub fn list_by_type(
        &mut self,
        campaign_id: &str,
        doc_type: &str,
    ) -> ServiceResult<Vec<Document>> {
        dal::list_documents_by_type(self.conn, campaign_id, doc_type).map_err(ServiceError::from)
    }

    /// Get a document by ID.
    ///
    /// Returns `None` if the document doesn't exist.
    pub fn get(&mut self, id: &str) -> ServiceResult<Option<Document>> {
        dal::get_document_optional(self.conn, id).map_err(ServiceError::from)
    }

    /// Update a document.
    ///
    /// Returns the updated document, or an error if not found.
    pub fn update(&mut self, id: &str, input: UpdateDocumentInput) -> ServiceResult<Document> {
        let now = Utc::now().to_rfc3339();

        // Build the update changeset
        let title_ref = input.title.as_deref();
        let content_ref = input.content.as_deref();
        let doc_type_ref = input.doc_type.as_deref();

        let update = DalUpdateDocument {
            title: title_ref,
            content: content_ref,
            doc_type: doc_type_ref,
            module_id: None, // Not changing module assignment
            updated_at: Some(&now),
        };

        let rows = dal::update_document(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Document", id));
        }

        dal::get_document(self.conn, id).map_err(ServiceError::from)
    }

    /// Delete a document permanently.
    pub fn delete(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_document(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Document", id));
        }

        Ok(())
    }

    /// Move a document to a different module.
    pub fn move_to_module(&mut self, id: &str, module_id: &str) -> ServiceResult<Document> {
        let now = Utc::now().to_rfc3339();
        let update = DalUpdateDocument::move_to_module(module_id, &now);

        let rows = dal::update_document(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Document", id));
        }

        dal::get_document(self.conn, id).map_err(ServiceError::from)
    }

    /// Move a document out of a module to campaign level.
    pub fn move_to_campaign(&mut self, id: &str) -> ServiceResult<Document> {
        let now = Utc::now().to_rfc3339();
        let update = DalUpdateDocument::move_to_campaign(&now);

        let rows = dal::update_document(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Document", id));
        }

        dal::get_document(self.conn, id).map_err(ServiceError::from)
    }

    /// Check if a document exists.
    pub fn exists(&mut self, id: &str) -> ServiceResult<bool> {
        dal::document_exists(self.conn, id).map_err(ServiceError::from)
    }

    /// Count documents for a campaign (all documents, including module docs).
    pub fn count_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<i64> {
        dal::count_campaign_documents(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Count documents for a module.
    pub fn count_for_module(&mut self, module_id: &str) -> ServiceResult<i64> {
        dal::count_module_documents(self.conn, module_id).map_err(ServiceError::from)
    }

    /// Search documents using full-text search.
    pub fn search(
        &mut self,
        campaign_id: &str,
        query: &str,
    ) -> ServiceResult<Vec<dal::DocumentSearchResult>> {
        dal::search_documents(self.conn, campaign_id, query).map_err(ServiceError::from)
    }

    /// Search documents within a specific module.
    pub fn search_in_module(
        &mut self,
        module_id: &str,
        query: &str,
    ) -> ServiceResult<Vec<dal::DocumentSearchResult>> {
        dal::search_module_documents(self.conn, module_id, query).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::campaign::{insert_campaign, insert_module};
    use crate::models::campaign::{NewCampaign, NewModule};
    use crate::test_utils::setup_test_db;

    fn create_test_campaign(conn: &mut SqliteConnection) -> String {
        let campaign_id = Uuid::new_v4().to_string();
        let campaign = NewCampaign::new(&campaign_id, "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");
        campaign_id
    }

    fn create_test_module(conn: &mut SqliteConnection, campaign_id: &str) -> String {
        let module_id = Uuid::new_v4().to_string();
        let module = NewModule::new(&module_id, campaign_id, "Test Module", 1);
        insert_module(conn, &module).expect("Failed to create module");
        module_id
    }

    #[test]
    fn test_create_campaign_document() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = DocumentService::new(&mut conn);

        let input = CreateDocumentInput::for_campaign(&campaign_id, "My Notes");
        let doc = service.create(input).expect("Failed to create document");

        assert_eq!(doc.title, "My Notes");
        assert_eq!(doc.campaign_id, campaign_id);
        assert!(doc.module_id.is_none());
        assert_eq!(doc.doc_type, "user_document");
        assert!(doc.content.is_empty());
    }

    #[test]
    fn test_create_module_document() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = DocumentService::new(&mut conn);

        let input = CreateDocumentInput::for_module(&campaign_id, &module_id, "Room Description");
        let doc = service.create(input).expect("Failed to create document");

        assert_eq!(doc.title, "Room Description");
        assert_eq!(doc.module_id, Some(module_id));
    }

    #[test]
    fn test_create_document_with_content() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = DocumentService::new(&mut conn);

        let input = CreateDocumentInput::for_campaign(&campaign_id, "Session Notes")
            .with_type("session")
            .with_content("# Session 1\n\nThe party gathered...");
        let doc = service.create(input).expect("Failed to create document");

        assert_eq!(doc.doc_type, "session");
        assert!(doc.content.contains("Session 1"));
    }

    #[test]
    fn test_list_for_campaign() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = DocumentService::new(&mut conn);

        // Create campaign-level and module-level documents
        let input1 = CreateDocumentInput::for_campaign(&campaign_id, "Campaign Doc 1");
        let input2 = CreateDocumentInput::for_campaign(&campaign_id, "Campaign Doc 2");
        let input3 = CreateDocumentInput::for_module(&campaign_id, &module_id, "Module Doc");
        service.create(input1).expect("Failed to create document");
        service.create(input2).expect("Failed to create document");
        service.create(input3).expect("Failed to create document");

        // list_for_campaign should only return campaign-level docs
        let docs = service
            .list_for_campaign(&campaign_id)
            .expect("Failed to list documents");
        assert_eq!(docs.len(), 2);
    }

    #[test]
    fn test_list_for_module() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = DocumentService::new(&mut conn);

        let input1 = CreateDocumentInput::for_module(&campaign_id, &module_id, "Module Doc 1");
        let input2 = CreateDocumentInput::for_module(&campaign_id, &module_id, "Module Doc 2");
        let input3 = CreateDocumentInput::for_campaign(&campaign_id, "Campaign Doc");
        service.create(input1).expect("Failed to create document");
        service.create(input2).expect("Failed to create document");
        service.create(input3).expect("Failed to create document");

        let docs = service
            .list_for_module(&module_id)
            .expect("Failed to list documents");
        assert_eq!(docs.len(), 2);
    }

    #[test]
    fn test_get_document() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = DocumentService::new(&mut conn);

        let input = CreateDocumentInput::for_campaign(&campaign_id, "Test Doc");
        let created = service.create(input).expect("Failed to create document");

        let retrieved = service
            .get(&created.id)
            .expect("Failed to get document")
            .expect("Document not found");

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.title, "Test Doc");
    }

    #[test]
    fn test_get_document_not_found() {
        let mut conn = setup_test_db();

        let mut service = DocumentService::new(&mut conn);

        let result = service.get("nonexistent").expect("Failed to query document");
        assert!(result.is_none());
    }

    #[test]
    fn test_update_document_title() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = DocumentService::new(&mut conn);

        let input = CreateDocumentInput::for_campaign(&campaign_id, "Original Title");
        let created = service.create(input).expect("Failed to create document");

        let update = UpdateDocumentInput::set_title("New Title");
        let updated = service
            .update(&created.id, update)
            .expect("Failed to update document");

        assert_eq!(updated.title, "New Title");
    }

    #[test]
    fn test_update_document_content() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = DocumentService::new(&mut conn);

        let input = CreateDocumentInput::for_campaign(&campaign_id, "Test Doc");
        let created = service.create(input).expect("Failed to create document");

        let update = UpdateDocumentInput::set_content("# New Content\n\nUpdated!");
        let updated = service
            .update(&created.id, update)
            .expect("Failed to update document");

        assert!(updated.content.contains("New Content"));
    }

    #[test]
    fn test_update_document_not_found() {
        let mut conn = setup_test_db();

        let mut service = DocumentService::new(&mut conn);

        let update = UpdateDocumentInput::set_title("New Title");
        let result = service.update("nonexistent", update);

        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_delete_document() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = DocumentService::new(&mut conn);

        let input = CreateDocumentInput::for_campaign(&campaign_id, "Test Doc");
        let created = service.create(input).expect("Failed to create document");

        assert!(service.exists(&created.id).expect("Failed to check exists"));

        service
            .delete(&created.id)
            .expect("Failed to delete document");

        assert!(!service.exists(&created.id).expect("Failed to check exists"));
    }

    #[test]
    fn test_delete_document_not_found() {
        let mut conn = setup_test_db();

        let mut service = DocumentService::new(&mut conn);

        let result = service.delete("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_move_document_to_module() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = DocumentService::new(&mut conn);

        // Create campaign-level document
        let input = CreateDocumentInput::for_campaign(&campaign_id, "Test Doc");
        let created = service.create(input).expect("Failed to create document");
        assert!(created.module_id.is_none());

        // Move to module
        let moved = service
            .move_to_module(&created.id, &module_id)
            .expect("Failed to move document");
        assert_eq!(moved.module_id, Some(module_id));
    }

    #[test]
    fn test_move_document_to_campaign() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = DocumentService::new(&mut conn);

        // Create module document
        let input = CreateDocumentInput::for_module(&campaign_id, &module_id, "Test Doc");
        let created = service.create(input).expect("Failed to create document");
        assert!(created.module_id.is_some());

        // Move to campaign level
        let moved = service
            .move_to_campaign(&created.id)
            .expect("Failed to move document");
        assert!(moved.module_id.is_none());
    }

    #[test]
    fn test_count_documents() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let module_id = create_test_module(&mut conn, &campaign_id);

        let mut service = DocumentService::new(&mut conn);

        let input1 = CreateDocumentInput::for_campaign(&campaign_id, "Doc 1");
        let input2 = CreateDocumentInput::for_campaign(&campaign_id, "Doc 2");
        let input3 = CreateDocumentInput::for_module(&campaign_id, &module_id, "Module Doc");
        service.create(input1).expect("Failed to create document");
        service.create(input2).expect("Failed to create document");
        service.create(input3).expect("Failed to create document");

        assert_eq!(
            service
                .count_for_campaign(&campaign_id)
                .expect("Failed to count"),
            3
        );
        assert_eq!(
            service
                .count_for_module(&module_id)
                .expect("Failed to count"),
            1
        );
    }

    #[test]
    fn test_list_by_type() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = DocumentService::new(&mut conn);

        let input1 =
            CreateDocumentInput::for_campaign(&campaign_id, "Session 1").with_type("session");
        let input2 =
            CreateDocumentInput::for_campaign(&campaign_id, "Session 2").with_type("session");
        let input3 = CreateDocumentInput::for_campaign(&campaign_id, "NPC List").with_type("npc");
        service.create(input1).expect("Failed to create document");
        service.create(input2).expect("Failed to create document");
        service.create(input3).expect("Failed to create document");

        let sessions = service
            .list_by_type(&campaign_id, "session")
            .expect("Failed to list by type");
        assert_eq!(sessions.len(), 2);

        let npcs = service
            .list_by_type(&campaign_id, "npc")
            .expect("Failed to list by type");
        assert_eq!(npcs.len(), 1);
    }
}
