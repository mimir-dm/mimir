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

/// `GET /books`: a book or an adventure that the reader can open
/// (MIMIR-T-0736).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookInfo {
    /// The key of `/books/{source}`: a source code, or an adventure's id.
    pub source: String,
    pub name: String,
    /// "book" or "adventure".
    pub kind: String,
    /// An adventure's level range.
    #[serde(default)]
    pub level_start: Option<i64>,
    #[serde(default)]
    pub level_end: Option<i64>,
    /// An adventure's storyline (e.g. "Starter Set").
    #[serde(default)]
    pub storyline: Option<String>,
}

/// `POST /catalog/import-homebrew`: what a homebrew file brought.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomebrewImport {
    pub sources: Vec<String>,
    /// Catalog entries (monsters, items, spells, …).
    pub entries: usize,
    pub adventures: usize,
    pub books: usize,
}

/// `GET /books/{source}/plan`: an adventure as a campaign, before it is
/// made (MIMIR-T-0739).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdventurePlan {
    pub source: String,
    pub name: String,
    pub parts: Vec<AdventurePart>,
}

/// A part of an adventure (a chapter, an appendix) in the plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdventurePart {
    pub index: usize,
    pub name: String,
    /// "module" or "document": what it becomes unless the DM chooses.
    pub role: String,
    /// Its documents as a module.
    pub documents: Vec<String>,
    /// Its square-grid maps (they become Mimir maps).
    pub maps: Vec<String>,
    /// The creatures it names (they become module monsters).
    pub creatures: usize,
}

/// `POST /books/{source}/campaign`: the DM's choices.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AdventureCampaignRequest {
    /// The campaign's name; empty: the adventure's.
    #[serde(default)]
    pub name: String,
    /// A choice per part; a part not listed takes its role in the plan.
    #[serde(default)]
    pub parts: Vec<PartChoice>,
}

/// What one part becomes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartChoice {
    pub index: usize,
    /// "module", "document" or "skip".
    pub role: String,
    /// A module in one document instead of one per section.
    #[serde(default)]
    pub one_document: bool,
}

/// What the conversion made.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdventureCampaign {
    pub campaign_id: String,
    pub modules: usize,
    pub documents: usize,
    pub maps: usize,
    pub monsters: usize,
    /// What could not come: map images not in the catalog images, creatures
    /// not in the catalog.
    pub missing: Vec<String>,
}

/// `GET /books/{source}`: a book's data (sections are `data[i]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookData {
    pub source: String,
    pub name: String,
    pub data: Value,
}
