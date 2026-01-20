//! Document repository for managing campaign documents

use crate::{
    connection::DbConnection,
    dal::traits::DocumentRepositoryTrait,
    error::{DbError, Result},
    models::campaign::documents::{Document, NewDocument, UpdateDocument},
    schema::documents,
};
use diesel::prelude::*;

/// Repository for campaign document CRUD operations.
pub struct DocumentRepository;

impl DocumentRepository {
    /// Create a new document
    pub fn create(conn: &mut DbConnection, new_document: NewDocument) -> Result<Document> {
        diesel::insert_into(documents::table)
            .values(&new_document)
            .returning(Document::as_returning())
            .get_result(conn)
            .map_err(DbError::Query)
    }

    /// Get document by ID
    pub fn find_by_id(conn: &mut DbConnection, document_id: i32) -> Result<Document> {
        documents::table
            .find(document_id)
            .first(conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => DbError::NotFound {
                    entity_type: "Document".to_string(),
                    id: document_id.to_string(),
                },
                _ => DbError::Query(e),
            })
    }

    /// Get all documents for a campaign
    pub fn find_by_campaign(conn: &mut DbConnection, campaign_id: i32) -> Result<Vec<Document>> {
        documents::table
            .filter(documents::campaign_id.eq(campaign_id))
            .order(documents::created_at.asc())
            .load(conn)
            .map_err(DbError::Query)
    }

    /// Get all documents for a module
    pub fn find_by_module(conn: &mut DbConnection, module_id: i32) -> Result<Vec<Document>> {
        documents::table
            .filter(documents::module_id.eq(module_id))
            .order(documents::created_at.asc())
            .load(conn)
            .map_err(DbError::Query)
    }

    /// Alias for find_by_module for compatibility
    pub fn list_by_module(conn: &mut DbConnection, module_id: i32) -> Result<Vec<Document>> {
        Self::find_by_module(conn, module_id)
    }

    /// Find a document by module and template
    pub fn find_by_module_and_template(
        conn: &mut DbConnection,
        module_id: i32,
        template_id: &str,
    ) -> Result<Option<Document>> {
        documents::table
            .filter(documents::module_id.eq(module_id))
            .filter(documents::template_id.eq(template_id))
            .first(conn)
            .optional()
            .map_err(DbError::Query)
    }

    /// Get all documents for a session
    pub fn find_by_session(conn: &mut DbConnection, session_id: i32) -> Result<Vec<Document>> {
        documents::table
            .filter(documents::session_id.eq(session_id))
            .order(documents::created_at.asc())
            .load(conn)
            .map_err(DbError::Query)
    }

    /// Get all documents by template ID
    pub fn find_by_template(conn: &mut DbConnection, template_id: &str) -> Result<Vec<Document>> {
        documents::table
            .filter(documents::template_id.eq(template_id))
            .order(documents::created_at.asc())
            .load(conn)
            .map_err(DbError::Query)
    }

    /// Get all incomplete documents for a campaign
    pub fn find_incomplete_by_campaign(
        conn: &mut DbConnection,
        campaign_id: i32,
    ) -> Result<Vec<Document>> {
        documents::table
            .filter(documents::campaign_id.eq(campaign_id))
            .filter(documents::completed_at.is_null())
            .order(documents::created_at.asc())
            .load(conn)
            .map_err(DbError::Query)
    }

    /// Get all completed documents for a campaign
    pub fn find_completed_by_campaign(
        conn: &mut DbConnection,
        campaign_id: i32,
    ) -> Result<Vec<Document>> {
        documents::table
            .filter(documents::campaign_id.eq(campaign_id))
            .filter(documents::completed_at.is_not_null())
            .order(documents::created_at.asc())
            .load(conn)
            .map_err(DbError::Query)
    }

    /// Update a document
    pub fn update(
        conn: &mut DbConnection,
        document_id: i32,
        update: UpdateDocument,
    ) -> Result<Document> {
        diesel::update(documents::table.find(document_id))
            .set(&update)
            .returning(Document::as_returning())
            .get_result(conn)
            .map_err(DbError::Query)
    }

    /// Mark a document as completed
    pub fn mark_completed(conn: &mut DbConnection, document_id: i32) -> Result<Document> {
        let update = UpdateDocument {
            title: None,
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            completed_at: Some(chrono::Utc::now().to_rfc3339()),
        };

        Self::update(conn, document_id, update)
    }

    /// Delete a document
    pub fn delete(conn: &mut DbConnection, document_id: i32) -> Result<usize> {
        diesel::delete(documents::table.find(document_id))
            .execute(conn)
            .map_err(DbError::Query)
    }

    /// Check if a document exists by file path
    pub fn exists_by_path(conn: &mut DbConnection, file_path: &str) -> Result<bool> {
        let count = documents::table
            .filter(documents::file_path.eq(file_path))
            .count()
            .get_result::<i64>(conn)
            .map_err(DbError::Query)?;

        Ok(count > 0)
    }

    /// Get all handout documents for a campaign
    pub fn find_handouts_by_campaign(
        conn: &mut DbConnection,
        campaign_id: i32,
    ) -> Result<Vec<Document>> {
        documents::table
            .filter(documents::campaign_id.eq(campaign_id))
            .filter(documents::document_type.eq("handout"))
            .order(documents::created_at.asc())
            .load(conn)
            .map_err(DbError::Query)
    }
}

// =============================================================================
// Instance-based wrapper for trait implementation
// =============================================================================

/// Instance-based document repository that wraps the static methods.
///
/// This wrapper provides the same functionality as [`DocumentRepository`] but
/// implements [`DocumentRepositoryTrait`] for use with dependency injection.
pub struct DocumentRepositoryInstance<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> DocumentRepositoryInstance<'a> {
    /// Create a new document repository instance
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }
}

impl<'a> DocumentRepositoryTrait for DocumentRepositoryInstance<'a> {
    fn create(&mut self, new_document: NewDocument) -> Result<Document> {
        DocumentRepository::create(self.conn, new_document)
    }

    fn find_by_id(&mut self, document_id: i32) -> Result<Document> {
        DocumentRepository::find_by_id(self.conn, document_id)
    }

    fn find_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Document>> {
        DocumentRepository::find_by_campaign(self.conn, campaign_id)
    }

    fn find_by_module(&mut self, module_id: i32) -> Result<Vec<Document>> {
        DocumentRepository::find_by_module(self.conn, module_id)
    }

    fn find_by_module_and_template(
        &mut self,
        module_id: i32,
        template_id: &str,
    ) -> Result<Option<Document>> {
        DocumentRepository::find_by_module_and_template(self.conn, module_id, template_id)
    }

    fn find_by_session(&mut self, session_id: i32) -> Result<Vec<Document>> {
        DocumentRepository::find_by_session(self.conn, session_id)
    }

    fn find_by_template(&mut self, template_id: &str) -> Result<Vec<Document>> {
        DocumentRepository::find_by_template(self.conn, template_id)
    }

    fn find_incomplete_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Document>> {
        DocumentRepository::find_incomplete_by_campaign(self.conn, campaign_id)
    }

    fn find_completed_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Document>> {
        DocumentRepository::find_completed_by_campaign(self.conn, campaign_id)
    }

    fn update(&mut self, document_id: i32, update: UpdateDocument) -> Result<Document> {
        DocumentRepository::update(self.conn, document_id, update)
    }

    fn mark_completed(&mut self, document_id: i32) -> Result<Document> {
        DocumentRepository::mark_completed(self.conn, document_id)
    }

    fn delete(&mut self, document_id: i32) -> Result<usize> {
        DocumentRepository::delete(self.conn, document_id)
    }

    fn exists_by_path(&mut self, file_path: &str) -> Result<bool> {
        DocumentRepository::exists_by_path(self.conn, file_path)
    }

    fn find_handouts_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Document>> {
        DocumentRepository::find_handouts_by_campaign(self.conn, campaign_id)
    }
}
