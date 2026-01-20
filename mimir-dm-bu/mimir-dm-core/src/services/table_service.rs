//! Table catalog service.
//!
//! Provides database-backed table search, retrieval, and import functionality.
//! Supports filtering by name, category, and source.

use crate::error::Result;
use crate::models::catalog::table::{
    CatalogTable, NewCatalogTable, Table, TableData, TableFilters, TableSummary,
};
use crate::services::CatalogService;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Service for searching and managing tables in the catalog.
pub struct TableService<'a> {
    /// Database connection reference.
    pub conn: &'a mut SqliteConnection,
}

impl<'a> TableService<'a> {
    /// Creates a new table service with the given database connection.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Searches tables with the given filters.
    pub fn search_tables(&mut self, filters: TableFilters) -> Result<Vec<TableSummary>> {
        use crate::schema::catalog_tables::dsl::*;

        let mut query = catalog_tables.into_boxed();

        // Filter by name
        if let Some(search_name) = &filters.name {
            if !search_name.is_empty() {
                query = query.filter(name.like(format!("%{}%", search_name)));
            }
        }

        // Filter by categories
        if let Some(cats) = &filters.categories {
            if !cats.is_empty() {
                query = query.filter(category.eq_any(cats));
            }
        }

        // Filter by sources
        if let Some(sources) = &filters.sources {
            if !sources.is_empty() {
                query = query.filter(source.eq_any(sources));
            }
        }

        let tables = query.order(name.asc()).load::<CatalogTable>(self.conn)?;

        Ok(tables.iter().map(TableSummary::from).collect())
    }

    /// Gets a table by its database ID.
    pub fn get_table_by_id(&mut self, table_id: i32) -> Result<Option<Table>> {
        use crate::schema::catalog_tables::dsl::*;

        let catalog_table = catalog_tables
            .find(table_id)
            .first::<CatalogTable>(self.conn)
            .optional()?;

        match catalog_table {
            Some(t) => {
                let parsed_table = serde_json::from_str(&t.full_table_json)?;
                Ok(Some(parsed_table))
            }
            None => Ok(None),
        }
    }

    /// Gets a table by its name and source book.
    pub fn get_table_by_name_and_source(
        &mut self,
        table_name: &str,
        table_source: &str,
    ) -> Result<Option<Table>> {
        use crate::schema::catalog_tables::dsl::*;

        let catalog_table = catalog_tables
            .filter(name.eq(table_name))
            .filter(source.eq(table_source))
            .first::<CatalogTable>(self.conn)
            .optional()?;

        match catalog_table {
            Some(t) => {
                let parsed_table = serde_json::from_str(&t.full_table_json)?;
                Ok(Some(parsed_table))
            }
            None => Ok(None),
        }
    }

    /// Gets all unique table categories.
    pub fn get_table_categories(&mut self) -> Result<Vec<String>> {
        use crate::schema::catalog_tables::dsl::*;

        let mut categories: Vec<String> =
            catalog_tables.select(category).distinct().load(self.conn)?;
        categories.sort();
        Ok(categories)
    }

    /// Gets all unique source books containing tables.
    pub fn get_table_sources(&mut self) -> Result<Vec<String>> {
        use crate::schema::catalog_tables::dsl::*;

        let mut sources: Vec<String> = catalog_tables.select(source).distinct().load(self.conn)?;
        sources.sort();
        Ok(sources)
    }

    /// Import all table data from an uploaded book directory
    pub fn import_tables_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        info!(
            "Importing tables from book directory: {:?} (source: {})",
            book_dir, source
        );

        let mut total_imported = 0;
        let table_files = Self::find_table_files(book_dir)?;

        if table_files.is_empty() {
            info!("No table files found in book directory");
            return Ok(0);
        }

        info!("Found {} table files to process", table_files.len());

        for table_file in table_files {
            debug!("Processing table file: {:?}", table_file);

            match Self::import_tables_from_file(conn, &table_file, source) {
                Ok(count) => {
                    info!("Imported {} tables from {:?}", count, table_file);
                    total_imported += count;
                }
                Err(e) => {
                    debug!("Failed to import tables from {:?}: {}", table_file, e);
                    // Continue processing other files instead of failing completely
                }
            }
        }

        info!(
            "Successfully imported {} total tables from {}",
            total_imported, source
        );
        Ok(total_imported)
    }

    fn find_table_files(book_dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        // Look for tables directory
        let tables_dir = book_dir.join("tables");
        if tables_dir.exists() && tables_dir.is_dir() {
            let entries = fs::read_dir(&tables_dir)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    fn import_tables_from_file(
        conn: &mut SqliteConnection,
        file_path: &Path,
        source: &str,
    ) -> Result<usize> {
        debug!("Reading table file: {:?}", file_path);

        let content = fs::read_to_string(file_path)?;
        let table_data: TableData = serde_json::from_str(&content)?;

        if let Some(tables) = table_data.table {
            let new_tables: Vec<NewCatalogTable> = tables
                .iter()
                .map(|table| {
                    let mut new_table = NewCatalogTable::from(table);
                    // Always override the source with the book source to ensure consistency
                    new_table.source = source.to_string();
                    new_table
                })
                .collect();

            debug!(
                "Inserting {} tables individually (SQLite limitation)",
                new_tables.len()
            );

            use crate::schema::catalog_tables;
            for table in &new_tables {
                diesel::insert_into(catalog_tables::table)
                    .values(table)
                    .on_conflict((catalog_tables::name, catalog_tables::source))
                    .do_nothing()
                    .execute(conn)?;
            }

            info!(
                "Successfully imported {} tables into database",
                new_tables.len()
            );
            Ok(new_tables.len())
        } else {
            Ok(0)
        }
    }

    /// Remove all tables from a specific source
    pub fn remove_tables_from_source(conn: &mut SqliteConnection, source: &str) -> Result<usize> {
        use crate::schema::catalog_tables;
        info!("Removing tables from source: {}", source);

        let deleted =
            diesel::delete(catalog_tables::table.filter(catalog_tables::source.eq(source)))
                .execute(conn)?;

        info!("Removed {} tables from source: {}", deleted, source);
        Ok(deleted)
    }
}

impl<'a> CatalogService for TableService<'a> {
    type Filters = TableFilters;
    type Summary = TableSummary;
    type Full = Table;

    fn search(&mut self, filters: Self::Filters) -> Result<Vec<Self::Summary>> {
        self.search_tables(filters)
    }

    fn get_by_name_and_source(&mut self, name: &str, source: &str) -> Result<Option<Self::Full>> {
        self.get_table_by_name_and_source(name, source)
    }

    fn get_sources(&mut self) -> Result<Vec<String>> {
        self.get_table_sources()
    }
}
