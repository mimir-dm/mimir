//! Cult and boon catalog service.
//!
//! Provides database-backed cult and boon search, retrieval, and import functionality.
//! Supports filtering by name, source, category, and cult type.

use crate::error::Result;
use crate::models::catalog::cult::{
    BoonData, CatalogCult, CultBoonSummary, CultData, CultFilters, NewCatalogCult,
};
use crate::schema::catalog_cults;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Service for searching and managing cults and boons in the catalog.
pub struct CultService;

impl CultService {
    /// Searches cults and boons with the given filters.
    pub fn search_cults(
        &self,
        conn: &mut SqliteConnection,
        filters: CultFilters,
    ) -> Result<Vec<CultBoonSummary>> {
        debug!("Searching cults with filters: {:?}", filters);

        let mut query = catalog_cults::table.into_boxed();

        // Apply search filter
        if let Some(search) = &filters.name {
            if !search.is_empty() {
                let search_pattern = format!("%{}%", search);
                query = query.filter(catalog_cults::name.like(search_pattern));
            }
        }

        // Apply source filter
        if let Some(sources) = &filters.source {
            if !sources.is_empty() {
                query = query.filter(catalog_cults::source.eq_any(sources));
            }
        }

        // Apply category filter (cult vs boon)
        if let Some(categories) = &filters.category {
            if !categories.is_empty() {
                query = query.filter(catalog_cults::category.eq_any(categories));
            }
        }

        // Apply cult type filter (Diabolical, Demonic, Elder Evil, etc.)
        if let Some(cult_types) = &filters.cult_type {
            if !cult_types.is_empty() {
                query = query.filter(catalog_cults::cult_type.eq_any(cult_types));
            }
        }

        let results: Vec<CatalogCult> = query
            .select(CatalogCult::as_select())
            .order(catalog_cults::name.asc())
            .load(conn)?;

        // Convert to CultBoonSummary
        let summaries: Vec<CultBoonSummary> = results.iter().map(CultBoonSummary::from).collect();

        info!("Found {} cults/boons matching filters", summaries.len());
        Ok(summaries)
    }

    /// Gets cult details by name and source.
    pub fn get_cult_details(
        &self,
        conn: &mut SqliteConnection,
        name: String,
        source: String,
    ) -> Result<Option<CatalogCult>> {
        debug!("Getting cult details for: {} from {}", name, source);

        catalog_cults::table
            .select(CatalogCult::as_select())
            .filter(catalog_cults::name.eq(name))
            .filter(catalog_cults::source.eq(source))
            .first(conn)
            .optional()
            .map_err(Into::into)
    }

    /// Gets all unique source books containing cults.
    pub fn get_cult_sources(&self, conn: &mut SqliteConnection) -> Result<Vec<String>> {
        let sources: Vec<String> = catalog_cults::table
            .select(catalog_cults::source)
            .distinct()
            .order(catalog_cults::source.asc())
            .load(conn)?;

        Ok(sources)
    }

    /// Gets the total count of cults in the catalog.
    pub fn get_cult_count(&self, conn: &mut SqliteConnection) -> Result<i64> {
        catalog_cults::table
            .count()
            .get_result(conn)
            .map_err(Into::into)
    }

    /// Gets all unique cult types.
    pub fn get_cult_types(&self, conn: &mut SqliteConnection) -> Result<Vec<String>> {
        let types: Vec<Option<String>> = catalog_cults::table
            .select(catalog_cults::cult_type)
            .distinct()
            .load(conn)?;

        let types: Vec<String> = types.into_iter().flatten().collect();

        Ok(types)
    }

    /// Gets all unique cult categories.
    pub fn get_cult_categories(&self, conn: &mut SqliteConnection) -> Result<Vec<String>> {
        let categories: Vec<String> = catalog_cults::table
            .select(catalog_cults::category)
            .distinct()
            .order(catalog_cults::category.asc())
            .load(conn)?;

        Ok(categories)
    }

    /// Import cults from a book directory
    pub fn import_cults_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        info!(
            "Importing cults from book directory: {:?} (source: {})",
            book_dir, source
        );
        let mut total_imported = 0;

        // Import cults
        let cults_dir = book_dir.join("cults");
        if cults_dir.exists() {
            info!("Found cults directory: {:?}", cults_dir);
            let cult_entries = fs::read_dir(&cults_dir)?;

            for entry in cult_entries {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    debug!(
                        "Processing cult file: {:?}",
                        path.file_name().unwrap_or_default()
                    );
                    let count = Self::import_cults_from_file(conn, &path, source)?;
                    info!("Imported {} cults from {:?}", count, path);
                    total_imported += count;
                }
            }
        }

        // Import boons from dedicated boons directory
        let boons_dir = book_dir.join("boons");
        if boons_dir.exists() {
            info!("Found boons directory: {:?}", boons_dir);
            let boon_entries = fs::read_dir(&boons_dir)?;

            for entry in boon_entries {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    debug!(
                        "Processing boon file: {:?}",
                        path.file_name().unwrap_or_default()
                    );
                    let count = Self::import_boons_from_file(conn, &path, source)?;
                    info!("Imported {} boons from {:?}", count, path);
                    total_imported += count;
                }
            }
        }

        info!(
            "Successfully imported {} total cults/boons from {}",
            total_imported, source
        );
        Ok(total_imported)
    }

    fn import_cults_from_file(
        conn: &mut SqliteConnection,
        file_path: &Path,
        source: &str,
    ) -> Result<usize> {
        debug!("Reading cult file: {:?}", file_path);

        let content = fs::read_to_string(file_path)?;
        let cult_data: CultData = serde_json::from_str(&content)?;

        if let Some(cults) = cult_data.cult {
            let new_cults: Vec<NewCatalogCult> = cults
                .iter()
                .map(|cult| {
                    let mut new_cult = NewCatalogCult::from(cult);
                    if new_cult.source.is_empty() {
                        new_cult.source = source.to_string();
                    }
                    new_cult
                })
                .collect();

            debug!(
                "Inserting {} cults individually (SQLite limitation)",
                new_cults.len()
            );

            for cult in &new_cults {
                diesel::insert_into(catalog_cults::table)
                    .values(cult)
                    .on_conflict((catalog_cults::name, catalog_cults::source))
                    .do_nothing()
                    .execute(conn)?;
            }

            info!(
                "Successfully imported {} cults into database",
                new_cults.len()
            );
            Ok(new_cults.len())
        } else {
            Ok(0)
        }
    }

    fn import_boons_from_file(
        conn: &mut SqliteConnection,
        file_path: &Path,
        source: &str,
    ) -> Result<usize> {
        debug!("Reading boon file: {:?}", file_path);

        let content = fs::read_to_string(file_path)?;
        let boon_data: BoonData = serde_json::from_str(&content)?;

        if let Some(boons) = boon_data.boon {
            let new_boons: Vec<NewCatalogCult> = boons
                .iter()
                .map(|boon| {
                    let mut new_boon = NewCatalogCult::from(boon);
                    if new_boon.source.is_empty() {
                        new_boon.source = source.to_string();
                    }
                    new_boon
                })
                .collect();

            debug!(
                "Inserting {} boons individually (SQLite limitation)",
                new_boons.len()
            );

            for boon in &new_boons {
                diesel::insert_into(catalog_cults::table)
                    .values(boon)
                    .on_conflict((catalog_cults::name, catalog_cults::source))
                    .do_nothing()
                    .execute(conn)?;
            }

            info!(
                "Successfully imported {} boons into database",
                new_boons.len()
            );
            Ok(new_boons.len())
        } else {
            Ok(0)
        }
    }

    /// Remove all cults/boons from a specific source
    pub fn remove_cults_from_source(conn: &mut SqliteConnection, source: &str) -> Result<usize> {
        info!("Removing cults/boons from source: {}", source);

        let deleted = diesel::delete(catalog_cults::table.filter(catalog_cults::source.eq(source)))
            .execute(conn)?;

        info!("Removed {} cults/boons from source: {}", deleted, source);
        Ok(deleted)
    }
}
