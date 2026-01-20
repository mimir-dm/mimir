//! Background catalog service.
//!
//! Provides database-backed background search, retrieval, and import functionality.
//! Supports filtering by name, skills, tools, features, and source.

use crate::error::Result;
use crate::models::catalog::{
    BackgroundData, BackgroundFilters, BackgroundSummary, CatalogBackground, NewCatalogBackground,
};
use crate::schema::catalog_backgrounds;
use crate::services::CatalogService;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Service for searching and managing character backgrounds in the catalog.
///
/// This service follows the stateful pattern with a borrowed database connection.
/// It implements `CatalogService` for generic catalog operations.
///
/// # Example
///
/// ```ignore
/// use mimir_dm_core::services::BackgroundService;
/// use mimir_dm_core::models::catalog::BackgroundFilters;
///
/// let mut service = BackgroundService::new(&mut conn);
/// let filters = BackgroundFilters::default();
/// let backgrounds = service.search_backgrounds(filters)?;
/// ```
pub struct BackgroundService<'a> {
    /// Database connection
    pub conn: &'a mut SqliteConnection,
}

impl<'a> BackgroundService<'a> {
    /// Create a new BackgroundService with a database connection.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Search backgrounds with optional filters.
    ///
    /// # Arguments
    /// * `filters` - Search criteria including name pattern, sources, and tool requirements
    ///
    /// # Returns
    /// * `Ok(Vec<BackgroundSummary>)` - List of matching background summaries
    /// * `Err(DbError)` - If the database query fails
    pub fn search_backgrounds(
        &mut self,
        filters: BackgroundFilters,
    ) -> Result<Vec<BackgroundSummary>> {
        debug!("Searching backgrounds with filters: {:?}", filters);

        let mut query = catalog_backgrounds::table.into_boxed();

        // Apply search pattern filter
        if let Some(search_pattern) = filters.search_pattern.clone() {
            if !search_pattern.is_empty() {
                let pattern = format!("%{}%", search_pattern.to_lowercase());
                query = query.filter(
                    catalog_backgrounds::name
                        .like(pattern.clone())
                        .or(catalog_backgrounds::skills.like(pattern.clone()))
                        .or(catalog_backgrounds::tools.like(pattern.clone()))
                        .or(catalog_backgrounds::feature.like(pattern)),
                );
            }
        }

        // Apply source filter
        if let Some(sources) = filters.sources {
            if !sources.is_empty() {
                query = query.filter(catalog_backgrounds::source.eq_any(sources));
            }
        }

        // Apply has_tools filter
        if let Some(has_tools) = filters.has_tools {
            if has_tools {
                query = query.filter(catalog_backgrounds::tools.ne("None"));
            } else {
                query = query.filter(catalog_backgrounds::tools.eq("None"));
            }
        }

        let backgrounds = query
            .select(CatalogBackground::as_select())
            .load::<CatalogBackground>(self.conn)?;

        Ok(backgrounds.iter().map(BackgroundSummary::from).collect())
    }

    /// Get a specific background by name and source.
    ///
    /// # Arguments
    /// * `name` - Exact name of the background
    /// * `source` - Source book code (e.g., "PHB", "SCAG")
    ///
    /// # Returns
    /// * `Ok(Some(CatalogBackground))` - The full background data if found
    /// * `Ok(None)` - If no background matches
    /// * `Err(DbError)` - If the database query fails
    pub fn get_background_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> Result<Option<CatalogBackground>> {
        catalog_backgrounds::table
            .filter(
                catalog_backgrounds::name
                    .eq(name)
                    .and(catalog_backgrounds::source.eq(source)),
            )
            .select(CatalogBackground::as_select())
            .first::<CatalogBackground>(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// Get all distinct source books that contain backgrounds.
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - Sorted list of source book codes
    /// * `Err(DbError)` - If the database query fails
    pub fn get_background_sources(&mut self) -> Result<Vec<String>> {
        catalog_backgrounds::table
            .select(catalog_backgrounds::source)
            .distinct()
            .order(catalog_backgrounds::source.asc())
            .load::<String>(self.conn)
            .map_err(Into::into)
    }

    /// Get total count of backgrounds in the catalog.
    ///
    /// # Returns
    /// * `Ok(i64)` - Total number of backgrounds
    /// * `Err(DbError)` - If the database query fails
    pub fn get_background_count(&mut self) -> Result<i64> {
        catalog_backgrounds::table
            .count()
            .get_result::<i64>(self.conn)
            .map_err(Into::into)
    }

    /// Import all background data from an uploaded book directory.
    ///
    /// Scans the `backgrounds/` subdirectory for JSON files and imports each background.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `book_dir` - Path to the book directory containing background data
    /// * `source` - Source book code to assign to imported backgrounds
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of backgrounds imported
    /// * `Err(DbError)` - If reading files or database operations fail
    pub fn import_backgrounds_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        info!(
            "Importing backgrounds from book directory: {:?} (source: {})",
            book_dir, source
        );
        let mut imported_count = 0;

        let backgrounds_dir = book_dir.join("backgrounds");
        if !backgrounds_dir.exists() || !backgrounds_dir.is_dir() {
            debug!("No backgrounds directory found in book: {:?}", book_dir);
            return Ok(0);
        }

        info!("Found backgrounds directory: {:?}", backgrounds_dir);

        // Read all JSON files in the backgrounds directory
        let entries = fs::read_dir(&backgrounds_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            debug!("Processing background file: {:?}", path);

            let content = fs::read_to_string(&path)?;
            let background_data: BackgroundData = serde_json::from_str(&content)?;

            if let Some(backgrounds) = background_data.background {
                let new_backgrounds: Vec<NewCatalogBackground> = backgrounds
                    .into_iter()
                    .map(|mut background| {
                        background.source = source.to_string();
                        NewCatalogBackground::from(&background)
                    })
                    .collect();

                if !new_backgrounds.is_empty() {
                    let inserted = diesel::insert_into(catalog_backgrounds::table)
                        .values(&new_backgrounds)
                        .execute(conn)?;

                    imported_count += inserted;
                    info!("Imported {} backgrounds from {:?}", inserted, path);
                }
            }
        }

        info!(
            "Successfully imported {} backgrounds from source: {}",
            imported_count, source
        );
        Ok(imported_count)
    }

    /// Remove all backgrounds from a specific source.
    ///
    /// Used when removing a book from the library to clean up its catalog data.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `source` - Source book code to remove backgrounds from
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of backgrounds deleted
    /// * `Err(DbError)` - If the database operation fails
    pub fn remove_backgrounds_by_source(
        conn: &mut SqliteConnection,
        source: &str,
    ) -> Result<usize> {
        info!("Removing backgrounds from source: {}", source);

        let deleted = diesel::delete(
            catalog_backgrounds::table.filter(catalog_backgrounds::source.eq(source)),
        )
        .execute(conn)?;

        info!("Removed {} backgrounds from source: {}", deleted, source);
        Ok(deleted)
    }
}

impl<'a> CatalogService for BackgroundService<'a> {
    type Filters = BackgroundFilters;
    type Summary = BackgroundSummary;
    type Full = CatalogBackground;

    fn search(&mut self, filters: Self::Filters) -> Result<Vec<Self::Summary>> {
        self.search_backgrounds(filters)
    }

    fn get_by_name_and_source(&mut self, name: &str, source: &str) -> Result<Option<Self::Full>> {
        self.get_background_by_name_and_source(name, source)
    }

    fn get_sources(&mut self) -> Result<Vec<String>> {
        self.get_background_sources()
    }
}
