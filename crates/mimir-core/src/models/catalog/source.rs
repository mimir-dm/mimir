//! Catalog Source Model
//!
//! Represents a source book in the catalog (e.g., Player's Handbook, Monster Manual).

use crate::schema::catalog_sources;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A catalog source (book) that has been imported.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = catalog_sources)]
#[diesel(primary_key(code))]
pub struct CatalogSource {
    /// Unique source code (e.g., "PHB", "MM", "XGE")
    pub code: String,
    /// Display name (e.g., "Player's Handbook")
    pub name: String,
    /// Whether this source is enabled for display (SQLite stores as integer)
    pub enabled: i32,
    /// ISO 8601 timestamp of when this source was imported
    pub imported_at: String,
}

impl CatalogSource {
    /// Check if the source is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled != 0
    }
}

/// Data for inserting a new catalog source.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = catalog_sources)]
pub struct NewCatalogSource<'a> {
    pub code: &'a str,
    pub name: &'a str,
    pub enabled: i32,
    pub imported_at: &'a str,
}

impl<'a> NewCatalogSource<'a> {
    /// Create a new catalog source entry.
    ///
    /// # Arguments
    /// * `code` - Source code (e.g., "PHB")
    /// * `name` - Display name (e.g., "Player's Handbook")
    /// * `enabled` - Whether the source is enabled
    /// * `imported_at` - ISO 8601 timestamp
    pub fn new(code: &'a str, name: &'a str, enabled: bool, imported_at: &'a str) -> Self {
        Self {
            code,
            name,
            enabled: if enabled { 1 } else { 0 },
            imported_at,
        }
    }
}

/// Data for updating a catalog source's enabled status.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = catalog_sources)]
pub struct UpdateCatalogSource {
    pub enabled: Option<i32>,
}

impl UpdateCatalogSource {
    /// Create an update to change the enabled status.
    pub fn set_enabled(enabled: bool) -> Self {
        Self {
            enabled: Some(if enabled { 1 } else { 0 }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_catalog_source() {
        let source = NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z");
        assert_eq!(source.code, "PHB");
        assert_eq!(source.name, "Player's Handbook");
        assert_eq!(source.enabled, 1);
    }

    #[test]
    fn test_new_catalog_source_disabled() {
        let source = NewCatalogSource::new("HB", "Homebrew", false, "2024-01-20T12:00:00Z");
        assert_eq!(source.enabled, 0);
    }

    #[test]
    fn test_update_catalog_source() {
        let update = UpdateCatalogSource::set_enabled(false);
        assert_eq!(update.enabled, Some(0));

        let update = UpdateCatalogSource::set_enabled(true);
        assert_eq!(update.enabled, Some(1));
    }
}
