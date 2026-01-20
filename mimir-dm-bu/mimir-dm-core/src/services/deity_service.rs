//! Deity catalog service.
//!
//! Provides database-backed deity search, retrieval, and import functionality.
//! Supports filtering by name, pantheon, alignment, domains, and source.

use crate::error::Result;
use crate::models::catalog::deity::{
    CatalogDeity, Deity, DeityData, DeityFilters, DeitySummary, NewCatalogDeity,
};
use crate::schema::catalog_deities;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Service for searching and managing deities in the catalog.
pub struct DeityService<'a> {
    /// Database connection reference.
    pub conn: &'a mut SqliteConnection,
}

impl<'a> DeityService<'a> {
    /// Creates a new deity service with the given database connection.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Search deities with filters
    pub fn search_deities(&mut self, filters: DeityFilters) -> Result<Vec<DeitySummary>> {
        use crate::schema::catalog_deities::dsl::*;

        let mut query = catalog_deities.into_boxed();

        // Filter by name (partial match)
        if let Some(name_filter) = &filters.name {
            if !name_filter.is_empty() {
                let search_pattern = format!("%{}%", name_filter.to_lowercase());
                query = query.filter(name.like(search_pattern));
            }
        }

        // Filter by sources
        if let Some(source_filters) = &filters.sources {
            if !source_filters.is_empty() {
                query = query.filter(source.eq_any(source_filters));
            }
        }

        // Filter by pantheons
        if let Some(pantheon_filters) = &filters.pantheons {
            if !pantheon_filters.is_empty() {
                query = query.filter(pantheon.eq_any(pantheon_filters));
            }
        }

        // Filter by alignments
        if let Some(alignment_filters) = &filters.alignments {
            if !alignment_filters.is_empty() {
                query = query.filter(alignment.eq_any(alignment_filters));
            }
        }

        // Filter by domains (partial match in comma-separated string)
        if let Some(domain_filters) = &filters.domains {
            if !domain_filters.is_empty() {
                for domain in domain_filters {
                    let search_pattern = format!("%{}%", domain);
                    query = query.filter(domains.like(search_pattern));
                }
            }
        }

        let deities = query
            .limit(super::DEFAULT_QUERY_LIMIT)
            .load::<CatalogDeity>(self.conn)?;

        Ok(deities.iter().map(DeitySummary::from).collect())
    }

    /// Get deity by name and source
    pub fn get_deity_by_name_and_source(
        &mut self,
        deity_name: &str,
        deity_source: &str,
    ) -> Result<Option<Deity>> {
        use crate::schema::catalog_deities::dsl::*;

        let catalog_deity = catalog_deities
            .filter(name.eq(deity_name))
            .filter(source.eq(deity_source))
            .first::<CatalogDeity>(self.conn)
            .optional()?;

        match catalog_deity {
            Some(deity_record) => {
                let parsed_deity: Deity = serde_json::from_str(&deity_record.full_deity_json)?;
                Ok(Some(parsed_deity))
            }
            None => Ok(None),
        }
    }

    /// Get all unique pantheons for filtering
    pub fn get_all_pantheons(&mut self) -> Result<Vec<String>> {
        use crate::schema::catalog_deities::dsl::*;

        let pantheons: Vec<Option<String>> = catalog_deities
            .select(pantheon)
            .distinct()
            .filter(pantheon.is_not_null())
            .load(self.conn)?;

        let mut result: Vec<String> = pantheons
            .into_iter()
            .flatten()
            .filter(|p| !p.is_empty())
            .collect();

        result.sort();
        Ok(result)
    }

    /// Get all unique domains for filtering
    pub fn get_all_domains(&mut self) -> Result<Vec<String>> {
        use crate::schema::catalog_deities::dsl::*;

        let domain_strings: Vec<Option<String>> = catalog_deities
            .select(domains)
            .distinct()
            .filter(domains.is_not_null())
            .load(self.conn)?;

        let mut all_domains = std::collections::HashSet::new();

        // Parse comma-separated domains
        for domain_str in domain_strings.into_iter().flatten() {
            for domain in domain_str.split(',') {
                let trimmed = domain.trim();
                if !trimmed.is_empty() {
                    all_domains.insert(trimmed.to_string());
                }
            }
        }

        let mut result: Vec<String> = all_domains.into_iter().collect();
        result.sort();
        Ok(result)
    }

    /// Get all unique alignments for filtering
    pub fn get_all_alignments(&mut self) -> Result<Vec<String>> {
        use crate::schema::catalog_deities::dsl::*;

        let alignments: Vec<Option<String>> = catalog_deities
            .select(alignment)
            .distinct()
            .filter(alignment.is_not_null())
            .load(self.conn)?;

        let mut result: Vec<String> = alignments
            .into_iter()
            .flatten()
            .filter(|a| !a.is_empty())
            .collect();

        result.sort();
        Ok(result)
    }

    /// Get deity statistics by source
    pub fn get_deity_count_by_source(&mut self) -> Result<Vec<(String, i64)>> {
        use crate::schema::catalog_deities::dsl::*;

        let counts = catalog_deities
            .group_by(source)
            .select((source, diesel::dsl::count_star()))
            .load::<(String, i64)>(self.conn)?;

        Ok(counts)
    }

    /// Import all deity data from an uploaded book directory
    pub fn import_deities_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        info!(
            "Importing deities from book directory: {:?} (source: {})",
            book_dir, source
        );

        let mut total_imported = 0;
        let deity_files = Self::find_deity_files(book_dir)?;

        if deity_files.is_empty() {
            info!("No deity files found in book directory");
            return Ok(0);
        }

        info!("Found {} deity files to process", deity_files.len());

        for deity_file in deity_files {
            debug!("Processing deity file: {:?}", deity_file);

            match Self::import_deities_from_file(conn, &deity_file, source) {
                Ok(count) => {
                    info!("Imported {} deities from {:?}", count, deity_file);
                    total_imported += count;
                }
                Err(e) => {
                    debug!("Failed to import deities from {:?}: {}", deity_file, e);
                    // Continue processing other files instead of failing completely
                }
            }
        }

        info!("Total deities imported: {}", total_imported);
        Ok(total_imported)
    }

    /// Find deity files in a book directory (deities/*.json files)
    fn find_deity_files(book_dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        // Check the deities directory
        let deities_dir = book_dir.join("deities");
        if deities_dir.exists() && deities_dir.is_dir() {
            let entries = fs::read_dir(&deities_dir)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // Skip index files and foundry files
                    if filename == "index.json" || filename == "foundry.json" {
                        continue;
                    }

                    debug!("Found deity file: {:?}", path);
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    /// Import deities from a single JSON file
    fn import_deities_from_file(
        conn: &mut SqliteConnection,
        file_path: &Path,
        source: &str,
    ) -> Result<usize> {
        debug!("Reading deities from file: {:?}", file_path);

        let content = fs::read_to_string(file_path)?;
        let data: DeityData = serde_json::from_str(&content)?;

        if let Some(deities) = data.deity {
            if !deities.is_empty() {
                let new_deities: Vec<NewCatalogDeity> = deities
                    .iter()
                    .map(|deity| {
                        let mut new_deity = NewCatalogDeity::from(deity);
                        // Always override the source with the book source to ensure consistency
                        new_deity.source = source.to_string();

                        // Also update the source in the full_deity_json to maintain consistency
                        if let Ok(mut deity_json) =
                            serde_json::from_str::<serde_json::Value>(&new_deity.full_deity_json)
                        {
                            if let Some(obj) = deity_json.as_object_mut() {
                                obj.insert(
                                    "source".to_string(),
                                    serde_json::Value::String(source.to_string()),
                                );
                                if let Ok(updated_json) = serde_json::to_string(&deity_json) {
                                    new_deity.full_deity_json = updated_json;
                                }
                            }
                        }

                        new_deity
                    })
                    .collect();

                debug!(
                    "Inserting {} deities individually (SQLite limitation)",
                    new_deities.len()
                );

                for deity in &new_deities {
                    diesel::insert_into(catalog_deities::table)
                        .values(deity)
                        .on_conflict((catalog_deities::name, catalog_deities::source))
                        .do_nothing()
                        .execute(conn)?;
                }

                info!(
                    "Successfully imported {} deities into database",
                    new_deities.len()
                );
                Ok(new_deities.len())
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    /// Remove all deities from a specific source
    pub fn remove_deities_from_source(conn: &mut SqliteConnection, source: &str) -> Result<usize> {
        info!("Removing deities from source: {}", source);

        let deleted =
            diesel::delete(catalog_deities::table.filter(catalog_deities::source.eq(source)))
                .execute(conn)?;

        info!("Removed {} deities from source: {}", deleted, source);
        Ok(deleted)
    }
}
