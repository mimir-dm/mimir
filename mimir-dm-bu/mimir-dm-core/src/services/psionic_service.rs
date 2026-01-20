//! Psionic catalog service.
//!
//! Provides database-backed psionic search, retrieval, and import functionality.
//! Supports filtering by name, psionic type, order, and source.

use crate::error::Result;
use crate::models::catalog::{
    CatalogPsionic, NewCatalogPsionic, Psionic, PsionicFilters, PsionicSummary,
};
use crate::schema::catalog_psionics;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Service for searching and managing psionics in the catalog.
pub struct PsionicService;

impl PsionicService {
    /// Searches psionics with the given filters.
    pub fn search_psionics(
        conn: &mut SqliteConnection,
        filters: PsionicFilters,
    ) -> Result<Vec<PsionicSummary>> {
        let mut query = catalog_psionics::table.into_boxed();

        // Filter by name
        if let Some(name_filter) = &filters.name {
            query = query.filter(catalog_psionics::name.like(format!("%{}%", name_filter)));
        }

        // Filter by psionic types ("D", "T")
        if let Some(psionic_types) = &filters.psionic_types {
            if !psionic_types.is_empty() {
                query = query.filter(catalog_psionics::psionic_type.eq_any(psionic_types));
            }
        }

        // Filter by orders (Avatar, Awakened, etc.)
        if let Some(orders) = &filters.orders {
            if !orders.is_empty() {
                query = query.filter(catalog_psionics::psionic_order.eq_any(orders));
            }
        }

        // Filter by sources
        if let Some(sources) = &filters.sources {
            if !sources.is_empty() {
                query = query.filter(catalog_psionics::source.eq_any(sources));
            }
        }

        let catalog_psionics: Vec<CatalogPsionic> = query
            .select(CatalogPsionic::as_select())
            .order(catalog_psionics::name.asc())
            .load(conn)?;

        let summaries: Vec<PsionicSummary> =
            catalog_psionics.iter().map(PsionicSummary::from).collect();

        Ok(summaries)
    }

    /// Gets a psionic by its name and source book.
    pub fn get_psionic_by_name_and_source(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<Psionic>> {
        let catalog_psionic: Option<CatalogPsionic> = catalog_psionics::table
            .filter(catalog_psionics::name.eq(name))
            .filter(catalog_psionics::source.eq(source))
            .select(CatalogPsionic::as_select())
            .first(conn)
            .optional()?;

        match catalog_psionic {
            Some(cp) => {
                let psionic = serde_json::from_str::<Psionic>(&cp.full_psionic_json)?;
                Ok(Some(psionic))
            }
            None => Ok(None),
        }
    }

    /// Gets a psionic by its database ID.
    pub fn get_psionic_by_id(conn: &mut SqliteConnection, id: i32) -> Result<Option<Psionic>> {
        let catalog_psionic: Option<CatalogPsionic> = catalog_psionics::table
            .filter(catalog_psionics::id.eq(id))
            .select(CatalogPsionic::as_select())
            .first(conn)
            .optional()?;

        match catalog_psionic {
            Some(cp) => {
                let psionic = serde_json::from_str::<Psionic>(&cp.full_psionic_json)?;
                Ok(Some(psionic))
            }
            None => Ok(None),
        }
    }

    /// Gets all unique psionic types.
    pub fn get_all_psionic_types(conn: &mut SqliteConnection) -> Result<Vec<String>> {
        let types: Vec<String> = catalog_psionics::table
            .select(catalog_psionics::psionic_type)
            .distinct()
            .order(catalog_psionics::psionic_type.asc())
            .load(conn)?;

        Ok(types)
    }

    /// Gets all unique psionic orders.
    pub fn get_all_psionic_orders(conn: &mut SqliteConnection) -> Result<Vec<String>> {
        let orders: Vec<Option<String>> = catalog_psionics::table
            .select(catalog_psionics::psionic_order)
            .distinct()
            .order(catalog_psionics::psionic_order.asc())
            .load(conn)?;

        // Filter out None values and collect
        let filtered_orders: Vec<String> = orders.into_iter().flatten().collect();

        Ok(filtered_orders)
    }

    /// Gets all unique source books containing psionics.
    pub fn get_all_psionic_sources(conn: &mut SqliteConnection) -> Result<Vec<String>> {
        let sources: Vec<String> = catalog_psionics::table
            .select(catalog_psionics::source)
            .distinct()
            .order(catalog_psionics::source.asc())
            .load(conn)?;

        Ok(sources)
    }

    /// Import psionics from a book directory
    pub fn import_psionics_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        info!(
            "Importing psionics from book directory: {:?} (source: {})",
            book_dir, source
        );
        let mut total_imported = 0;

        let psionics_dir = book_dir.join("psionics");
        if psionics_dir.exists() {
            info!("Found psionics directory: {:?}", psionics_dir);
            let psionic_entries = fs::read_dir(&psionics_dir)?;

            for entry in psionic_entries {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    debug!(
                        "Processing psionic file: {:?}",
                        path.file_name().unwrap_or_default()
                    );
                    let count = Self::import_psionics_from_file(conn, &path, source)?;
                    info!("Imported {} psionics from {:?}", count, path);
                    total_imported += count;
                }
            }
        }

        info!(
            "Successfully imported {} total psionics from {}",
            total_imported, source
        );
        Ok(total_imported)
    }

    fn import_psionics_from_file(
        conn: &mut SqliteConnection,
        file_path: &Path,
        source: &str,
    ) -> Result<usize> {
        debug!("Reading psionic file: {:?}", file_path);

        let content = fs::read_to_string(file_path)?;
        let psionic_data: PsionicData = serde_json::from_str(&content)?;

        if let Some(psionics) = psionic_data.psionic {
            let new_psionics: Vec<NewCatalogPsionic> = psionics
                .iter()
                .map(|psionic| {
                    let mut new_psionic = NewCatalogPsionic::from(psionic);
                    if new_psionic.source.is_empty() {
                        new_psionic.source = source.to_string();
                    }
                    new_psionic
                })
                .collect();

            debug!(
                "Inserting {} psionics individually (SQLite limitation)",
                new_psionics.len()
            );

            for psionic in &new_psionics {
                diesel::insert_into(catalog_psionics::table)
                    .values(psionic)
                    .on_conflict((catalog_psionics::name, catalog_psionics::source))
                    .do_nothing()
                    .execute(conn)?;
            }

            info!(
                "Successfully imported {} psionics into database",
                new_psionics.len()
            );
            Ok(new_psionics.len())
        } else {
            Ok(0)
        }
    }

    /// Remove all psionics from a specific source
    pub fn remove_psionics_from_source(conn: &mut SqliteConnection, source: &str) -> Result<usize> {
        info!("Removing psionics from source: {}", source);

        let deleted =
            diesel::delete(catalog_psionics::table.filter(catalog_psionics::source.eq(source)))
                .execute(conn)?;

        info!("Removed {} psionics from source: {}", deleted, source);
        Ok(deleted)
    }
}

/// Container for JSON parsing
#[derive(Debug, serde::Deserialize)]
struct PsionicData {
    psionic: Option<Vec<Psionic>>,
}
