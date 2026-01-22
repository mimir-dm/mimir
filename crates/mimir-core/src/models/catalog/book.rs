//! Book Model
//!
//! Represents book content from 5etools (PHB, DMG, etc. with full chapter content).

use crate::schema::books;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A book with its full content (chapters, sections, entries).
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = books)]
pub struct Book {
    /// Auto-increment ID
    pub id: Option<i32>,
    /// Source code (e.g., "PHB", "DMG") - unique
    pub source: String,
    /// Display name (e.g., "Player's Handbook")
    pub name: String,
    /// Full book content as JSON (array of sections with entries)
    pub data: String,
    /// Table of contents as JSON (from books.json contents field)
    pub contents: Option<String>,
    /// Path to cover image (local asset path)
    pub cover_path: Option<String>,
}

/// Data for inserting a new book.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = books)]
pub struct NewBook<'a> {
    pub source: &'a str,
    pub name: &'a str,
    pub data: &'a str,
    pub contents: Option<&'a str>,
    pub cover_path: Option<&'a str>,
}

impl<'a> NewBook<'a> {
    /// Create a new book entry.
    ///
    /// # Arguments
    /// * `source` - Source code (e.g., "PHB")
    /// * `name` - Display name (e.g., "Player's Handbook")
    /// * `data` - Full book content JSON
    pub fn new(source: &'a str, name: &'a str, data: &'a str) -> Self {
        Self {
            source,
            name,
            data,
            contents: None,
            cover_path: None,
        }
    }

    /// Set the table of contents.
    pub fn with_contents(mut self, contents: &'a str) -> Self {
        self.contents = Some(contents);
        self
    }

    /// Set the cover image path.
    pub fn with_cover_path(mut self, cover_path: &'a str) -> Self {
        self.cover_path = Some(cover_path);
        self
    }
}

/// Response structure for book content (what frontend expects).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookContent {
    /// Source code
    pub id: String,
    /// Display name
    pub name: String,
    /// Parsed book sections (the data field parsed as JSON)
    pub data: serde_json::Value,
    /// Table of contents
    pub contents: Option<serde_json::Value>,
    /// Cover image path
    pub cover_path: Option<String>,
}

impl From<Book> for BookContent {
    fn from(book: Book) -> Self {
        Self {
            id: book.source,
            name: book.name,
            data: serde_json::from_str(&book.data).unwrap_or(serde_json::Value::Null),
            contents: book.contents.and_then(|c| serde_json::from_str(&c).ok()),
            cover_path: book.cover_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_book() {
        let book = NewBook::new("PHB", "Player's Handbook", "[]");
        assert_eq!(book.source, "PHB");
        assert_eq!(book.name, "Player's Handbook");
        assert_eq!(book.data, "[]");
        assert!(book.contents.is_none());
        assert!(book.cover_path.is_none());
    }

    #[test]
    fn test_new_book_with_contents() {
        let book = NewBook::new("PHB", "Player's Handbook", "[]")
            .with_contents("[{\"name\":\"Chapter 1\"}]")
            .with_cover_path("assets://catalog/covers/phb.webp");

        assert_eq!(book.contents, Some("[{\"name\":\"Chapter 1\"}]"));
        assert_eq!(book.cover_path, Some("assets://catalog/covers/phb.webp"));
    }
}
