//! Document model for tracking campaign documents

use crate::schema::documents;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a document instance created from a template or user
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = documents)]
pub struct Document {
    pub id: i32,
    pub campaign_id: i32,
    pub module_id: Option<i32>,
    pub session_id: Option<i32>,
    pub template_id: String,
    pub document_type: String,
    pub title: String,
    pub file_path: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    /// File type: 'markdown', 'png', 'jpg', 'webp', 'gif', 'svg'
    pub file_type: String,
    /// Whether this document was created by the user (not from a template)
    pub is_user_created: bool,
}

/// New document to be inserted
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = documents)]
pub struct NewDocument {
    pub campaign_id: i32,
    pub module_id: Option<i32>,
    pub session_id: Option<i32>,
    pub template_id: String,
    pub document_type: String,
    pub title: String,
    pub file_path: String,
    pub file_type: String,
    pub is_user_created: bool,
}

/// Update existing document
#[derive(Debug, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = documents)]
#[derive(Default)]
pub struct UpdateDocument {
    pub title: Option<String>,
    pub updated_at: Option<String>,
    pub completed_at: Option<String>,
}

impl Document {
    /// Check if the document has been completed
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Get the level of the document (campaign, module, session, or handout)
    pub fn level(&self) -> DocumentLevel {
        // Check if this is a handout based on document_type
        if self.document_type == "handout" {
            DocumentLevel::Handout
        } else if self.session_id.is_some() {
            DocumentLevel::Session
        } else if self.module_id.is_some() {
            DocumentLevel::Module
        } else {
            DocumentLevel::Campaign
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentLevel {
    Campaign,
    Module,
    Session,
    Handout,
}

impl DocumentLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocumentLevel::Campaign => "campaign",
            DocumentLevel::Module => "module",
            DocumentLevel::Session => "session",
            DocumentLevel::Handout => "handout",
        }
    }
}

/// Supported file types for documents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Markdown,
    Png,
    Jpg,
    Webp,
    Gif,
    Svg,
}

impl FileType {
    /// Parse file type from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" => Some(Self::Markdown),
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpg),
            "webp" => Some(Self::Webp),
            "gif" => Some(Self::Gif),
            "svg" => Some(Self::Svg),
            _ => None,
        }
    }

    /// Check if this is an image type
    pub fn is_image(&self) -> bool {
        !matches!(self, Self::Markdown)
    }

    /// Get the string representation for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Webp => "webp",
            Self::Gif => "gif",
            Self::Svg => "svg",
        }
    }

    /// Get the MIME type for this file type
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Markdown => "text/markdown",
            Self::Png => "image/png",
            Self::Jpg => "image/jpeg",
            Self::Webp => "image/webp",
            Self::Gif => "image/gif",
            Self::Svg => "image/svg+xml",
        }
    }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for FileType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(Self::Markdown),
            "png" => Ok(Self::Png),
            "jpg" | "jpeg" => Ok(Self::Jpg),
            "webp" => Ok(Self::Webp),
            "gif" => Ok(Self::Gif),
            "svg" => Ok(Self::Svg),
            _ => Err(format!("Unknown file type: {}", s)),
        }
    }
}
