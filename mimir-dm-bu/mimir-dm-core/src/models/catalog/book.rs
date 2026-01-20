//! Book tracking models for uploaded archives

use crate::schema::uploaded_books;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Database model for tracking uploaded book archives
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = uploaded_books)]
pub struct UploadedBook {
    pub id: String,                    // Book ID (e.g., "PHB", "DMG")
    pub name: String,                  // Display name (e.g., "Player's Handbook")
    pub location: String,              // Full path to extracted directory
    pub archive_path: String,          // Full path to stored .tar.gz file
    pub uploaded_at: String,           // ISO timestamp when uploaded
    pub metadata_json: Option<String>, // Full metadata.json content for reference
}

/// Insertable model for new uploaded books
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = uploaded_books)]
pub struct NewUploadedBook {
    pub id: String,
    pub name: String,
    pub location: String,
    pub archive_path: String,
    pub uploaded_at: String,
    pub metadata_json: Option<String>,
}
