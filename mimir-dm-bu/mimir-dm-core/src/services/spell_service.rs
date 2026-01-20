//! Database service layer for spell operations
//!
//! This service provides database-backed spell search and retrieval,
//! replacing the in-memory catalog system.

use crate::error::Result;
use crate::models::catalog::{
    CatalogSpell, ClassReference, Classes, NewCatalogSpell, Spell, SpellData, SpellFilters,
    SpellSummary,
};
use crate::schema::catalog_spells;
use crate::services::CatalogService;
use diesel::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

/// Spell class data from sources.json.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SpellSourceEntry {
    /// Class references for the spell.
    pub class: Option<Vec<ClassReference>>,
}

/// Map of source -> spell name -> class data.
pub type SpellSources = HashMap<String, HashMap<String, SpellSourceEntry>>;

/// Service for searching and managing spells in the catalog.
pub struct SpellService;

impl SpellService {
    /// Search spells with optional filters.
    ///
    /// Queries the catalog_spells table with the provided filter criteria.
    /// Results are returned sorted by name.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `filters` - Search criteria including name, levels, schools, sources, tags
    ///
    /// # Returns
    /// * `Ok(Vec<SpellSummary>)` - List of matching spell summaries
    /// * `Err(DbError)` - If the database query fails
    pub fn search_spells(
        conn: &mut SqliteConnection,
        filters: SpellFilters,
    ) -> Result<Vec<SpellSummary>> {
        debug!("Searching spells with filters: {:?}", filters);

        let mut query = catalog_spells::table.into_boxed();

        // Apply name search filter
        if let Some(name_query) = &filters.query {
            if !name_query.trim().is_empty() {
                query = query.filter(catalog_spells::name.like(format!("%{}%", name_query.trim())));
            }
        }

        // Apply level filters
        if !filters.levels.is_empty() {
            query = query.filter(catalog_spells::level.eq_any(&filters.levels));
        }

        // Apply school filters
        if !filters.schools.is_empty() {
            query = query.filter(catalog_spells::school.eq_any(&filters.schools));
        }

        // Apply source filters
        if !filters.sources.is_empty() {
            query = query.filter(catalog_spells::source.eq_any(&filters.sources));
        }

        // Apply tag filters (requires JSON containment check)
        if !filters.tags.is_empty() {
            for tag in &filters.tags {
                // SQLite doesn't have native JSON operators, so we use LIKE
                query = query.filter(catalog_spells::tags.like(format!("%\"{}\"%%", tag)));
            }
        }

        // Apply class filters (search in full_spell_json for class names)
        if !filters.classes.is_empty() {
            for class_name in &filters.classes {
                // Search for class name in the fromClassList array within the JSON
                // Pattern matches: "name": "Wizard" (case-insensitive would be better but SQLite LIKE is case-insensitive for ASCII)
                query = query.filter(
                    catalog_spells::full_spell_json.like(format!("%\"name\":\"{}\"%", class_name))
                );
            }
        }

        // Apply pagination
        if let Some(offset) = filters.offset {
            query = query.offset(offset as i64);
        }

        // Apply limit only if explicitly requested
        if let Some(limit) = filters.limit {
            query = query.limit(limit as i64);
        }

        // Execute query with explicit select
        let catalog_spells: Vec<CatalogSpell> =
            query.select(CatalogSpell::as_select()).load(conn)?;

        let summaries: Vec<SpellSummary> = catalog_spells
            .iter()
            .map(|spell| spell.to_summary())
            .collect();

        info!("Found {} spells matching search criteria", summaries.len());
        Ok(summaries)
    }

    /// Get detailed spell information by name and source.
    ///
    /// Retrieves the full spell data including description, components,
    /// casting time, and all other spell properties.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `name` - Exact spell name to look up
    /// * `source` - Source book code (e.g., "PHB", "XGE")
    ///
    /// # Returns
    /// * `Ok(Some(Spell))` - Full spell data if found
    /// * `Ok(None)` - If no spell matches name and source
    /// * `Err(DbError)` - If the database query fails
    pub fn get_spell_details(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<Spell>> {
        debug!("Getting spell details: {} from {}", name, source);

        let catalog_spell: Option<CatalogSpell> = catalog_spells::table
            .filter(catalog_spells::name.eq(name))
            .filter(catalog_spells::source.eq(source))
            .select(CatalogSpell::as_select())
            .first(conn)
            .optional()?;

        if let Some(spell_record) = catalog_spell {
            // Parse the full JSON spell data
            let spell: Spell = serde_json::from_str(&spell_record.full_spell_json)?;

            debug!("Found spell details for: {}", name);
            Ok(Some(spell))
        } else {
            debug!(
                "No spell found with name '{}' from source '{}'",
                name, source
            );
            Ok(None)
        }
    }

    /// Get unique spell sources for filter dropdown.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of unique source codes, sorted alphabetically
    pub fn get_spell_sources(conn: &mut SqliteConnection) -> Result<Vec<String>> {
        debug!("Getting unique spell sources");

        let sources: Vec<String> = catalog_spells::table
            .select(catalog_spells::source)
            .distinct()
            .order(catalog_spells::source)
            .load(conn)?;

        debug!("Found {} unique spell sources", sources.len());
        Ok(sources)
    }

    /// Get unique spell schools for filter dropdown.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of unique schools (e.g., "Evocation", "Necromancy")
    pub fn get_spell_schools(conn: &mut SqliteConnection) -> Result<Vec<String>> {
        debug!("Getting unique spell schools");

        let schools: Vec<String> = catalog_spells::table
            .select(catalog_spells::school)
            .distinct()
            .order(catalog_spells::school)
            .load(conn)?;

        debug!("Found {} unique spell schools", schools.len());
        Ok(schools)
    }

    /// Get spell count by source for statistics.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    ///
    /// # Returns
    /// * `Ok(Vec<(String, i64)>)` - List of (source, count) tuples sorted by source
    pub fn get_spell_count_by_source(conn: &mut SqliteConnection) -> Result<Vec<(String, i64)>> {
        debug!("Getting spell count by source");

        use diesel::dsl::count_star;

        let counts: Vec<(String, i64)> = catalog_spells::table
            .group_by(catalog_spells::source)
            .select((catalog_spells::source, count_star()))
            .order(catalog_spells::source)
            .load(conn)?;

        debug!("Found spell counts for {} sources", counts.len());
        Ok(counts)
    }

    /// Get total spell count.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    ///
    /// # Returns
    /// * `Ok(i64)` - Total number of spells in the database
    pub fn get_total_spell_count(conn: &mut SqliteConnection) -> Result<i64> {
        debug!("Getting total spell count");

        use diesel::dsl::count_star;

        let count: i64 = catalog_spells::table.select(count_star()).first(conn)?;

        debug!("Total spells in database: {}", count);
        Ok(count)
    }

    /// Import all spell data from an uploaded book directory.
    ///
    /// Scans the book directory for spell JSON files, parses them, and inserts
    /// the spells into the catalog_spells table. Handles multiple file formats
    /// including spells-*.json and book-*.json files. Merges class associations
    /// from sources.json if available.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `book_dir` - Path to the extracted book directory
    /// * `source` - Source code for the book (e.g., "PHB")
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of spells successfully imported
    /// * `Err(DbError)` - If file reading or database operations fail
    pub fn import_spells_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        info!(
            "Importing spells from book directory: {:?} (source: {})",
            book_dir, source
        );

        let mut total_imported = 0;
        let spell_files = Self::find_spell_files(book_dir)?;

        if spell_files.is_empty() {
            info!("No spell files found in book directory");
            return Ok(0);
        }

        // Load spell-class associations from sources.json if it exists
        let spell_sources = Self::load_spell_sources(book_dir);
        if spell_sources.is_some() {
            info!("Loaded spell-class associations from sources.json");
        } else {
            info!("No sources.json found, spells will not have class associations");
        }

        info!("Found {} spell files to process", spell_files.len());

        for spell_file in spell_files {
            debug!("Processing spell file: {:?}", spell_file);

            match Self::import_spells_from_file(conn, &spell_file, source, spell_sources.as_ref()) {
                Ok(count) => {
                    info!("Imported {} spells from {:?}", count, spell_file);
                    total_imported += count;
                }
                Err(e) => {
                    error!("Failed to import spells from {:?}: {}", spell_file, e);
                    // Continue processing other files instead of failing completely
                }
            }
        }

        info!(
            "Successfully imported {} total spells from {}",
            total_imported, source
        );
        Ok(total_imported)
    }

    /// Load spell-class associations from sources.json
    fn load_spell_sources(book_dir: &Path) -> Option<SpellSources> {
        info!("Looking for sources.json, book_dir: {:?}", book_dir);

        // Look for sources.json in common locations
        // Also check parent directories since sources.json is shared across books
        let mut possible_paths = vec![
            book_dir.join("data/spells/sources.json"),
            book_dir.join("spells/sources.json"),
            book_dir.join("sources.json"),
        ];

        // Check parent directory (for shared sources.json)
        if let Some(parent) = book_dir.parent() {
            possible_paths.push(parent.join("sources.json"));
            possible_paths.push(parent.join("spells/sources.json"));
        }

        for path in possible_paths {
            if path.exists() {
                debug!("Found sources.json at: {:?}", path);
                match fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<SpellSources>(&content) {
                        Ok(sources) => {
                            let total_spells: usize = sources.values().map(|m| m.len()).sum();
                            debug!("Loaded {} spell entries from sources.json", total_spells);
                            return Some(sources);
                        }
                        Err(e) => {
                            warn!("Failed to parse sources.json: {}", e);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to read sources.json: {}", e);
                    }
                }
            }
        }
        None
    }

    /// Find all spell JSON files in a book directory
    fn find_spell_files(book_dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut spell_files = Vec::new();

        // Common locations for spell data
        let search_dirs = vec![
            book_dir.join("data"),
            book_dir.join("spells"),
            book_dir.join("book"),
            book_dir.to_path_buf(),
        ];

        for dir in search_dirs {
            if !dir.exists() {
                continue;
            }

            debug!("Searching for spell files in: {:?}", dir);

            // Look for spell-specific files and general book files
            let file_patterns = vec!["spells-*.json", "spell*.json", "book-*.json"];

            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_file() {
                        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                        // Check if file matches any of our patterns
                        for pattern in &file_patterns {
                            if Self::matches_pattern(file_name, pattern) {
                                debug!("Found potential spell file: {:?}", path);
                                spell_files.push(path);
                                break;
                            }
                        }
                    }
                }
            }
        }

        spell_files.sort();
        spell_files.dedup();
        Ok(spell_files)
    }

    /// Import spells from a single JSON file
    fn import_spells_from_file(
        conn: &mut SqliteConnection,
        file_path: &Path,
        source: &str,
        spell_sources: Option<&SpellSources>,
    ) -> Result<usize> {
        debug!("Reading spell file: {:?}", file_path);

        let content = fs::read_to_string(file_path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        let mut spells_to_import = Vec::new();

        // Try to parse as SpellData structure first (spells-*.json files)
        if let Ok(spell_data) = serde_json::from_value::<SpellData>(json_value.clone()) {
            debug!(
                "Parsed as SpellData, found {} spells",
                spell_data.spell.len()
            );

            for spell in spell_data.spell {
                spells_to_import.push(spell);
            }
        }
        // Try to extract spells from book structure (book-*.json files)
        else if let Some(spell_array) = json_value.get("spell").and_then(|v| v.as_array()) {
            debug!(
                "Found spell array in book structure with {} entries",
                spell_array.len()
            );

            for spell_value in spell_array {
                match serde_json::from_value::<Spell>(spell_value.clone()) {
                    Ok(spell) => spells_to_import.push(spell),
                    Err(e) => {
                        warn!("Failed to parse spell from book structure: {}", e);
                        continue;
                    }
                }
            }
        }
        // Check for nested data structure
        else if let Some(data_array) = json_value.get("data").and_then(|v| v.as_array()) {
            debug!("Checking nested data structure");

            for data_section in data_array {
                if let Some(spell_array) = data_section.get("spell").and_then(|v| v.as_array()) {
                    debug!(
                        "Found nested spell array with {} entries",
                        spell_array.len()
                    );

                    for spell_value in spell_array {
                        match serde_json::from_value::<Spell>(spell_value.clone()) {
                            Ok(spell) => spells_to_import.push(spell),
                            Err(e) => {
                                warn!("Failed to parse nested spell: {}", e);
                                continue;
                            }
                        }
                    }
                }
            }
        }

        if spells_to_import.is_empty() {
            debug!("No spells found in file: {:?}", file_path);
            return Ok(0);
        }

        debug!(
            "Processing {} spells for database import",
            spells_to_import.len()
        );

        // Transform spells to database format, merging class data from sources.json
        let catalog_spells: Vec<NewCatalogSpell> = spells_to_import
            .into_iter()
            .map(|mut spell| {
                // Look up class data from sources.json
                if let Some(sources) = spell_sources {
                    if let Some(source_spells) = sources.get(&spell.source) {
                        if let Some(entry) = source_spells.get(&spell.name) {
                            if let Some(class_list) = &entry.class {
                                // Merge class data into spell
                                spell.classes = Some(Classes {
                                    from_class_list: Some(class_list.clone()),
                                    from_subclass: None,
                                });
                                debug!(
                                    "Added {} class associations for spell '{}'",
                                    class_list.len(),
                                    spell.name
                                );
                            }
                        }
                    }
                }
                NewCatalogSpell::from_spell(spell, source)
            })
            .collect();

        // Batch insert spells
        Self::batch_insert_spells(conn, catalog_spells)
    }

    /// Batch insert spells into the database
    fn batch_insert_spells(
        conn: &mut SqliteConnection,
        spells: Vec<NewCatalogSpell>,
    ) -> Result<usize> {
        if spells.is_empty() {
            return Ok(0);
        }

        debug!("Batch inserting {} spells into database", spells.len());

        // Use INSERT OR IGNORE to handle duplicates gracefully
        let inserted = diesel::insert_or_ignore_into(catalog_spells::table)
            .values(&spells)
            .execute(conn)?;

        debug!(
            "Successfully inserted {} spells (duplicates ignored)",
            inserted
        );
        Ok(inserted)
    }

    /// Remove all spells from a specific source.
    ///
    /// Used when removing a book from the library to clean up its catalog data.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `source` - Source code of the book to remove (e.g., "PHB")
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of spells deleted
    pub fn remove_spells_by_source(conn: &mut SqliteConnection, source: &str) -> Result<usize> {
        info!("Removing all spells from source: {}", source);

        let deleted =
            diesel::delete(catalog_spells::table.filter(catalog_spells::source.eq(source)))
                .execute(conn)?;

        info!("Removed {} spells from source: {}", deleted, source);
        Ok(deleted)
    }

    /// Simple pattern matching helper for file name patterns
    fn matches_pattern(filename: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return filename.starts_with(parts[0]) && filename.ends_with(parts[1]);
            }
        }
        filename == pattern
    }
}

/// Stateful wrapper around SpellService for implementing CatalogService trait.
///
/// SpellService uses static methods for all operations. This wrapper provides
/// a stateful interface that holds the database connection, enabling trait
/// implementation while maintaining backward compatibility with existing code.
pub struct SpellServiceStateful<'a> {
    /// Database connection reference.
    pub conn: &'a mut SqliteConnection,
}

impl<'a> SpellServiceStateful<'a> {
    /// Creates a new stateful spell service with the given database connection.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }
}

impl<'a> CatalogService for SpellServiceStateful<'a> {
    type Filters = SpellFilters;
    type Summary = SpellSummary;
    type Full = Spell;

    fn search(&mut self, filters: Self::Filters) -> Result<Vec<Self::Summary>> {
        SpellService::search_spells(self.conn, filters)
    }

    fn get_by_name_and_source(&mut self, name: &str, source: &str) -> Result<Option<Self::Full>> {
        SpellService::get_spell_details(self.conn, name, source)
    }

    fn get_sources(&mut self) -> Result<Vec<String>> {
        SpellService::get_spell_sources(self.conn)
    }
}
