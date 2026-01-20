//! Feat catalog service.
//!
//! Provides database-backed feat search, retrieval, and import functionality.
//! Supports filtering by name, prerequisites, and source.

use crate::error::Result;
use crate::models::catalog::{CatalogFeat, Feat, FeatData, FeatFilters, FeatSummary, NewCatalogFeat};
use crate::schema::catalog_feats;
use crate::services::CatalogService;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Service for searching and managing feats in the catalog.
///
/// This service follows the stateful pattern with a borrowed database connection.
/// It implements `CatalogService` for generic catalog operations.
///
/// # Example
///
/// ```ignore
/// use mimir_dm_core::services::FeatService;
/// use mimir_dm_core::models::catalog::FeatFilters;
///
/// let mut service = FeatService::new(&mut conn);
/// let filters = FeatFilters::default();
/// let feats = service.search(filters)?;
/// ```
pub struct FeatService<'a> {
    /// Database connection
    pub conn: &'a mut SqliteConnection,
}

impl<'a> FeatService<'a> {
    /// Create a new FeatService with a database connection.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Search feats with optional filters.
    ///
    /// # Arguments
    /// * `filters` - Search criteria including name pattern, sources, and prerequisites
    ///
    /// # Returns
    /// * `Ok(Vec<FeatSummary>)` - List of matching feat summaries
    /// * `Err(DbError)` - If the database query fails
    pub fn search_feats(&mut self, filters: FeatFilters) -> Result<Vec<FeatSummary>> {
        debug!("Searching feats with filters: {:?}", filters);

        let mut query = catalog_feats::table.into_boxed();

        // Apply search pattern filter
        if let Some(search_pattern) = filters.search_pattern.clone() {
            if !search_pattern.is_empty() {
                let pattern = format!("%{}%", search_pattern.to_lowercase());
                query = query.filter(
                    catalog_feats::name
                        .like(pattern.clone())
                        .or(catalog_feats::prerequisites.like(pattern.clone()))
                        .or(catalog_feats::brief.like(pattern)),
                );
            }
        }

        // Apply source filter
        if let Some(sources) = filters.sources {
            if !sources.is_empty() {
                query = query.filter(catalog_feats::source.eq_any(sources));
            }
        }

        // Apply has_prerequisites filter
        if let Some(has_prerequisites) = filters.has_prerequisites {
            if has_prerequisites {
                query = query.filter(catalog_feats::prerequisites.is_not_null());
            } else {
                query = query.filter(catalog_feats::prerequisites.is_null());
            }
        }

        let feats = query
            .select(CatalogFeat::as_select())
            .load::<CatalogFeat>(self.conn)?;

        Ok(feats.iter().map(FeatSummary::from).collect())
    }

    /// Get a specific feat by name and source.
    ///
    /// # Arguments
    /// * `name` - Exact name of the feat
    /// * `source` - Source book code (e.g., "PHB", "XGE")
    ///
    /// # Returns
    /// * `Ok(Some(Feat))` - The full feat data if found
    /// * `Ok(None)` - If not found
    /// * `Err(DbError)` - If database query fails
    pub fn get_feat_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> Result<Option<Feat>> {
        debug!("Getting feat by name: {} and source: {}", name, source);

        let catalog_feat: Option<CatalogFeat> = catalog_feats::table
            .filter(
                catalog_feats::name
                    .eq(name)
                    .and(catalog_feats::source.eq(source)),
            )
            .select(CatalogFeat::as_select())
            .first::<CatalogFeat>(self.conn)
            .optional()?;

        match catalog_feat {
            Some(feat) => {
                let full_feat: Feat = serde_json::from_str(&feat.full_feat_json)?;
                Ok(Some(full_feat))
            }
            None => Ok(None),
        }
    }

    /// Get all distinct source books that contain feats.
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - Sorted list of source book codes
    /// * `Err(DbError)` - If the database query fails
    pub fn get_feat_sources(&mut self) -> Result<Vec<String>> {
        catalog_feats::table
            .select(catalog_feats::source)
            .distinct()
            .order(catalog_feats::source.asc())
            .load::<String>(self.conn)
            .map_err(Into::into)
    }

    /// Get total count of feats in the catalog.
    ///
    /// # Returns
    /// * `Ok(i64)` - Total number of feats
    /// * `Err(DbError)` - If the database query fails
    pub fn get_feat_count(&mut self) -> Result<i64> {
        catalog_feats::table
            .count()
            .get_result::<i64>(self.conn)
            .map_err(Into::into)
    }

    /// Import all feat data from an uploaded book directory.
    ///
    /// Scans the `feats/` subdirectory for JSON files and imports each feat.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `book_dir` - Path to the book directory containing feat data
    /// * `source` - Source book code to assign to imported feats
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of feats imported
    /// * `Err(DbError)` - If reading files or database operations fail
    pub fn import_feats_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        info!(
            "Importing feats from book directory: {:?} (source: {})",
            book_dir, source
        );
        let mut imported_count = 0;

        let feats_dir = book_dir.join("feats");
        if !feats_dir.exists() || !feats_dir.is_dir() {
            debug!("No feats directory found in book: {:?}", book_dir);
            return Ok(0);
        }

        info!("Found feats directory: {:?}", feats_dir);

        // Read all JSON files in the feats directory
        let entries = fs::read_dir(&feats_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            debug!("Processing feat file: {:?}", path);

            let content = fs::read_to_string(&path)?;
            let feat_data: FeatData = serde_json::from_str(&content)?;

            if let Some(feats) = feat_data.feat {
                let new_feats: Vec<NewCatalogFeat> = feats
                    .into_iter()
                    .map(|mut feat| {
                        feat.source = source.to_string();
                        NewCatalogFeat::from(&feat)
                    })
                    .collect();

                if !new_feats.is_empty() {
                    let inserted = diesel::insert_into(catalog_feats::table)
                        .values(&new_feats)
                        .execute(conn)?;

                    imported_count += inserted;
                    info!("Imported {} feats from {:?}", inserted, path);
                }
            }
        }

        info!(
            "Successfully imported {} feats from source: {}",
            imported_count, source
        );
        Ok(imported_count)
    }

    /// Remove all feats from a specific source.
    ///
    /// Used when removing a book from the library to clean up its catalog data.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `source` - Source book code to remove feats from
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of feats deleted
    /// * `Err(DbError)` - If the database operation fails
    pub fn remove_feats_by_source(conn: &mut SqliteConnection, source: &str) -> Result<usize> {
        info!("Removing feats from source: {}", source);

        let deleted = diesel::delete(catalog_feats::table)
            .filter(catalog_feats::source.eq(source))
            .execute(conn)?;

        info!("Removed {} feats from source: {}", deleted, source);
        Ok(deleted)
    }
}

impl<'a> CatalogService for FeatService<'a> {
    type Filters = FeatFilters;
    type Summary = FeatSummary;
    type Full = Feat;

    fn search(&mut self, filters: Self::Filters) -> Result<Vec<Self::Summary>> {
        self.search_feats(filters)
    }

    fn get_by_name_and_source(&mut self, name: &str, source: &str) -> Result<Option<Self::Full>> {
        self.get_feat_by_name_and_source(name, source)
    }

    fn get_sources(&mut self) -> Result<Vec<String>> {
        self.get_feat_sources()
    }
}
