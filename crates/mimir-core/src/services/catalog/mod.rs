//! Catalog Services
//!
//! Services for accessing D&D 5e catalog content (monsters, spells, items, etc.).

mod action;
mod background;
mod class;
mod condition;
mod feat;
mod hazard;
mod item;
mod language;
mod monster;
mod race;
mod spell;
mod trap;

pub use action::*;
pub use background::*;
pub use class::*;
pub use condition::*;
pub use feat::*;
pub use hazard::*;
pub use item::*;
pub use language::*;
pub use monster::*;
pub use race::*;
pub use spell::*;
pub use trap::*;

use crate::services::ServiceResult;

/// Trait for catalog entity services.
///
/// Provides a consistent interface for searching and retrieving catalog entities.
/// Each entity type implements this trait with its own filter and entity types.
pub trait CatalogEntityService {
    /// The full entity type returned by get operations.
    type Entity;

    /// Filter type used for search operations.
    type Filter: Default;

    /// Search entities with filters.
    ///
    /// Returns up to `DEFAULT_QUERY_LIMIT` results.
    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>>;

    /// Search entities with pagination.
    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>>;

    /// Get an entity by its database ID.
    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>>;

    /// Get an entity by name and source.
    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>>;

    /// List all distinct sources that have this entity type.
    fn list_sources(&mut self) -> ServiceResult<Vec<String>>;

    /// Count total entities.
    fn count(&mut self) -> ServiceResult<i64>;

    /// Count entities from a specific source.
    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64>;
}
