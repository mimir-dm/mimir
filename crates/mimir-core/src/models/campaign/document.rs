//! Document Model
//!
//! Markdown content storage for campaigns and modules.

use crate::schema::documents;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A document - markdown content within a campaign.
/// Documents always belong to a campaign, optionally to a specific module.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = documents)]
pub struct Document {
    /// Unique document ID (UUID)
    pub id: String,
    /// Campaign this document belongs to
    pub campaign_id: String,
    /// Module this document belongs to (optional)
    pub module_id: Option<String>,
    /// Document title
    pub title: String,
    /// Markdown content
    pub content: String,
    /// Document type (freeform: note, session, npc, location, etc.)
    pub doc_type: String,
    /// ISO8601 timestamp of creation
    pub created_at: String,
    /// ISO8601 timestamp of last update
    pub updated_at: String,
}

impl Document {
    /// Check if this document belongs to a module.
    pub fn is_module_document(&self) -> bool {
        self.module_id.is_some()
    }

    /// Check if this is a campaign-level document (not in a module).
    pub fn is_campaign_document(&self) -> bool {
        self.module_id.is_none()
    }
}

/// Data for inserting a new document.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = documents)]
pub struct NewDocument<'a> {
    pub id: &'a str,
    pub campaign_id: &'a str,
    pub module_id: Option<&'a str>,
    pub title: &'a str,
    pub content: &'a str,
    pub doc_type: &'a str,
}

impl<'a> NewDocument<'a> {
    /// Create a new campaign-level document.
    pub fn for_campaign(
        id: &'a str,
        campaign_id: &'a str,
        title: &'a str,
        doc_type: &'a str,
    ) -> Self {
        Self {
            id,
            campaign_id,
            module_id: None,
            title,
            content: "",
            doc_type,
        }
    }

    /// Create a new module-level document.
    pub fn for_module(
        id: &'a str,
        campaign_id: &'a str,
        module_id: &'a str,
        title: &'a str,
        doc_type: &'a str,
    ) -> Self {
        Self {
            id,
            campaign_id,
            module_id: Some(module_id),
            title,
            content: "",
            doc_type,
        }
    }

    /// Set the content.
    pub fn with_content(mut self, content: &'a str) -> Self {
        self.content = content;
        self
    }
}

/// Data for updating a document.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = documents)]
pub struct UpdateDocument<'a> {
    pub title: Option<&'a str>,
    pub content: Option<&'a str>,
    pub doc_type: Option<&'a str>,
    pub module_id: Option<Option<&'a str>>,
    pub updated_at: Option<&'a str>,
}

impl<'a> UpdateDocument<'a> {
    /// Create an update to change the title.
    pub fn set_title(title: &'a str, updated_at: &'a str) -> Self {
        Self {
            title: Some(title),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Create an update to change the content.
    pub fn set_content(content: &'a str, updated_at: &'a str) -> Self {
        Self {
            content: Some(content),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Create an update to change title and content together.
    pub fn set_title_and_content(title: &'a str, content: &'a str, updated_at: &'a str) -> Self {
        Self {
            title: Some(title),
            content: Some(content),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Create an update to change the document type.
    pub fn set_doc_type(doc_type: &'a str, updated_at: &'a str) -> Self {
        Self {
            doc_type: Some(doc_type),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Create an update to move document to a module.
    pub fn move_to_module(module_id: &'a str, updated_at: &'a str) -> Self {
        Self {
            module_id: Some(Some(module_id)),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Create an update to move document out of a module (to campaign level).
    pub fn move_to_campaign(updated_at: &'a str) -> Self {
        Self {
            module_id: Some(None),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_campaign_document() {
        let doc = NewDocument::for_campaign("doc-1", "camp-1", "Session Notes", "session");
        assert_eq!(doc.id, "doc-1");
        assert_eq!(doc.campaign_id, "camp-1");
        assert!(doc.module_id.is_none());
        assert_eq!(doc.title, "Session Notes");
        assert_eq!(doc.doc_type, "session");
        assert_eq!(doc.content, "");
    }

    #[test]
    fn test_new_module_document() {
        let doc = NewDocument::for_module("doc-1", "camp-1", "mod-1", "Dungeon Map", "location");
        assert_eq!(doc.id, "doc-1");
        assert_eq!(doc.campaign_id, "camp-1");
        assert_eq!(doc.module_id, Some("mod-1"));
        assert_eq!(doc.title, "Dungeon Map");
        assert_eq!(doc.doc_type, "location");
    }

    #[test]
    fn test_new_document_with_content() {
        let doc = NewDocument::for_campaign("doc-1", "camp-1", "Notes", "note")
            .with_content("# Session 1\n\nThe party met in a tavern...");
        assert!(doc.content.contains("Session 1"));
    }

    #[test]
    fn test_update_document_title() {
        let update = UpdateDocument::set_title("New Title", "2024-01-20T12:00:00Z");
        assert_eq!(update.title, Some("New Title"));
        assert!(update.content.is_none());
        assert!(update.updated_at.is_some());
    }

    #[test]
    fn test_update_document_content() {
        let update = UpdateDocument::set_content("New content", "2024-01-20T12:00:00Z");
        assert!(update.title.is_none());
        assert_eq!(update.content, Some("New content"));
    }

    #[test]
    fn test_update_document_move_to_module() {
        let update = UpdateDocument::move_to_module("mod-1", "2024-01-20T12:00:00Z");
        assert_eq!(update.module_id, Some(Some("mod-1")));
    }

    #[test]
    fn test_update_document_move_to_campaign() {
        let update = UpdateDocument::move_to_campaign("2024-01-20T12:00:00Z");
        assert_eq!(update.module_id, Some(None));
    }
}
