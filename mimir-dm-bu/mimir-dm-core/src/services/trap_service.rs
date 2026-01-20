//! Trap and hazard catalog service.
//!
//! Provides database-backed trap and hazard search, retrieval, and import functionality.
//! Supports filtering by name, source, category, and trap type.

use crate::error::Result;
use crate::models::catalog::{
    CatalogTrap, HazardData, NewCatalogTrap, TrapData, TrapFilters, TrapOrHazard, TrapSummary,
};
use crate::schema::catalog_traps;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

/// Service for searching and managing traps and hazards in the catalog.
pub struct TrapService;

impl TrapService {
    /// Searches traps and hazards with the given filters.
    pub fn search_traps(
        &self,
        conn: &mut SqliteConnection,
        filters: TrapFilters,
    ) -> Result<Vec<TrapSummary>> {
        debug!("Searching traps with filters: {:?}", filters);

        let mut query = catalog_traps::table.into_boxed();

        // Apply search filter
        if let Some(search) = &filters.search {
            if !search.is_empty() {
                let search_pattern = format!("%{}%", search);
                query = query.filter(catalog_traps::name.like(search_pattern));
            }
        }

        // Apply source filter
        if let Some(sources) = &filters.sources {
            if !sources.is_empty() {
                query = query.filter(catalog_traps::source.eq_any(sources));
            }
        }

        // Apply category filter (Trap vs Hazard)
        if let Some(categories) = &filters.categories {
            if !categories.is_empty() {
                query = query.filter(catalog_traps::category.eq_any(categories));
            }
        }

        // Apply trap type filter
        if let Some(trap_types) = &filters.trap_types {
            if !trap_types.is_empty() {
                query = query.filter(catalog_traps::trap_type.eq_any(trap_types));
            }
        }

        let results: Vec<CatalogTrap> = query.order(catalog_traps::name.asc()).load(conn)?;

        // Convert to TrapSummary
        let summaries: Vec<TrapSummary> = results
            .iter()
            .map(|trap| TrapSummary {
                name: trap.name.clone(),
                source: trap.source.clone(),
                trap_type: trap
                    .trap_type
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
                category: trap.category.clone(),
            })
            .collect();

        info!("Found {} traps matching filters", summaries.len());
        Ok(summaries)
    }

    /// Gets trap details by name and source.
    pub fn get_trap_details(
        &self,
        conn: &mut SqliteConnection,
        name: String,
        source: String,
    ) -> Result<Option<CatalogTrap>> {
        debug!("Getting trap details for: {} from {}", name, source);

        catalog_traps::table
            .filter(catalog_traps::name.eq(name))
            .filter(catalog_traps::source.eq(source))
            .first(conn)
            .optional()
            .map_err(Into::into)
    }

    /// Gets all unique source books containing traps.
    pub fn get_trap_sources(&self, conn: &mut SqliteConnection) -> Result<Vec<String>> {
        let sources: Vec<String> = catalog_traps::table
            .select(catalog_traps::source)
            .distinct()
            .order(catalog_traps::source.asc())
            .load(conn)?;

        Ok(sources)
    }

    /// Gets the total count of traps in the catalog.
    pub fn get_trap_count(&self, conn: &mut SqliteConnection) -> Result<i64> {
        catalog_traps::table
            .count()
            .get_result(conn)
            .map_err(Into::into)
    }

    /// Gets all unique trap types.
    pub fn get_trap_types(&self, conn: &mut SqliteConnection) -> Result<Vec<String>> {
        let types: Vec<Option<String>> = catalog_traps::table
            .select(catalog_traps::trap_type)
            .distinct()
            .load(conn)?;

        let types: Vec<String> = types.into_iter().flatten().collect();

        Ok(types)
    }

    /// Gets all unique trap categories.
    pub fn get_trap_categories(&self, conn: &mut SqliteConnection) -> Result<Vec<String>> {
        let categories: Vec<String> = catalog_traps::table
            .select(catalog_traps::category)
            .distinct()
            .order(catalog_traps::category.asc())
            .load(conn)?;

        Ok(categories)
    }

    /// Import all trap and hazard data from an uploaded book directory
    pub fn import_traps_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        info!(
            "Importing traps from book directory: {:?} (source: {})",
            book_dir, source
        );

        let mut total_imported = 0;

        // Import traps
        let traps_dir = book_dir.join("traps");
        if traps_dir.exists() && traps_dir.is_dir() {
            info!("Found traps directory: {:?}", traps_dir);
            total_imported += Self::import_traps_from_directory(conn, &traps_dir, source)?;
        } else {
            debug!("No traps directory found in book: {:?}", book_dir);
        }

        // Import hazards
        let hazards_dir = book_dir.join("hazards");
        if hazards_dir.exists() && hazards_dir.is_dir() {
            info!("Found hazards directory: {:?}", hazards_dir);
            total_imported += Self::import_hazards_from_directory(conn, &hazards_dir, source)?;
        } else {
            debug!("No hazards directory found in book: {:?}", book_dir);
        }

        info!(
            "Successfully imported {} total traps/hazards from {}",
            total_imported, source
        );
        Ok(total_imported)
    }

    /// Import traps from a directory
    fn import_traps_from_directory(
        conn: &mut SqliteConnection,
        traps_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        let mut total_imported = 0;

        for entry in fs::read_dir(traps_dir)? {
            let entry = entry?;
            let file_path = entry.path();

            if !file_path.is_file()
                || file_path.extension().and_then(|e| e.to_str()) != Some("json")
            {
                continue;
            }

            debug!(
                "Processing trap file: {:?}",
                file_path.file_name().unwrap_or_default()
            );

            match Self::import_traps_from_file(conn, &file_path, source) {
                Ok(count) => {
                    info!("Imported {} traps from {:?}", count, file_path);
                    total_imported += count;
                }
                Err(e) => {
                    error!("Failed to import traps from {:?}: {}", file_path, e);
                }
            }
        }

        Ok(total_imported)
    }

    /// Import hazards from a directory
    fn import_hazards_from_directory(
        conn: &mut SqliteConnection,
        hazards_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        let mut total_imported = 0;

        for entry in fs::read_dir(hazards_dir)? {
            let entry = entry?;
            let file_path = entry.path();

            if !file_path.is_file()
                || file_path.extension().and_then(|e| e.to_str()) != Some("json")
            {
                continue;
            }

            debug!(
                "Processing hazard file: {:?}",
                file_path.file_name().unwrap_or_default()
            );

            match Self::import_hazards_from_file(conn, &file_path, source) {
                Ok(count) => {
                    info!("Imported {} hazards from {:?}", count, file_path);
                    total_imported += count;
                }
                Err(e) => {
                    error!("Failed to import hazards from {:?}: {}", file_path, e);
                }
            }
        }

        Ok(total_imported)
    }

    /// Import traps from a single JSON file
    fn import_traps_from_file(
        conn: &mut SqliteConnection,
        file_path: &Path,
        _source: &str,
    ) -> Result<usize> {
        debug!("Reading trap file: {:?}", file_path);

        let content = fs::read_to_string(file_path)?;

        let trap_data: TrapData = serde_json::from_str(&content)?;

        if let Some(traps) = trap_data.trap {
            let traps: Vec<TrapOrHazard> = traps.into_iter().map(TrapOrHazard::Trap).collect();
            let new_traps: Vec<NewCatalogTrap> = traps.iter().map(NewCatalogTrap::from).collect();

            debug!(
                "Inserting {} traps individually (SQLite limitation)",
                new_traps.len()
            );

            for trap in &new_traps {
                let result = diesel::insert_into(catalog_traps::table)
                    .values(trap)
                    .on_conflict((catalog_traps::name, catalog_traps::source))
                    .do_nothing()
                    .execute(conn);

                if let Err(e) = result {
                    warn!("Failed to insert trap {}: {}", trap.name, e);
                }
            }

            info!(
                "Successfully imported {} traps into database",
                new_traps.len()
            );
            Ok(new_traps.len())
        } else {
            Ok(0)
        }
    }

    /// Import hazards from a single JSON file
    fn import_hazards_from_file(
        conn: &mut SqliteConnection,
        file_path: &Path,
        _source: &str,
    ) -> Result<usize> {
        debug!("Reading hazard file: {:?}", file_path);

        let content = fs::read_to_string(file_path)?;

        let hazard_data: HazardData = serde_json::from_str(&content)?;

        if let Some(hazards) = hazard_data.hazard {
            let hazards: Vec<TrapOrHazard> =
                hazards.into_iter().map(TrapOrHazard::Hazard).collect();
            let new_hazards: Vec<NewCatalogTrap> =
                hazards.iter().map(NewCatalogTrap::from).collect();

            debug!(
                "Inserting {} hazards individually (SQLite limitation)",
                new_hazards.len()
            );

            for hazard in &new_hazards {
                let result = diesel::insert_into(catalog_traps::table)
                    .values(hazard)
                    .on_conflict((catalog_traps::name, catalog_traps::source))
                    .do_nothing()
                    .execute(conn);

                if let Err(e) = result {
                    warn!("Failed to insert hazard {}: {}", hazard.name, e);
                }
            }

            info!(
                "Successfully imported {} hazards into database",
                new_hazards.len()
            );
            Ok(new_hazards.len())
        } else {
            Ok(0)
        }
    }

    /// Remove all traps from a specific source
    pub fn remove_traps_from_source(conn: &mut SqliteConnection, source: &str) -> Result<usize> {
        info!("Removing traps from source: {}", source);

        let deleted = diesel::delete(catalog_traps::table.filter(catalog_traps::source.eq(source)))
            .execute(conn)?;

        info!("Removed {} traps from source: {}", deleted, source);
        Ok(deleted)
    }
}
