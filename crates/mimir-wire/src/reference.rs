//! The catalog and the book reader (MIMIR-T-0723).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A row of a catalog search: its table cells, as the category's headers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogRow {
    pub name: String,
    pub source: String,
    pub cells: Vec<String>,
}

/// `GET /catalog/search/{category}`: a page of rows.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CatalogPage {
    /// After Name and Source.
    pub headers: Vec<String>,
    pub rows: Vec<CatalogRow>,
    /// More rows after these (ask with a larger offset).
    pub more: bool,
}

/// `GET /catalog/sources`: a source book of the catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceInfo {
    pub code: String,
    pub name: String,
    /// It has a readable book (not only catalog entries).
    pub book: bool,
}

/// `GET /books/{source}`: a book's data (sections are `data[i]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookData {
    pub source: String,
    pub name: String,
    pub data: Value,
}
