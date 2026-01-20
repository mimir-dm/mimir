//! Template document repository operations

use crate::models::campaign::template_documents::{
    NewTemplateDocument, TemplateDocument, UpdateTemplateDocument,
};
use crate::schema::template_documents;
use diesel::prelude::*;
use diesel::result::Error;
use sha2::{Digest, Sha256};

/// Repository for template document operations
pub struct TemplateRepository;

impl TemplateRepository {
    /// Create a new template document with the next version number
    pub fn create(
        conn: &mut SqliteConnection,
        mut new_template: NewTemplateDocument,
    ) -> Result<TemplateDocument, Error> {
        // Compute content hash
        let content_hash = Self::compute_hash(&new_template.document_content);
        new_template.content_hash = Some(content_hash.clone());

        // Check if the latest version has the same content
        if let Ok(latest) = Self::get_latest(conn, &new_template.document_id) {
            if latest.content_hash == content_hash {
                // Content hasn't changed from latest version, return existing
                return Ok(latest);
            }
        }

        // Get the next version number for this document_id
        let next_version = Self::get_next_version(conn, &new_template.document_id)?;
        new_template.version_number = Some(next_version);

        // Deactivate all previous versions
        Self::deactivate_all_versions(conn, &new_template.document_id)?;

        // Set this version as active
        new_template.is_active = Some(true);

        diesel::insert_into(template_documents::table)
            .values(&new_template)
            .get_result(conn)
    }

    /// Get the latest version of a template document
    pub fn get_latest(
        conn: &mut SqliteConnection,
        document_id: &str,
    ) -> Result<TemplateDocument, Error> {
        template_documents::table
            .filter(template_documents::document_id.eq(document_id))
            .filter(template_documents::is_active.eq(true))
            .order(template_documents::version_number.desc())
            .first(conn)
    }

    /// Get a specific version of a template document
    pub fn get_version(
        conn: &mut SqliteConnection,
        document_id: &str,
        version_number: i32,
    ) -> Result<TemplateDocument, Error> {
        template_documents::table
            .filter(template_documents::document_id.eq(document_id))
            .filter(template_documents::version_number.eq(version_number))
            .first(conn)
    }

    /// Get all versions of a template document
    pub fn get_all_versions(
        conn: &mut SqliteConnection,
        document_id: &str,
    ) -> Result<Vec<TemplateDocument>, Error> {
        template_documents::table
            .filter(template_documents::document_id.eq(document_id))
            .order(template_documents::version_number.desc())
            .load(conn)
    }

    /// Get all active template documents
    pub fn get_all_active(conn: &mut SqliteConnection) -> Result<Vec<TemplateDocument>, Error> {
        template_documents::table
            .filter(template_documents::is_active.eq(true))
            .order(template_documents::document_id.asc())
            .load(conn)
    }

    /// Get templates by document level
    pub fn get_by_level(
        conn: &mut SqliteConnection,
        level: &str,
    ) -> Result<Vec<TemplateDocument>, Error> {
        template_documents::table
            .filter(template_documents::document_level.eq(level))
            .filter(template_documents::is_active.eq(true))
            .order(template_documents::document_id.asc())
            .load(conn)
    }

    /// Get templates by document type
    pub fn get_by_type(
        conn: &mut SqliteConnection,
        doc_type: &str,
    ) -> Result<Vec<TemplateDocument>, Error> {
        template_documents::table
            .filter(template_documents::document_type.eq(doc_type))
            .filter(template_documents::is_active.eq(true))
            .order(template_documents::document_id.asc())
            .load(conn)
    }

    /// Update a template document (creates a new version)
    pub fn update(
        conn: &mut SqliteConnection,
        document_id: &str,
        updates: UpdateTemplateDocument,
    ) -> Result<TemplateDocument, Error> {
        // Get the latest version
        let latest = Self::get_latest(conn, document_id)?;

        // Create new content
        let new_content = updates
            .document_content
            .clone()
            .unwrap_or(latest.document_content.clone());
        let content_hash = Self::compute_hash(&new_content);

        // Check if content has changed from latest version
        if latest.content_hash == content_hash {
            // Content hasn't changed, return the latest version
            return Ok(latest);
        }

        // Create a new version based on the latest
        let new_template = NewTemplateDocument {
            document_id: document_id.to_string(),
            version_number: Some(latest.version_number + 1),
            document_content: new_content,
            content_hash: Some(content_hash),
            document_type: updates.document_type.unwrap_or(latest.document_type),
            document_level: updates.document_level.unwrap_or(latest.document_level),
            purpose: updates.purpose.unwrap_or(latest.purpose),
            variables_schema: updates.variables_schema.unwrap_or(latest.variables_schema),
            default_values: updates.default_values.unwrap_or(latest.default_values),
            is_active: Some(true),
            metadata: updates.metadata.unwrap_or(latest.metadata),
        };

        // Deactivate all previous versions
        Self::deactivate_all_versions(conn, document_id)?;

        diesel::insert_into(template_documents::table)
            .values(&new_template)
            .get_result(conn)
    }

    /// Delete all versions of a template document
    pub fn delete_all_versions(
        conn: &mut SqliteConnection,
        document_id: &str,
    ) -> Result<usize, Error> {
        diesel::delete(
            template_documents::table.filter(template_documents::document_id.eq(document_id)),
        )
        .execute(conn)
    }

    /// Delete a specific version of a template document
    pub fn delete_version(
        conn: &mut SqliteConnection,
        document_id: &str,
        version_number: i32,
    ) -> Result<usize, Error> {
        diesel::delete(
            template_documents::table
                .filter(template_documents::document_id.eq(document_id))
                .filter(template_documents::version_number.eq(version_number)),
        )
        .execute(conn)
    }

    /// Set a specific version as active
    pub fn set_active_version(
        conn: &mut SqliteConnection,
        document_id: &str,
        version_number: i32,
    ) -> Result<TemplateDocument, Error> {
        // Deactivate all versions
        Self::deactivate_all_versions(conn, document_id)?;

        // Activate the specific version
        diesel::update(
            template_documents::table
                .filter(template_documents::document_id.eq(document_id))
                .filter(template_documents::version_number.eq(version_number)),
        )
        .set(template_documents::is_active.eq(true))
        .execute(conn)?;

        Self::get_version(conn, document_id, version_number)
    }

    /// Helper: Get the next version number for a document
    fn get_next_version(conn: &mut SqliteConnection, document_id: &str) -> Result<i32, Error> {
        let max_version: Option<i32> = template_documents::table
            .filter(template_documents::document_id.eq(document_id))
            .select(diesel::dsl::max(template_documents::version_number))
            .first(conn)?;

        Ok(max_version.unwrap_or(0) + 1)
    }

    /// Helper: Deactivate all versions of a document
    fn deactivate_all_versions(
        conn: &mut SqliteConnection,
        document_id: &str,
    ) -> Result<usize, Error> {
        diesel::update(
            template_documents::table.filter(template_documents::document_id.eq(document_id)),
        )
        .set(template_documents::is_active.eq(false))
        .execute(conn)
    }

    /// Helper: Compute SHA-256 hash of content
    fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
