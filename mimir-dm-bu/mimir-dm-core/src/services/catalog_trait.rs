//! Generic trait for catalog services.
//!
//! This module defines a common interface for all catalog services (spells, monsters,
//! items, etc.), enabling generic code and reducing duplication across the 20+ catalog
//! service implementations.

use crate::error::Result;

/// Trait for catalog services that provide search and retrieval operations.
///
/// All catalog services follow a similar pattern:
/// - Search with filters returning summaries
/// - Get full entity by name and source
/// - Get list of available sources for filtering
///
/// This trait captures that common interface, allowing generic implementations
/// and reducing code duplication.
///
/// # Type Parameters
///
/// Implementations must specify three associated types:
/// - `Filters`: The filter struct used for search queries (e.g., `SpellFilters`)
/// - `Summary`: The lightweight type returned from searches (e.g., `SpellSummary`)
/// - `Full`: The complete entity type with all details (e.g., `Spell`)
///
/// # Example
///
/// ```ignore
/// use mimir_dm_core::services::{CatalogService, MonsterService};
/// use mimir_dm_core::models::catalog::monster::MonsterFilters;
///
/// let mut service = MonsterService::new(conn);
/// let filters = MonsterFilters::default();
/// let monsters = service.search(filters)?;
/// ```
pub trait CatalogService {
    /// The filter type used for search queries.
    ///
    /// Should implement `Default` to allow searching with no filters.
    type Filters: Default;

    /// The summary type returned from search results.
    ///
    /// This is typically a lightweight struct containing just the fields
    /// needed for list views (name, source, key identifying info).
    type Summary;

    /// The full entity type returned from detail queries.
    ///
    /// This contains all available data about the entity, typically
    /// including the parsed JSON data from the source books.
    type Full;

    /// Search the catalog with optional filters.
    ///
    /// Performs a database query against the catalog table, applying
    /// any provided filters. Results are typically sorted alphabetically
    /// by name.
    ///
    /// # Arguments
    ///
    /// * `filters` - Search criteria specific to this catalog type.
    ///   Use `Default::default()` to search with no filters.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Summary>)` - List of matching summaries
    /// * `Err(DbError)` - If the database query fails
    fn search(&mut self, filters: Self::Filters) -> Result<Vec<Self::Summary>>;

    /// Get a single entity by its name and source.
    ///
    /// This is the primary lookup method for retrieving full entity details.
    /// The combination of name + source uniquely identifies entities in the
    /// catalog (the same spell/monster/item name can appear in multiple books).
    ///
    /// # Arguments
    ///
    /// * `name` - The entity name to look up (case-sensitive)
    /// * `source` - The source book identifier (e.g., "PHB", "MM")
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Full))` - The full entity if found
    /// * `Ok(None)` - If no entity matches the name/source combination
    /// * `Err(DbError)` - If the database query fails
    fn get_by_name_and_source(&mut self, name: &str, source: &str) -> Result<Option<Self::Full>>;

    /// Get all unique source identifiers in this catalog.
    ///
    /// Returns a sorted list of all source book identifiers that have
    /// entities in this catalog. Used to populate source filter dropdowns
    /// in the UI.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - Sorted list of unique source identifiers
    /// * `Err(DbError)` - If the database query fails
    fn get_sources(&mut self) -> Result<Vec<String>>;
}
