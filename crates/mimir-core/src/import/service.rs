//! Catalog Import Service
//!
//! Imports 5etools data into the catalog database with proper field extraction,
//! transaction handling, and FTS indexing.

use crate::dal::catalog::{self, insert_source};
use crate::fts::{flatten_entries, index_entity, ContentType};
use crate::import::{collect_source_entities, copy_images, discover_available_sources, get_token_path, CollectedEntities};
use crate::models::catalog::*;
use anyhow::{Context, Result};
use diesel::connection::SimpleConnection;
use diesel::SqliteConnection;
use flate2::read::GzDecoder;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use tar::Archive;
use tracing::{error, info, warn};

/// Result of an import operation.
#[derive(Debug, Default)]
pub struct ImportResult {
    /// Source codes that were successfully imported.
    pub sources_imported: Vec<String>,
    /// Sources that failed to import, with error messages.
    pub sources_failed: Vec<(String, String)>,
    /// Count of entities imported, by type.
    pub entity_counts: HashMap<String, usize>,
    /// Total entities imported.
    pub total_entities: usize,
    /// Count of images copied.
    pub images_copied: usize,
}

impl ImportResult {
    /// Check if the import was fully successful (no failures).
    pub fn is_success(&self) -> bool {
        self.sources_failed.is_empty()
    }

    /// Get a summary of the import.
    pub fn summary(&self) -> String {
        let mut s = format!(
            "Imported {} sources, {} failed, {} total entities, {} images\n",
            self.sources_imported.len(),
            self.sources_failed.len(),
            self.total_entities,
            self.images_copied
        );

        if !self.entity_counts.is_empty() {
            s.push_str("Entity counts:\n");
            let mut counts: Vec<_> = self.entity_counts.iter().collect();
            counts.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
            for (entity_type, count) in counts {
                s.push_str(&format!("  {}: {}\n", entity_type, count));
            }
        }

        if !self.sources_failed.is_empty() {
            s.push_str("Failed sources:\n");
            for (source, error) in &self.sources_failed {
                s.push_str(&format!("  {}: {}\n", source, error));
            }
        }

        s
    }
}

/// Default groups to include when importing.
///
/// Groups in 5etools:
/// - "core" - PHB, DMG, MM
/// - "supplement" - XGE, TCE, MPMM, etc.
/// - "setting" - Eberron, Ravnica, Theros, etc.
/// - "adventure" - Published adventures (CoS, LMoP, etc.)
/// - "screen" - DM Screen supplements
/// - "homebrew" - Homebrew content
/// - Other one-offs
const DEFAULT_ALLOWED_GROUPS: &[&str] = &["core", "supplement"];

/// Catalog import service for importing 5etools data.
pub struct CatalogImportService<'a> {
    conn: &'a mut SqliteConnection,
    /// Optional path to 5etools img directory.
    source_img_dir: Option<PathBuf>,
    /// Optional path to destination directory where images will be copied.
    dest_img_dir: Option<PathBuf>,
    /// Count of images copied during import.
    images_copied: usize,
    /// Groups to include (None = all groups, Some = only specified groups).
    allowed_groups: Option<Vec<String>>,
}

impl<'a> CatalogImportService<'a> {
    /// Create a new import service with a database connection.
    ///
    /// By default, only imports reference material (core, supplement, setting groups).
    /// Use `with_all_groups()` to import everything including adventures.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self {
            conn,
            source_img_dir: None,
            dest_img_dir: None,
            images_copied: 0,
            allowed_groups: Some(
                DEFAULT_ALLOWED_GROUPS
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
        }
    }

    /// Import all groups (including adventures, screens, etc.)
    pub fn with_all_groups(mut self) -> Self {
        self.allowed_groups = None;
        self
    }

    /// Set specific groups to import.
    ///
    /// Common groups: "core", "supplement", "setting", "adventure", "screen"
    pub fn with_groups(mut self, groups: Vec<String>) -> Self {
        self.allowed_groups = Some(groups);
        self
    }

    /// Configure image copying from source to destination.
    ///
    /// All images from the source directory will be copied to the destination,
    /// preserving the directory structure so paths in imported data work as-is.
    ///
    /// - `source_img_dir`: Path to 5etools img directory
    /// - `dest_img_dir`: Path to destination directory for images
    pub fn with_image_copy(mut self, source_img_dir: PathBuf, dest_img_dir: PathBuf) -> Self {
        self.source_img_dir = Some(source_img_dir);
        self.dest_img_dir = Some(dest_img_dir);
        self
    }

    /// Import all available sources from a 5etools data directory.
    ///
    /// Each source is imported in its own transaction. If a source fails,
    /// it's rolled back and the next source is attempted.
    ///
    /// If image copying is configured (via `with_image_copy`), all images are
    /// copied first before entity import begins.
    pub fn import_from_directory(&mut self, repo_path: &Path) -> Result<ImportResult> {
        let mut result = ImportResult::default();

        // Copy images first if configured
        if let (Some(source), Some(dest)) = (&self.source_img_dir, &self.dest_img_dir) {
            info!("Copying images from {:?} to {:?}", source, dest);
            match copy_images(source, dest) {
                Ok(count) => {
                    self.images_copied = count;
                    info!("Copied {} images", count);
                }
                Err(e) => {
                    error!("Failed to copy images: {}", e);
                    // Continue with import even if image copy fails
                }
            }
        }

        // Discover available sources
        let all_books = discover_available_sources(repo_path)
            .context("Failed to discover available sources")?;

        // Filter by allowed groups
        let total_books = all_books.len();
        let books: Vec<_> = if let Some(ref allowed) = self.allowed_groups {
            info!(
                "Filtering to groups: {:?} (from {} total sources)",
                allowed, total_books
            );
            all_books
                .into_iter()
                .filter(|book| {
                    let dominated = book.group
                        .as_ref()
                        .map(|g| allowed.iter().any(|a| a.eq_ignore_ascii_case(g)))
                        .unwrap_or(false);
                    if !dominated {
                        info!(
                            "Skipping {} ({}) - group: {:?}",
                            book.name,
                            book.id,
                            book.group
                        );
                    }
                    dominated
                })
                .collect()
        } else {
            all_books
        };

        info!("Importing {} source books", books.len());

        for book in &books {
            let source_code = &book.id;
            info!("Importing source: {} ({})", book.name, source_code);

            match self.import_source_with_transaction(repo_path, source_code, &book.name) {
                Ok(counts) => {
                    let total: usize = counts.values().sum();
                    info!(
                        "Successfully imported {} entities from {}",
                        total, source_code
                    );
                    result.sources_imported.push(source_code.clone());
                    result.total_entities += total;

                    // Merge entity counts
                    for (entity_type, count) in counts {
                        *result.entity_counts.entry(entity_type).or_insert(0) += count;
                    }
                }
                Err(e) => {
                    error!("Failed to import {}: {}", source_code, e);
                    result
                        .sources_failed
                        .push((source_code.clone(), e.to_string()));
                }
            }
        }

        // Global magic variant expansion for disk-based import
        match self.expand_magic_variants_from_disk(repo_path) {
            Ok(count) => {
                if count > 0 {
                    *result.entity_counts.entry("item (expanded variant)".to_string()).or_insert(0) += count;
                    result.total_entities += count;
                }
            }
            Err(e) => {
                warn!("Failed to expand magic variants: {}", e);
            }
        }

        // Add image count to result
        result.images_copied = self.images_copied;

        Ok(result)
    }

    /// Expand magic variant templates from disk-based 5etools data.
    fn expand_magic_variants_from_disk(&mut self, repo_path: &Path) -> Result<usize> {
        let data_dir = repo_path.join("data");

        let variants_file = data_dir.join("magicvariants.json");
        let base_items_file = data_dir.join("items-base.json");

        if !variants_file.exists() || !base_items_file.exists() {
            return Ok(0);
        }

        let variants_data: Value = serde_json::from_str(
            &std::fs::read_to_string(&variants_file)?
        )?;
        let base_items_data: Value = serde_json::from_str(
            &std::fs::read_to_string(&base_items_file)?
        )?;

        let all_variants = variants_data
            .get("magicvariant")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let all_base_items = base_items_data
            .get("baseitem")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        if all_variants.is_empty() || all_base_items.is_empty() {
            return Ok(0);
        }

        info!(
            "Expanding {} magic variant templates against {} base items",
            all_variants.len(),
            all_base_items.len()
        );

        let mut collected = CollectedEntities::default();
        collected.add("magicvariant", all_variants);
        collected.add("baseitem", all_base_items);

        self.expand_and_import_magic_variants(&collected, "")
    }

    /// Import all sources from a tar.gz archive, streaming directly without extraction.
    ///
    /// This reads the tarball once, collecting all JSON files into memory,
    /// then processes them without writing to disk.
    pub fn import_from_tarball(&mut self, tarball_path: &Path) -> Result<ImportResult> {
        let mut result = ImportResult::default();

        info!("Streaming import from tarball: {:?}", tarball_path);

        // Read all JSON files from the tarball into memory
        let json_files = read_json_from_tarball(tarball_path)
            .context("Failed to read JSON files from tarball")?;

        info!("Loaded {} JSON files from tarball", json_files.len());

        // Parse books.json to discover available sources
        let books = parse_books_from_memory(&json_files)
            .context("Failed to parse books.json")?;

        // Filter by allowed groups
        let total_books = books.len();
        let books: Vec<_> = if let Some(ref allowed) = self.allowed_groups {
            info!(
                "Filtering to groups: {:?} (from {} total sources)",
                allowed, total_books
            );
            books
                .into_iter()
                .filter(|book| {
                    let dominated = book.group
                        .as_ref()
                        .map(|g| allowed.iter().any(|a| a.eq_ignore_ascii_case(g)))
                        .unwrap_or(false);
                    if !dominated {
                        info!(
                            "Skipping {} ({}) - group: {:?}",
                            book.name,
                            book.id,
                            book.group
                        );
                    }
                    dominated
                })
                .collect()
        } else {
            books
        };

        info!("Importing {} source books", books.len());

        for book in &books {
            let source_code = &book.id;
            info!("Importing source: {} ({})", book.name, source_code);

            match self.import_source_from_memory(&json_files, source_code, &book.name) {
                Ok(counts) => {
                    let total: usize = counts.values().sum();
                    info!(
                        "Successfully imported {} entities from {}",
                        total, source_code
                    );
                    result.sources_imported.push(source_code.clone());
                    result.total_entities += total;

                    for (entity_type, count) in counts {
                        *result.entity_counts.entry(entity_type).or_insert(0) += count;
                    }
                }
                Err(e) => {
                    error!("Failed to import {}: {}", source_code, e);
                    result
                        .sources_failed
                        .push((source_code.clone(), e.to_string()));
                }
            }
        }

        // Global magic variant expansion: expand all variants against all base items
        // regardless of source (e.g., DMG variants + PHB base items)
        match self.expand_magic_variants_from_memory(&json_files) {
            Ok(count) => {
                if count > 0 {
                    *result.entity_counts.entry("item (expanded variant)".to_string()).or_insert(0) += count;
                    result.total_entities += count;
                }
            }
            Err(e) => {
                warn!("Failed to expand magic variants: {}", e);
            }
        }

        Ok(result)
    }

    /// Expand magic variant templates from in-memory JSON files.
    /// Reads magicvariants.json and items-base.json directly (unfiltered by source),
    /// then expands each variant against matching base items.
    fn expand_magic_variants_from_memory(
        &mut self,
        json_files: &HashMap<String, String>,
    ) -> Result<usize> {
        // Collect ALL magicvariants (not filtered by source)
        let mut all_variants: Vec<Value> = Vec::new();
        let mut all_base_items: Vec<Value> = Vec::new();

        for (path, content) in json_files {
            if path.contains("fluff") {
                continue;
            }

            if path.ends_with("magicvariants.json") || path.contains("magicvariants") {
                if let Ok(data) = serde_json::from_str::<Value>(content) {
                    if let Some(variants) = data.get("magicvariant").and_then(|v| v.as_array()) {
                        all_variants.extend(variants.clone());
                    }
                }
            }

            // Collect base items from items-base.json
            if path.ends_with("items-base.json") {
                if let Ok(data) = serde_json::from_str::<Value>(content) {
                    if let Some(items) = data.get("baseitem").and_then(|v| v.as_array()) {
                        all_base_items.extend(items.clone());
                    }
                }
            }
        }

        if all_variants.is_empty() || all_base_items.is_empty() {
            return Ok(0);
        }

        info!(
            "Expanding {} magic variant templates against {} base items",
            all_variants.len(),
            all_base_items.len()
        );

        // Build a CollectedEntities with variants and base items for the expansion method
        let mut collected = CollectedEntities::default();
        collected.add("magicvariant", all_variants);
        collected.add("baseitem", all_base_items);

        self.expand_and_import_magic_variants(&collected, "")
    }

    /// Import a single source from in-memory JSON files.
    fn import_source_from_memory(
        &mut self,
        json_files: &HashMap<String, String>,
        source_code: &str,
        source_name: &str,
    ) -> Result<HashMap<String, usize>> {
        // Start a transaction
        self.conn
            .batch_execute("SAVEPOINT import_source")
            .context("Failed to create savepoint")?;

        let result = self.import_source_from_memory_internal(json_files, source_code, source_name);

        match &result {
            Ok(_) => {
                self.conn
                    .batch_execute("RELEASE SAVEPOINT import_source")
                    .context("Failed to release savepoint")?;
            }
            Err(_) => {
                self.conn
                    .batch_execute("ROLLBACK TO SAVEPOINT import_source")
                    .context("Failed to rollback savepoint")?;
            }
        }

        result
    }

    fn import_source_from_memory_internal(
        &mut self,
        json_files: &HashMap<String, String>,
        source_code: &str,
        source_name: &str,
    ) -> Result<HashMap<String, usize>> {
        let mut counts = HashMap::new();

        // Insert source record
        let now = chrono::Utc::now().to_rfc3339();
        let source = NewCatalogSource::new(source_code, source_name, true, &now);
        insert_source(self.conn, &source).context("Failed to insert source record")?;

        // Collect entities from in-memory JSON files
        let collected = collect_entities_from_memory(json_files, source_code)
            .context("Failed to collect entities from memory")?;

        // Import each entity type
        for entity_type in collected.entity_types() {
            if let Some(entities) = collected.get(entity_type) {
                let count = self.import_entities(entity_type, entities, source_code, &collected)?;
                if count > 0 {
                    counts.insert(entity_type.to_string(), count);
                }
            }
        }

        // Import spell-class associations from sources.json
        let spell_class_count = self.import_spell_class_associations_from_memory(json_files, source_code)?;
        if spell_class_count > 0 {
            info!("Imported {} spell-class associations for {}", spell_class_count, source_code);
        }

        // Import book content if available
        if collected.has_book_content() {
            match self.import_book(&collected, source_code, source_name) {
                Ok(_) => {
                    info!("Imported book content for {}", source_code);
                    counts.insert("book".to_string(), 1);
                }
                Err(e) => {
                    warn!("Failed to import book content for {}: {}", source_code, e);
                }
            }
        }

        Ok(counts)
    }

    /// Import a single source with transaction handling.
    fn import_source_with_transaction(
        &mut self,
        repo_path: &Path,
        source_code: &str,
        source_name: &str,
    ) -> Result<HashMap<String, usize>> {
        // Start a transaction using SAVEPOINT for nested transaction support
        self.conn
            .batch_execute("SAVEPOINT import_source")
            .context("Failed to create savepoint")?;

        let result = self.import_source_internal(repo_path, source_code, source_name);

        match &result {
            Ok(_) => {
                self.conn
                    .batch_execute("RELEASE SAVEPOINT import_source")
                    .context("Failed to release savepoint")?;
            }
            Err(_) => {
                self.conn
                    .batch_execute("ROLLBACK TO SAVEPOINT import_source")
                    .context("Failed to rollback savepoint")?;
            }
        }

        result
    }

    /// Internal import logic for a single source.
    fn import_source_internal(
        &mut self,
        repo_path: &Path,
        source_code: &str,
        source_name: &str,
    ) -> Result<HashMap<String, usize>> {
        let mut counts = HashMap::new();

        // Insert source record
        let now = chrono::Utc::now().to_rfc3339();
        let source = NewCatalogSource::new(source_code, source_name, true, &now);
        insert_source(self.conn, &source).context("Failed to insert source record")?;

        // Collect all entities from this source
        let collected = collect_source_entities(repo_path, source_code)
            .context("Failed to collect entities")?;

        // Import each entity type
        for entity_type in collected.entity_types() {
            if let Some(entities) = collected.get(entity_type) {
                let count = self.import_entities(entity_type, entities, source_code, &collected)?;
                if count > 0 {
                    counts.insert(entity_type.to_string(), count);
                }
            }
        }

        // Import spell-class associations from sources.json
        // This is separate from spell data in 5etools
        let spell_class_count = self.import_spell_class_associations(repo_path, source_code)?;
        if spell_class_count > 0 {
            info!("Imported {} spell-class associations for {}", spell_class_count, source_code);
        }

        // Import book content if available
        if collected.has_book_content() {
            match self.import_book(&collected, source_code, source_name) {
                Ok(_) => {
                    info!("Imported book content for {}", source_code);
                    counts.insert("book".to_string(), 1);
                }
                Err(e) => {
                    warn!("Failed to import book content for {}: {}", source_code, e);
                }
            }
        }

        Ok(counts)
    }

    /// Import entities of a specific type.
    fn import_entities(
        &mut self,
        entity_type: &str,
        entities: &[Value],
        source: &str,
        collected: &CollectedEntities,
    ) -> Result<usize> {
        let mut count = 0;

        for entity in entities {
            match self.import_single_entity(entity_type, entity, source, collected) {
                Ok(id) => {
                    count += 1;
                    // Index in FTS
                    if let Err(e) = self.index_entity_fts(entity_type, id, entity) {
                        warn!("Failed to index entity in FTS: {}", e);
                    }
                }
                Err(e) => {
                    let name = entity
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    warn!("Failed to import {} '{}': {}", entity_type, name, e);
                }
            }
        }

        Ok(count)
    }

    /// Import a single entity based on its type.
    fn import_single_entity(
        &mut self,
        entity_type: &str,
        entity: &Value,
        source: &str,
        collected: &CollectedEntities,
    ) -> Result<i32> {
        let name = entity
            .get("name")
            .and_then(|v| v.as_str())
            .context("Entity missing name")?;
        let data = serde_json::to_string(entity)?;

        // Look up fluff data for this entity
        let fluff = collected
            .get_fluff(entity_type, name, source)
            .map(|v| serde_json::to_string(v).ok())
            .flatten();
        let fluff_ref = fluff.as_deref();

        match entity_type {
            "monster" => self.import_monster(entity, name, source, &data, fluff_ref),
            "spell" => self.import_spell(entity, name, source, &data, fluff_ref),
            "item" | "baseitem" => self.import_item(entity, name, source, &data, fluff_ref),
            "magicvariant" => Ok(0), // Expanded separately after base items are imported
            "class" => self.import_class(name, source, &data, fluff_ref),
            "classFeature" => self.import_class_feature(entity, name, source, &data),
            "subclass" => self.import_subclass(entity, name, source, &data, fluff_ref),
            "subclassFeature" => self.import_subclass_feature(entity, name, source, &data),
            "race" | "subrace" => self.import_race(name, source, &data, fluff_ref),
            "background" => self.import_background(name, source, &data, fluff_ref),
            "feat" => self.import_feat(name, source, &data, fluff_ref),
            "condition" => self.import_condition(name, source, &data, fluff_ref),
            "disease" => self.import_disease(name, source, &data, fluff_ref),
            "action" => self.import_action(name, source, &data),
            "language" => self.import_language(entity, name, source, &data, fluff_ref),
            "vehicle" => self.import_vehicle(entity, name, source, &data, fluff_ref),
            "object" => self.import_object(entity, name, source, &data, fluff_ref),
            "trap" => self.import_trap(entity, name, source, &data, fluff_ref),
            "hazard" => self.import_hazard(name, source, &data, fluff_ref),
            "cult" | "boon" => self.import_cult(name, source, &data),
            "deity" => self.import_deity(entity, name, source, &data),
            "sense" => self.import_sense(name, source, &data),
            "skill" => self.import_skill(entity, name, source, &data),
            "optionalfeature" => self.import_optional_feature(entity, name, source, &data),
            "psionic" => self.import_psionic(entity, name, source, &data),
            "reward" => self.import_reward(entity, name, source, &data),
            "variantrule" => self.import_variant_rule(entity, name, source, &data),
            "table" => self.import_catalog_table(name, source, &data),
            _ => {
                // Unknown entity type - skip silently
                Ok(0)
            }
        }
    }

    // === Entity-specific import functions with field extraction ===

    fn import_monster(&mut self, entity: &Value, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let cr = extract_cr(entity);
        let creature_type = extract_creature_type(entity);
        let size = extract_size(entity);

        let mut monster = NewMonster::new(name, source, data);
        monster.fluff = fluff;
        if let Some(ref cr) = cr {
            monster = monster.with_cr(cr);
        }
        if let Some(ref ct) = creature_type {
            monster = monster.with_creature_type(ct);
        }
        if let Some(ref sz) = size {
            monster = monster.with_size(sz);
        }

        let monster_id = catalog::insert_monster(self.conn, &monster).context("Failed to insert monster")?;

        // Set token path if images are configured and token exists
        if let Some(dest_img_dir) = &self.dest_img_dir {
            if let Some(token_path) = get_token_path(dest_img_dir, source, name) {
                if let Err(e) = catalog::set_token_image_path(self.conn, monster_id, Some(&token_path)) {
                    warn!("Failed to update monster token path: {}", e);
                }
            }
        }

        Ok(monster_id)
    }

    fn import_spell(&mut self, entity: &Value, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let level = entity
            .get("level")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let school = entity
            .get("school")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let ritual = extract_ritual(entity);
        let concentration = extract_concentration(entity);

        let mut spell = NewSpell::new(name, source, level, data)
            .with_ritual(ritual)
            .with_concentration(concentration);
        spell.fluff = fluff;
        if let Some(ref sch) = school {
            spell = spell.with_school(sch);
        }

        let spell_id = catalog::insert_spell(self.conn, &spell).context("Failed to insert spell")?;

        // Extract and insert spell-class relationships
        self.import_spell_classes(spell_id, entity, source)?;

        Ok(spell_id)
    }

    fn import_spell_classes(&mut self, spell_id: i32, entity: &Value, source: &str) -> Result<()> {
        // Extract class lists from the spell data
        if let Some(classes) = entity.get("classes").and_then(|c| c.get("fromClassList")) {
            if let Some(class_array) = classes.as_array() {
                for class_entry in class_array {
                    if let Some(class_name) = class_entry.get("name").and_then(|n| n.as_str()) {
                        let class_source = class_entry
                            .get("source")
                            .and_then(|s| s.as_str())
                            .unwrap_or(source);

                        let spell_class = NewSpellClass {
                            spell_id,
                            class_name,
                            source: class_source,
                        };

                        if let Err(e) = catalog::insert_spell_class(self.conn, &spell_class) {
                            warn!("Failed to insert spell-class relationship: {}", e);
                        }
                    }
                }
            }
        }

        // Extract subclass lists
        if let Some(subclasses) = entity.get("classes").and_then(|c| c.get("fromSubclass")) {
            if let Some(subclass_array) = subclasses.as_array() {
                for subclass_entry in subclass_array {
                    if let (Some(class_obj), Some(subclass_obj)) = (
                        subclass_entry.get("class"),
                        subclass_entry.get("subclass"),
                    ) {
                        let class_name = class_obj.get("name").and_then(|n| n.as_str());
                        let subclass_name = subclass_obj.get("name").and_then(|n| n.as_str());
                        let subclass_source = subclass_obj
                            .get("source")
                            .and_then(|s| s.as_str())
                            .unwrap_or(source);

                        if let (Some(class_name), Some(subclass_name)) = (class_name, subclass_name)
                        {
                            let spell_subclass = NewSpellSubclass {
                                spell_id,
                                subclass_name,
                                class_name,
                                source: subclass_source,
                            };

                            if let Err(e) =
                                catalog::insert_spell_subclass(self.conn, &spell_subclass)
                            {
                                warn!("Failed to insert spell-subclass relationship: {}", e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Import spell-class associations from the 5etools sources.json file.
    ///
    /// In 5etools, spell-class associations are stored separately from spell data
    /// in `data/spells/sources.json`. This file maps source -> spell name -> classes.
    fn import_spell_class_associations(&mut self, repo_path: &Path, source_code: &str) -> Result<usize> {
        let sources_file = repo_path.join("data").join("spells").join("sources.json");
        if !sources_file.exists() {
            return Ok(0);
        }

        let content = std::fs::read_to_string(&sources_file)
            .context("Failed to read spells/sources.json")?;
        let sources_data: Value = serde_json::from_str(&content)
            .context("Failed to parse spells/sources.json")?;

        let mut count = 0;

        // sources.json structure: { "SOURCE_CODE": { "Spell Name": { "class": [...], "subclass": [...] } } }
        if let Some(source_spells) = sources_data.get(source_code).and_then(|v| v.as_object()) {
            for (spell_name, spell_data) in source_spells {
                // Look up the spell in the database
                let spell = match catalog::get_spell_by_name(self.conn, spell_name, source_code) {
                    Ok(Some(s)) => s,
                    Ok(None) => continue, // Spell not found, skip
                    Err(e) => {
                        warn!("Failed to look up spell '{}': {}", spell_name, e);
                        continue;
                    }
                };

                let spell_id = match spell.id {
                    Some(id) => id,
                    None => continue,
                };

                // Import class associations
                if let Some(classes) = spell_data.get("class").and_then(|v| v.as_array()) {
                    for class_entry in classes {
                        if let Some(class_name) = class_entry.get("name").and_then(|n| n.as_str()) {
                            let class_source = class_entry
                                .get("source")
                                .and_then(|s| s.as_str())
                                .unwrap_or(source_code);

                            let spell_class = NewSpellClass {
                                spell_id,
                                class_name,
                                source: class_source,
                            };

                            if catalog::insert_spell_class(self.conn, &spell_class).is_ok() {
                                count += 1;
                            }
                        }
                    }
                }

                // Import subclass associations
                if let Some(subclasses) = spell_data.get("subclass").and_then(|v| v.as_array()) {
                    for subclass_entry in subclasses {
                        if let (Some(class_obj), Some(subclass_obj)) = (
                            subclass_entry.get("class"),
                            subclass_entry.get("subclass"),
                        ) {
                            let class_name = class_obj.get("name").and_then(|n| n.as_str());
                            let subclass_name = subclass_obj.get("name").and_then(|n| n.as_str());
                            let subclass_source = subclass_obj
                                .get("source")
                                .and_then(|s| s.as_str())
                                .unwrap_or(source_code);

                            if let (Some(class_name), Some(subclass_name)) = (class_name, subclass_name) {
                                let spell_subclass = NewSpellSubclass {
                                    spell_id,
                                    subclass_name,
                                    class_name,
                                    source: subclass_source,
                                };

                                let _ = catalog::insert_spell_subclass(self.conn, &spell_subclass);
                            }
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Import spell-class associations from in-memory JSON files.
    ///
    /// This is the in-memory version used when importing from tarballs.
    fn import_spell_class_associations_from_memory(
        &mut self,
        json_files: &HashMap<String, String>,
        source_code: &str,
    ) -> Result<usize> {
        // Look for data/spells/sources.json in the in-memory files
        let sources_key = json_files.keys().find(|k| k.ends_with("data/spells/sources.json"));
        let content = match sources_key {
            Some(key) => json_files.get(key),
            None => return Ok(0),
        };

        let content = match content {
            Some(c) => c,
            None => return Ok(0),
        };

        let sources_data: Value = serde_json::from_str(content)
            .context("Failed to parse spells/sources.json")?;

        let mut count = 0;

        // sources.json structure: { "SOURCE_CODE": { "Spell Name": { "class": [...], "subclass": [...] } } }
        if let Some(source_spells) = sources_data.get(source_code).and_then(|v| v.as_object()) {
            for (spell_name, spell_data) in source_spells {
                // Look up the spell in the database
                let spell = match catalog::get_spell_by_name(self.conn, spell_name, source_code) {
                    Ok(Some(s)) => s,
                    Ok(None) => continue, // Spell not found, skip
                    Err(e) => {
                        warn!("Failed to look up spell '{}': {}", spell_name, e);
                        continue;
                    }
                };

                let spell_id = match spell.id {
                    Some(id) => id,
                    None => continue,
                };

                // Import class associations
                if let Some(classes) = spell_data.get("class").and_then(|v| v.as_array()) {
                    for class_entry in classes {
                        if let Some(class_name) = class_entry.get("name").and_then(|n| n.as_str()) {
                            let class_source = class_entry
                                .get("source")
                                .and_then(|s| s.as_str())
                                .unwrap_or(source_code);

                            let spell_class = NewSpellClass {
                                spell_id,
                                class_name,
                                source: class_source,
                            };

                            if catalog::insert_spell_class(self.conn, &spell_class).is_ok() {
                                count += 1;
                            }
                        }
                    }
                }

                // Import subclass associations
                if let Some(subclasses) = spell_data.get("subclass").and_then(|v| v.as_array()) {
                    for subclass_entry in subclasses {
                        if let (Some(class_obj), Some(subclass_obj)) = (
                            subclass_entry.get("class"),
                            subclass_entry.get("subclass"),
                        ) {
                            let class_name = class_obj.get("name").and_then(|n| n.as_str());
                            let subclass_name = subclass_obj.get("name").and_then(|n| n.as_str());
                            let subclass_source = subclass_obj
                                .get("source")
                                .and_then(|s| s.as_str())
                                .unwrap_or(source_code);

                            if let (Some(class_name), Some(subclass_name)) = (class_name, subclass_name) {
                                let spell_subclass = NewSpellSubclass {
                                    spell_id,
                                    subclass_name,
                                    class_name,
                                    source: subclass_source,
                                };

                                let _ = catalog::insert_spell_subclass(self.conn, &spell_subclass);
                            }
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    fn import_item(&mut self, entity: &Value, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let item_type = entity
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.split('|').next().unwrap_or(s).to_string());
        let rarity = entity
            .get("rarity")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut item = NewItem::new(name, source, data);
        item.fluff = fluff;
        if let Some(ref t) = item_type {
            item = item.with_type(t);
        }
        if let Some(ref r) = rarity {
            item = item.with_rarity(r);
        }

        let item_id = catalog::insert_item(self.conn, &item).context("Failed to insert item")?;

        // Extract and insert item attunement class restrictions
        self.import_item_attunement(item_id, entity)?;

        Ok(item_id)
    }

    fn import_item_attunement(&mut self, item_id: i32, entity: &Value) -> Result<()> {
        // Check for class-restricted attunement
        if let Some(req_attune) = entity.get("reqAttune") {
            // reqAttune can be a string like "by a cleric or paladin"
            if let Some(attune_str) = req_attune.as_str() {
                // Parse class names from the string
                let class_names = extract_attunement_classes(attune_str);
                for class_name in class_names {
                    let attunement = NewItemAttunementClass {
                        item_id,
                        class_name: &class_name,
                    };
                    if let Err(e) = catalog::insert_item_attunement_class(self.conn, &attunement) {
                        warn!("Failed to insert item attunement class: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Expand magic variant templates against base items and import the resulting concrete items.
    ///
    /// For each Generic Variant (e.g., "+1 Weapon"), matches it against base items using
    /// `requires`/`excludes` rules, then creates concrete items (e.g., "+1 Shortsword")
    /// by merging the base item data with the variant's `inherits` block.
    fn expand_and_import_magic_variants(
        &mut self,
        collected: &CollectedEntities,
        source_code: &str,
    ) -> Result<usize> {
        let variants = match collected.get("magicvariant") {
            Some(v) => v,
            None => return Ok(0),
        };

        // Collect base items for matching — use "baseitem" entities if available,
        // fall back to checking "item" entities that have base item characteristics
        let base_items: Vec<&Value> = collected
            .get("baseitem")
            .map(|v| v.iter().collect())
            .unwrap_or_default();

        if base_items.is_empty() {
            info!("No base items available for magic variant expansion");
            return Ok(0);
        }

        let mut count = 0;

        for variant in variants {
            let variant_name = match variant.get("name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => continue,
            };

            let requires = match variant.get("requires").and_then(|v| v.as_array()) {
                Some(r) => r,
                None => continue,
            };

            let excludes = variant.get("excludes");
            let inherits = match variant.get("inherits") {
                Some(i) => i,
                None => continue,
            };

            // Extract naming rules from inherits
            let name_prefix = inherits.get("namePrefix").and_then(|v| v.as_str()).unwrap_or("");
            let name_suffix = inherits.get("nameSuffix").and_then(|v| v.as_str()).unwrap_or("");
            let name_remove = inherits.get("nameRemove").and_then(|v| v.as_str()).unwrap_or("");

            // Determine the source for expanded items
            let variant_source = inherits
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or(source_code);

            for base_item in &base_items {
                // Check requires (ANY requirement object must match)
                let matches_any_req = requires.iter().any(|req| {
                    if let Some(req_obj) = req.as_object() {
                        req_obj.iter().all(|(key, val)| base_item_matches_field(base_item, key, val))
                    } else {
                        false
                    }
                });

                if !matches_any_req {
                    continue;
                }

                // Check excludes
                if let Some(exc) = excludes {
                    if base_item_excluded(base_item, exc) {
                        continue;
                    }
                }

                // Build expanded item name
                let base_name = base_item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let cleaned_name = if !name_remove.is_empty() {
                    base_name.replace(name_remove, "")
                } else {
                    base_name.to_string()
                };
                let expanded_name = format!("{}{}{}", name_prefix, cleaned_name, name_suffix);

                // Build expanded item JSON: start with base, overlay inherits
                let expanded = build_expanded_item(base_item, inherits, &expanded_name, variant_name);
                let data = serde_json::to_string(&expanded)?;

                let item_type = expanded
                    .get("type")
                    .and_then(|v| v.as_str())
                    .map(|s| s.split('|').next().unwrap_or(s).to_string());
                let rarity = expanded
                    .get("rarity")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let mut new_item = NewItem::new(&expanded_name, variant_source, &data);
                if let Some(ref t) = item_type {
                    new_item = new_item.with_type(t);
                }
                if let Some(ref r) = rarity {
                    new_item = new_item.with_rarity(r);
                }

                match catalog::insert_item(self.conn, &new_item) {
                    Ok(item_id) => {
                        count += 1;
                        // Index in FTS
                        if let Err(e) = self.index_entity_fts("item", item_id, &expanded) {
                            warn!("Failed to FTS index expanded item '{}': {}", expanded_name, e);
                        }
                        // Import attunement from expanded data
                        if let Err(e) = self.import_item_attunement(item_id, &expanded) {
                            warn!("Failed to import attunement for '{}': {}", expanded_name, e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to insert expanded item '{}': {}", expanded_name, e);
                    }
                }
            }
        }

        if count > 0 {
            info!(
                "Expanded {} concrete items from {} magic variant templates",
                count,
                variants.len()
            );
        }

        Ok(count)
    }

    fn import_class(&mut self, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let mut class = NewClass::new(name, source, data);
        class.fluff = fluff;
        catalog::insert_class(self.conn, &class).context("Failed to insert class")
    }

    fn import_class_feature(
        &mut self,
        entity: &Value,
        name: &str,
        source: &str,
        data: &str,
    ) -> Result<i32> {
        let class_name = entity
            .get("className")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let class_source = entity
            .get("classSource")
            .and_then(|v| v.as_str())
            .unwrap_or(source);
        let level = entity
            .get("level")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as i32;

        let feature = NewClassFeature::new(name, source, class_name, class_source, level, data);
        catalog::insert_class_feature(self.conn, &feature).context("Failed to insert class feature")
    }

    fn import_subclass(
        &mut self,
        entity: &Value,
        name: &str,
        source: &str,
        data: &str,
        fluff: Option<&str>,
    ) -> Result<i32> {
        let class_name = entity
            .get("className")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let mut subclass = NewSubclass::new(name, class_name, source, data);
        subclass.fluff = fluff;
        catalog::insert_subclass(self.conn, &subclass).context("Failed to insert subclass")
    }

    fn import_subclass_feature(
        &mut self,
        entity: &Value,
        name: &str,
        source: &str,
        data: &str,
    ) -> Result<i32> {
        let class_name = entity
            .get("className")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let class_source = entity
            .get("classSource")
            .and_then(|v| v.as_str())
            .unwrap_or(source);
        let subclass_name = entity
            .get("subclassShortName")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let subclass_source = entity
            .get("subclassSource")
            .and_then(|v| v.as_str())
            .unwrap_or(source);
        let level = entity
            .get("level")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as i32;

        let feature = NewSubclassFeature::new(
            name, source, class_name, class_source,
            subclass_name, subclass_source, level, data
        );
        catalog::insert_subclass_feature(self.conn, &feature).context("Failed to insert subclass feature")
    }

    fn import_race(&mut self, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let mut race = NewRace::new(name, source, data);
        race.fluff = fluff;
        catalog::insert_race(self.conn, &race).context("Failed to insert race")
    }

    fn import_background(&mut self, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let mut background = NewBackground::new(name, source, data);
        background.fluff = fluff;
        catalog::insert_background(self.conn, &background).context("Failed to insert background")
    }

    fn import_feat(&mut self, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let mut feat = NewFeat::new(name, source, data);
        feat.fluff = fluff;
        catalog::insert_feat(self.conn, &feat).context("Failed to insert feat")
    }

    fn import_condition(&mut self, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let mut condition = NewCondition::new(name, source, data);
        condition.fluff = fluff;
        catalog::insert_condition(self.conn, &condition).context("Failed to insert condition")
    }

    fn import_disease(&mut self, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let mut disease = NewDisease::new(name, source, data);
        disease.fluff = fluff;
        catalog::insert_disease(self.conn, &disease).context("Failed to insert disease")
    }

    fn import_action(&mut self, name: &str, source: &str, data: &str) -> Result<i32> {
        let action = NewAction::new(name, source, data);
        catalog::insert_action(self.conn, &action).context("Failed to insert action")
    }

    fn import_language(
        &mut self,
        entity: &Value,
        name: &str,
        source: &str,
        data: &str,
        fluff: Option<&str>,
    ) -> Result<i32> {
        let language_type = entity
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut language = NewLanguage::new(name, source, data);
        language.fluff = fluff;
        if let Some(ref t) = language_type {
            language = language.with_type(t);
        }
        catalog::insert_language(self.conn, &language).context("Failed to insert language")
    }

    fn import_vehicle(
        &mut self,
        entity: &Value,
        name: &str,
        source: &str,
        data: &str,
        fluff: Option<&str>,
    ) -> Result<i32> {
        let vehicle_type = entity
            .get("vehicleType")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut vehicle = NewVehicle::new(name, source, data);
        vehicle.fluff = fluff;
        if let Some(ref t) = vehicle_type {
            vehicle = vehicle.with_type(t);
        }
        catalog::insert_vehicle(self.conn, &vehicle).context("Failed to insert vehicle")
    }

    fn import_object(
        &mut self,
        entity: &Value,
        name: &str,
        source: &str,
        data: &str,
        fluff: Option<&str>,
    ) -> Result<i32> {
        let object_type = entity
            .get("objectType")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut object = NewObject::new(name, source, data);
        object.fluff = fluff;
        if let Some(ref t) = object_type {
            object = object.with_type(t);
        }
        catalog::insert_object(self.conn, &object).context("Failed to insert object")
    }

    fn import_trap(&mut self, entity: &Value, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let trap_tier = entity
            .get("trapTier")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut trap = NewTrap::new(name, source, data);
        trap.fluff = fluff;
        let trap = if let Some(tier) = trap_tier.as_deref() {
            trap.with_tier(tier)
        } else {
            trap
        };
        catalog::insert_trap(self.conn, &trap).context("Failed to insert trap")
    }

    fn import_hazard(&mut self, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let mut hazard = NewHazard::new(name, source, data);
        hazard.fluff = fluff;
        catalog::insert_hazard(self.conn, &hazard).context("Failed to insert hazard")
    }

    fn import_cult(&mut self, name: &str, source: &str, data: &str) -> Result<i32> {
        let cult = NewCult::new(name, source, data);
        catalog::insert_cult(self.conn, &cult).context("Failed to insert cult")
    }

    fn import_deity(
        &mut self,
        entity: &Value,
        name: &str,
        source: &str,
        data: &str,
    ) -> Result<i32> {
        let pantheon = entity
            .get("pantheon")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let deity = NewDeity::new(name, source, data);
        let deity = if let Some(p) = pantheon.as_deref() {
            deity.with_pantheon(p)
        } else {
            deity
        };
        catalog::insert_deity(self.conn, &deity).context("Failed to insert deity")
    }

    fn import_sense(&mut self, name: &str, source: &str, data: &str) -> Result<i32> {
        let sense = NewSense::new(name, source, data);
        catalog::insert_sense(self.conn, &sense).context("Failed to insert sense")
    }

    fn import_skill(&mut self, entity: &Value, name: &str, source: &str, data: &str) -> Result<i32> {
        let ability = entity
            .get("ability")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let skill = NewSkill::new(name, source, data);
        let skill = if let Some(a) = ability.as_deref() {
            skill.with_ability(a)
        } else {
            skill
        };
        catalog::insert_skill(self.conn, &skill).context("Failed to insert skill")
    }

    fn import_optional_feature(
        &mut self,
        entity: &Value,
        name: &str,
        source: &str,
        data: &str,
    ) -> Result<i32> {
        let feature_type = entity
            .get("featureType")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let feature = NewOptionalFeature::new(name, source, data);
        let feature = if let Some(ft) = feature_type.as_deref() {
            feature.with_feature_type(ft)
        } else {
            feature
        };
        catalog::insert_optional_feature(self.conn, &feature)
            .context("Failed to insert optional feature")
    }

    fn import_psionic(
        &mut self,
        entity: &Value,
        name: &str,
        source: &str,
        data: &str,
    ) -> Result<i32> {
        let psionic_type = entity
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let order = entity
            .get("order")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut psionic = NewPsionic::new(name, source, data);
        if let Some(ref pt) = psionic_type {
            psionic = psionic.with_type(pt);
        }
        if let Some(ref o) = order {
            psionic = psionic.with_order(o);
        }
        catalog::insert_psionic(self.conn, &psionic).context("Failed to insert psionic")
    }

    fn import_reward(
        &mut self,
        entity: &Value,
        name: &str,
        source: &str,
        data: &str,
    ) -> Result<i32> {
        let reward_type = entity
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let reward = NewReward::new(name, source, data);
        let reward = if let Some(rt) = reward_type.as_deref() {
            reward.with_type(rt)
        } else {
            reward
        };
        catalog::insert_reward(self.conn, &reward).context("Failed to insert reward")
    }

    fn import_variant_rule(
        &mut self,
        entity: &Value,
        name: &str,
        source: &str,
        data: &str,
    ) -> Result<i32> {
        let rule_type = entity
            .get("ruleType")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let rule = NewVariantRule::new(name, source, data);
        let rule = if let Some(rt) = rule_type.as_deref() {
            rule.with_type(rt)
        } else {
            rule
        };
        catalog::insert_variant_rule(self.conn, &rule).context("Failed to insert variant rule")
    }

    fn import_catalog_table(&mut self, name: &str, source: &str, data: &str) -> Result<i32> {
        let table = NewCatalogTable::new(name, source, data);
        catalog::insert_catalog_table(self.conn, &table).context("Failed to insert catalog table")
    }

    /// Import book content (readable chapters/sections).
    ///
    /// This imports the full book content that can be rendered in a reader view.
    /// Image paths in the content are rewritten to use the local image directory.
    fn import_book(
        &mut self,
        collected: &CollectedEntities,
        source: &str,
        name: &str,
    ) -> Result<i32> {
        let book_data = collected
            .book_content
            .as_ref()
            .context("No book content available")?;

        // Rewrite image paths if we have an image directory configured
        let processed_data = if self.dest_img_dir.is_some() {
            rewrite_book_image_paths(book_data)
        } else {
            book_data.clone()
        };

        let data_str = serde_json::to_string(&processed_data)?;

        // Get table of contents if available
        let toc_str = collected
            .book_contents_toc
            .as_ref()
            .map(|toc| serde_json::to_string(toc))
            .transpose()?;

        // Get cover path - books use pattern: book/{source}/cover.webp
        let cover_path = self.dest_img_dir.as_ref().and_then(|img_dir| {
            let webp_path = format!("book/{}/cover.webp", source);
            if crate::import::image_exists(img_dir, &webp_path) {
                return Some(webp_path);
            }
            let jpg_path = format!("book/{}/cover.jpg", source);
            if crate::import::image_exists(img_dir, &jpg_path) {
                return Some(jpg_path);
            }
            let png_path = format!("book/{}/cover.png", source);
            if crate::import::image_exists(img_dir, &png_path) {
                return Some(png_path);
            }
            None
        });

        let mut book = NewBook::new(source, name, &data_str);
        if let Some(ref toc) = toc_str {
            book = book.with_contents(toc);
        }
        if let Some(ref cover) = cover_path {
            book = book.with_cover_path(cover);
        }

        catalog::insert_book(self.conn, &book).context("Failed to insert book")
    }

    // === FTS Indexing ===

    fn index_entity_fts(&mut self, entity_type: &str, entity_id: i32, entity: &Value) -> Result<()> {
        let name = entity
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Extract and flatten entries for rules content
        if let Some(entries) = entity.get("entries") {
            if let Some(entries_array) = entries.as_array() {
                let text = flatten_entries(entries_array);
                if !text.is_empty() {
                    index_entity(
                        self.conn,
                        entity_type,
                        entity_id,
                        ContentType::Rules,
                        name,
                        &text,
                    )?;
                }
            }
        }

        // Index fluff content if present
        if let Some(fluff) = entity.get("fluff") {
            if let Some(fluff_entries) = fluff.get("entries").and_then(|e| e.as_array()) {
                let text = flatten_entries(fluff_entries);
                if !text.is_empty() {
                    index_entity(
                        self.conn,
                        entity_type,
                        entity_id,
                        ContentType::Fluff,
                        name,
                        &text,
                    )?;
                }
            }
        }

        Ok(())
    }
}

// === Tarball Streaming Helpers ===

/// Read all JSON files from a tar.gz archive into memory.
///
/// Returns a HashMap where keys are relative paths (e.g., "data/bestiary/bestiary-mm.json")
/// and values are the JSON content as strings.
fn read_json_from_tarball(tarball_path: &Path) -> Result<HashMap<String, String>> {
    use std::fs::File;

    let file = File::open(tarball_path)
        .context("Failed to open tarball")?;

    let decoder = GzDecoder::new(BufReader::new(file));
    let mut archive = Archive::new(decoder);

    let mut json_files = HashMap::new();
    let mut prefix_to_strip: Option<String> = None;

    let entries = archive.entries()
        .context("Failed to read archive entries")?;

    for entry_result in entries {
        let mut entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                warn!("Skipping entry: {}", e);
                continue;
            }
        };

        let path = match entry.path() {
            Ok(p) => p.to_path_buf(),
            Err(_) => continue,
        };

        // Detect prefix to strip (e.g., "5etools-2.24.0/")
        if prefix_to_strip.is_none() && entry.header().entry_type().is_dir() {
            let path_str = path.to_string_lossy();
            if path_str.starts_with("5etools-") || path_str.starts_with("5etools-mirror-") {
                prefix_to_strip = Some(path_str.trim_end_matches('/').to_string());
                info!("Detected archive prefix: {:?}", prefix_to_strip);
            }
        }

        // Skip non-files
        if !entry.header().entry_type().is_file() {
            continue;
        }

        // Only process JSON files in the data directory
        let path_str = path.to_string_lossy().to_string();

        // Strip prefix if detected
        let relative_path = if let Some(ref prefix) = prefix_to_strip {
            if let Some(stripped) = path_str.strip_prefix(prefix) {
                stripped.trim_start_matches('/').to_string()
            } else {
                path_str.clone()
            }
        } else {
            path_str.clone()
        };

        // Only keep JSON files in data/
        if !relative_path.starts_with("data/") || !relative_path.ends_with(".json") {
            continue;
        }

        // Read the file content
        let mut content = String::new();
        if entry.read_to_string(&mut content).is_ok() {
            json_files.insert(relative_path, content);
        }
    }

    info!("Read {} JSON files from tarball", json_files.len());
    Ok(json_files)
}

/// Book metadata for source discovery (simplified).
#[derive(Debug)]
struct BookMeta {
    id: String,
    name: String,
    group: Option<String>,
}

/// Parse books.json from in-memory files to discover available sources.
fn parse_books_from_memory(json_files: &HashMap<String, String>) -> Result<Vec<BookMeta>> {
    let books_content = json_files
        .get("data/books.json")
        .context("books.json not found in archive")?;

    let books_data: Value = serde_json::from_str(books_content)
        .context("Failed to parse books.json")?;

    let mut books = Vec::new();

    if let Some(book_array) = books_data.get("book").and_then(|b| b.as_array()) {
        for book in book_array {
            let id = book.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = book.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let group = book.get("group").and_then(|v| v.as_str()).map(|s| s.to_string());

            if !id.is_empty() {
                books.push(BookMeta { id, name, group });
            }
        }
    }

    Ok(books)
}

/// Collect entities from in-memory JSON files for a specific source.
fn collect_entities_from_memory(
    json_files: &HashMap<String, String>,
    source_code: &str,
) -> Result<CollectedEntities> {
    let mut collected = CollectedEntities::default();

    // Helper to check if an entity belongs to this source
    let matches_source = |entity: &Value| -> bool {
        entity.get("source")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case(source_code))
            .unwrap_or(false)
    };

    // Entity type mappings: (file_pattern, json_key, entity_type)
    let entity_mappings = [
        ("data/bestiary/bestiary-", "monster", "monster"),
        ("data/spells/spells-", "spell", "spell"),
        ("data/items", "item", "item"),
        ("data/items", "baseitem", "baseitem"),
        ("data/magicvariants.json", "magicvariant", "magicvariant"),
        ("data/class/class-", "class", "class"),
        ("data/class/class-", "subclass", "subclass"),
        ("data/class/class-", "classFeature", "classFeature"),
        ("data/class/class-", "subclassFeature", "subclassFeature"),
        ("data/races.json", "race", "race"),
        ("data/races.json", "subrace", "subrace"),
        ("data/backgrounds.json", "background", "background"),
        ("data/feats.json", "feat", "feat"),
        ("data/conditionsdiseases.json", "condition", "condition"),
        ("data/conditionsdiseases.json", "disease", "disease"),
        ("data/actions.json", "action", "action"),
        ("data/languages.json", "language", "language"),
        ("data/vehicles.json", "vehicle", "vehicle"),
        ("data/objects.json", "object", "object"),
        ("data/trapshazards.json", "trap", "trap"),
        ("data/trapshazards.json", "hazard", "hazard"),
        ("data/cultsboons.json", "cult", "cult"),
        ("data/cultsboons.json", "boon", "boon"),
        ("data/deities.json", "deity", "deity"),
        ("data/senses.json", "sense", "sense"),
        ("data/skills.json", "skill", "skill"),
        ("data/optionalfeatures.json", "optionalfeature", "optionalfeature"),
        ("data/psionics.json", "psionic", "psionic"),
        ("data/rewards.json", "reward", "reward"),
        ("data/variantrules.json", "variantrule", "variantrule"),
        ("data/tables.json", "table", "table"),
    ];

    // Collect fluff data first
    for (path, content) in json_files {
        if path.contains("fluff") {
            if let Ok(data) = serde_json::from_str::<Value>(content) {
                // Try to find fluff arrays
                for key in ["monsterFluff", "spellFluff", "itemFluff", "classFluff",
                            "raceFluff", "backgroundFluff", "featFluff", "languageFluff",
                            "vehicleFluff", "objectFluff", "trapFluff", "hazardFluff",
                            "conditionFluff", "diseaseFluff"] {
                    if let Some(fluff_array) = data.get(key).and_then(|v| v.as_array()) {
                        for fluff in fluff_array {
                            let name = fluff.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let source = fluff.get("source").and_then(|v| v.as_str()).unwrap_or("");
                            if source.eq_ignore_ascii_case(source_code) && !name.is_empty() {
                                let entity_type = key.trim_end_matches("Fluff");
                                collected.add_fluff(entity_type, name, source, fluff.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Collect entities
    for (path, content) in json_files {
        // Skip fluff files for entity collection
        if path.contains("fluff") {
            continue;
        }

        let data: Value = match serde_json::from_str(content) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for (file_pattern, json_key, entity_type) in &entity_mappings {
            if !path.contains(file_pattern) {
                continue;
            }

            if let Some(entities) = data.get(*json_key).and_then(|v| v.as_array()) {
                let filtered: Vec<Value> = entities
                    .iter()
                    .filter(|e| matches_source(e))
                    .cloned()
                    .collect();

                if !filtered.is_empty() {
                    collected.add(entity_type, filtered);
                }
            }
        }
    }

    // Collect book content if available
    let book_file = format!("data/book/book-{}.json", source_code.to_lowercase());
    if let Some(content) = json_files.get(&book_file) {
        if let Ok(data) = serde_json::from_str::<Value>(content) {
            if let Some(book_data) = data.get("data") {
                collected.book_content = Some(book_data.clone());
            }
        }
    }

    // Get TOC from books.json
    if let Some(books_content) = json_files.get("data/books.json") {
        if let Ok(books_data) = serde_json::from_str::<Value>(books_content) {
            if let Some(book_array) = books_data.get("book").and_then(|v| v.as_array()) {
                for book in book_array {
                    if book.get("id").and_then(|v| v.as_str()) == Some(source_code) {
                        if let Some(contents) = book.get("contents") {
                            collected.book_contents_toc = Some(contents.clone());
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(collected)
}

// === Field Extraction Helpers ===

/// Extract CR from a monster entity.
/// CR can be a string directly or an object with a "cr" field.
fn extract_cr(entity: &Value) -> Option<String> {
    match entity.get("cr") {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Object(obj)) => obj.get("cr").and_then(|v| v.as_str()).map(|s| s.to_string()),
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    }
}

/// Extract creature type from a monster entity.
/// Type can be a string directly or an object with a "type" field.
fn extract_creature_type(entity: &Value) -> Option<String> {
    match entity.get("type") {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Object(obj)) => obj
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        _ => None,
    }
}

/// Extract size from a monster entity.
/// Size is typically an array, we take the first element.
fn extract_size(entity: &Value) -> Option<String> {
    match entity.get("size") {
        Some(Value::Array(arr)) => arr.first().and_then(|v| v.as_str()).map(|s| s.to_string()),
        Some(Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

/// Extract ritual flag from a spell entity.
fn extract_ritual(entity: &Value) -> bool {
    entity
        .get("meta")
        .and_then(|m| m.get("ritual"))
        .and_then(|r| r.as_bool())
        .unwrap_or(false)
}

/// Extract concentration flag from a spell entity.
fn extract_concentration(entity: &Value) -> bool {
    entity
        .get("duration")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .any(|d| d.get("concentration").and_then(|c| c.as_bool()).unwrap_or(false))
        })
        .unwrap_or(false)
}

/// Rewrite image paths in book content to use local asset paths.
///
/// 5etools book content contains image references with paths like "img/book/PHB/foo.webp".
/// This function processes the entire book content recursively and updates these paths
/// to work with the local image directory.
fn rewrite_book_image_paths(content: &Value) -> Value {
    match content {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (key, value) in map {
                // Check for href objects with path
                if key == "href" {
                    if let Some(obj) = value.as_object() {
                        if obj.get("type").and_then(|t| t.as_str()) == Some("internal") {
                            if let Some(path) = obj.get("path").and_then(|p| p.as_str()) {
                                // Rewrite the path - strip "img/" prefix if present
                                let new_path = if path.starts_with("img/") {
                                    path.strip_prefix("img/").unwrap_or(path)
                                } else {
                                    path
                                };
                                let mut new_href = serde_json::Map::new();
                                new_href.insert("type".to_string(), Value::String("internal".to_string()));
                                new_href.insert("path".to_string(), Value::String(new_path.to_string()));
                                // Copy any other fields
                                for (k, v) in obj {
                                    if k != "type" && k != "path" {
                                        new_href.insert(k.clone(), v.clone());
                                    }
                                }
                                new_map.insert(key.clone(), Value::Object(new_href));
                                continue;
                            }
                        }
                    }
                }
                new_map.insert(key.clone(), rewrite_book_image_paths(value));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(rewrite_book_image_paths).collect()),
        _ => content.clone(),
    }
}

/// Extract class names from an attunement requirement string.
fn extract_attunement_classes(attune_str: &str) -> Vec<String> {
    let class_names = [
        "artificer",
        "barbarian",
        "bard",
        "cleric",
        "druid",
        "fighter",
        "monk",
        "paladin",
        "ranger",
        "rogue",
        "sorcerer",
        "warlock",
        "wizard",
    ];

    let lower = attune_str.to_lowercase();
    class_names
        .iter()
        .filter(|&&class| lower.contains(class))
        .map(|&s| s.to_string())
        .collect()
}

// === Magic Variant Expansion Helpers ===

/// Check if a base item field matches a requirement key-value pair.
fn base_item_matches_field(base_item: &Value, key: &str, expected: &Value) -> bool {
    let actual = base_item.get(key);

    match expected {
        // Boolean requirement: check if the base item has a truthy value for this key
        Value::Bool(true) => {
            actual.map_or(false, |v| v.as_bool().unwrap_or(false) || v == &Value::Bool(true))
        }
        Value::Bool(false) => {
            actual.map_or(true, |v| v.as_bool() == Some(false))
        }
        // String requirement: exact match against base item field
        Value::String(s) => {
            actual.map_or(false, |v| {
                if let Some(actual_str) = v.as_str() {
                    actual_str == s.as_str()
                } else if let Some(arr) = v.as_array() {
                    // Field might be an array (e.g., "property": ["F", "L"])
                    arr.iter().any(|item| item.as_str() == Some(s.as_str()))
                } else {
                    false
                }
            })
        }
        // Number: exact match
        Value::Number(_) => {
            actual.map_or(false, |v| v == expected)
        }
        _ => false,
    }
}

/// Check if a base item should be excluded based on the variant's excludes block.
fn base_item_excluded(base_item: &Value, excludes: &Value) -> bool {
    let exc_obj = match excludes.as_object() {
        Some(o) => o,
        None => return false,
    };

    for (key, val) in exc_obj {
        match val {
            // { "net": true } — exclude if base item has this boolean flag
            Value::Bool(true) => {
                if base_item.get(key).and_then(|v| v.as_bool()).unwrap_or(false) {
                    return true;
                }
                // Also check name match for string-like excludes
                if key == "name" || key.chars().next().map_or(false, |c| c.is_uppercase()) {
                    // Skip — this is a boolean flag check, not a name check
                }
            }
            // { "name": "Hide Armor" } — exclude by exact name
            Value::String(s) => {
                if let Some(actual) = base_item.get(key).and_then(|v| v.as_str()) {
                    if actual == s.as_str() {
                        return true;
                    }
                }
            }
            // { "property": ["2H", "2H|XPHB"] } — exclude if any value matches
            Value::Array(arr) => {
                if let Some(actual) = base_item.get(key) {
                    for exclude_val in arr {
                        if let Some(exc_str) = exclude_val.as_str() {
                            if let Some(actual_str) = actual.as_str() {
                                if actual_str == exc_str {
                                    return true;
                                }
                            }
                            if let Some(actual_arr) = actual.as_array() {
                                if actual_arr.iter().any(|v| v.as_str() == Some(exc_str)) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    false
}

/// Build an expanded item JSON by merging base item data with variant inherits.
fn build_expanded_item(
    base_item: &Value,
    inherits: &Value,
    expanded_name: &str,
    variant_name: &str,
) -> Value {
    let mut expanded = base_item.clone();

    // Overlay inherits fields onto the base item
    if let (Some(base_obj), Some(inherits_obj)) = (expanded.as_object_mut(), inherits.as_object()) {
        // Metadata fields from inherits that we skip (naming handled separately)
        let skip_keys = ["namePrefix", "nameSuffix", "nameRemove", "reprintedAs", "lootTables"];

        for (key, val) in inherits_obj {
            if skip_keys.contains(&key.as_str()) {
                continue;
            }
            base_obj.insert(key.clone(), val.clone());
        }

        // Set the expanded name
        base_obj.insert("name".to_string(), Value::String(expanded_name.to_string()));

        // Add provenance
        base_obj.insert(
            "_variantName".to_string(),
            Value::String(variant_name.to_string()),
        );

        // Remove baseitem marker if present
        base_obj.remove("_isBaseItem");

        // Resolve template variables like {=bonusWeapon}, {=dmgType} in entries
        resolve_template_variables(base_obj);
    }

    expanded
}

/// Map 5etools damage type codes to human-readable names for use in prose text.
fn expand_damage_type(code: &str) -> &str {
    match code {
        "A" => "Acid",
        "B" => "Bludgeoning",
        "C" => "Cold",
        "F" => "Fire",
        "O" => "Force",
        "L" => "Lightning",
        "N" => "Necrotic",
        "P" => "Piercing",
        "I" => "Poison",
        "Y" => "Psychic",
        "R" => "Radiant",
        "S" => "Slashing",
        "T" => "Thunder",
        other => other,
    }
}

/// Resolve 5etools template variables ({=fieldName}) throughout a JSON object.
/// Replaces {=key} with the string value of that key in the same object.
/// Damage type codes are expanded to full names in prose context.
fn resolve_template_variables(obj: &mut serde_json::Map<String, Value>) {
    // Build lookup of string values for substitution,
    // expanding damage types to readable names for prose
    let lookup: HashMap<String, String> = obj
        .iter()
        .filter_map(|(k, v)| {
            v.as_str().map(|s| {
                let display = if k == "dmgType" {
                    expand_damage_type(s).to_string()
                } else {
                    s.to_string()
                };
                (k.clone(), display)
            })
        })
        .collect();

    // Resolve in all string values recursively
    let keys: Vec<String> = obj.keys().cloned().collect();
    for key in keys {
        if let Some(val) = obj.get(&key).cloned() {
            let resolved = resolve_value_templates(&val, &lookup);
            obj.insert(key, resolved);
        }
    }
}

/// Recursively resolve {=key} templates in a JSON value.
fn resolve_value_templates(val: &Value, lookup: &HashMap<String, String>) -> Value {
    match val {
        Value::String(s) => {
            let mut result = s.clone();
            for (key, replacement) in lookup {
                let pattern = format!("{{={}}}", key);
                result = result.replace(&pattern, replacement);
            }
            Value::String(result)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| resolve_value_templates(v, lookup)).collect())
        }
        Value::Object(obj) => {
            let mut new_obj = obj.clone();
            for (k, v) in obj {
                new_obj.insert(k.clone(), resolve_value_templates(v, lookup));
            }
            Value::Object(new_obj)
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_cr_string() {
        let entity = json!({"cr": "5"});
        assert_eq!(extract_cr(&entity), Some("5".to_string()));
    }

    #[test]
    fn test_extract_cr_object() {
        let entity = json!({"cr": {"cr": "1/2", "lair": "3"}});
        assert_eq!(extract_cr(&entity), Some("1/2".to_string()));
    }

    #[test]
    fn test_extract_cr_number() {
        let entity = json!({"cr": 10});
        assert_eq!(extract_cr(&entity), Some("10".to_string()));
    }

    #[test]
    fn test_extract_creature_type_string() {
        let entity = json!({"type": "humanoid"});
        assert_eq!(extract_creature_type(&entity), Some("humanoid".to_string()));
    }

    #[test]
    fn test_extract_creature_type_object() {
        let entity = json!({"type": {"type": "humanoid", "tags": ["goblinoid"]}});
        assert_eq!(extract_creature_type(&entity), Some("humanoid".to_string()));
    }

    #[test]
    fn test_extract_size_array() {
        let entity = json!({"size": ["M"]});
        assert_eq!(extract_size(&entity), Some("M".to_string()));
    }

    #[test]
    fn test_extract_size_string() {
        let entity = json!({"size": "L"});
        assert_eq!(extract_size(&entity), Some("L".to_string()));
    }

    #[test]
    fn test_extract_ritual() {
        let spell = json!({"meta": {"ritual": true}});
        assert!(extract_ritual(&spell));

        let spell = json!({"meta": {}});
        assert!(!extract_ritual(&spell));
    }

    #[test]
    fn test_extract_concentration() {
        let spell = json!({"duration": [{"type": "timed", "concentration": true}]});
        assert!(extract_concentration(&spell));

        let spell = json!({"duration": [{"type": "instant"}]});
        assert!(!extract_concentration(&spell));
    }

    #[test]
    fn test_extract_attunement_classes() {
        let classes = extract_attunement_classes("by a cleric or paladin");
        assert_eq!(classes.len(), 2);
        assert!(classes.contains(&"cleric".to_string()));
        assert!(classes.contains(&"paladin".to_string()));
    }

    // === Magic Variant Expansion Tests ===

    #[test]
    fn test_base_item_matches_boolean_field() {
        let base = json!({"name": "Shortsword", "weapon": true, "sword": true});
        assert!(base_item_matches_field(&base, "weapon", &json!(true)));
        assert!(base_item_matches_field(&base, "sword", &json!(true)));
        assert!(!base_item_matches_field(&base, "armor", &json!(true)));
    }

    #[test]
    fn test_base_item_matches_string_field() {
        let base = json!({"name": "Shortsword", "type": "M", "weaponCategory": "martial"});
        assert!(base_item_matches_field(&base, "type", &json!("M")));
        assert!(base_item_matches_field(&base, "weaponCategory", &json!("martial")));
        assert!(!base_item_matches_field(&base, "type", &json!("R")));
    }

    #[test]
    fn test_base_item_matches_array_field() {
        // Base item with property as array
        let base = json!({"name": "Greatsword", "property": ["2H", "H"], "weapon": true});
        assert!(base_item_matches_field(&base, "property", &json!("2H")));
        assert!(!base_item_matches_field(&base, "property", &json!("F")));
    }

    #[test]
    fn test_base_item_excluded_boolean() {
        let base = json!({"name": "Net", "net": true, "weapon": true});
        assert!(base_item_excluded(&base, &json!({"net": true})));

        let base2 = json!({"name": "Shortsword", "sword": true, "weapon": true});
        assert!(!base_item_excluded(&base2, &json!({"net": true})));
    }

    #[test]
    fn test_base_item_excluded_string() {
        let base = json!({"name": "Hide Armor", "armor": true});
        assert!(base_item_excluded(&base, &json!({"name": "Hide Armor"})));
        assert!(!base_item_excluded(&base, &json!({"name": "Chain Mail"})));
    }

    #[test]
    fn test_base_item_excluded_array() {
        let base = json!({"name": "Greatsword", "property": ["2H", "H"]});
        assert!(base_item_excluded(&base, &json!({"property": ["2H", "2H|XPHB"]})));
        assert!(!base_item_excluded(&base, &json!({"property": ["F", "L"]})));
    }

    #[test]
    fn test_build_expanded_item() {
        let base = json!({
            "name": "Shortsword",
            "type": "M",
            "dmg1": "1d6",
            "dmgType": "P",
            "weight": 2,
            "weapon": true,
            "sword": true,
            "source": "PHB"
        });
        let inherits = json!({
            "namePrefix": "+1 ",
            "source": "DMG",
            "rarity": "uncommon",
            "bonusWeapon": "+1",
            "entries": ["You have a +1 bonus to attack and damage rolls."]
        });

        let expanded = build_expanded_item(&base, &inherits, "+1 Shortsword", "+1 Weapon");
        assert_eq!(expanded["name"], "+1 Shortsword");
        assert_eq!(expanded["source"], "DMG");
        assert_eq!(expanded["rarity"], "uncommon");
        assert_eq!(expanded["bonusWeapon"], "+1");
        // Base properties preserved
        assert_eq!(expanded["dmg1"], "1d6");
        assert_eq!(expanded["dmgType"], "P");
        assert_eq!(expanded["weight"], 2);
        assert_eq!(expanded["weapon"], true);
        // Provenance
        assert_eq!(expanded["_variantName"], "+1 Weapon");
        // namePrefix should NOT be in the output
        assert!(expanded.get("namePrefix").is_none());
    }

    #[test]
    fn test_template_variable_resolution() {
        let base = json!({"name": "Longbow", "dmgType": "P", "weapon": true, "source": "PHB"});
        let inherits = json!({
            "namePrefix": "+1 ",
            "source": "DMG",
            "bonusWeapon": "+1",
            "entries": [
                "You have a {=bonusWeapon} bonus to attack and damage rolls.",
                "Extra {=dmgType} damage on a crit."
            ]
        });

        let expanded = build_expanded_item(&base, &inherits, "+1 Longbow", "+1 Weapon");
        let entries = expanded["entries"].as_array().unwrap();
        assert_eq!(entries[0], "You have a +1 bonus to attack and damage rolls.");
        assert_eq!(entries[1], "Extra Piercing damage on a crit.");
    }

    #[test]
    fn test_build_expanded_item_suffix() {
        let base = json!({"name": "Arrow", "type": "A", "source": "PHB"});
        let inherits = json!({
            "nameSuffix": " of Slaying",
            "source": "DMG",
            "rarity": "very rare"
        });

        let expanded = build_expanded_item(&base, &inherits, "Arrow of Slaying", "Arrow of Slaying");
        assert_eq!(expanded["name"], "Arrow of Slaying");
        assert_eq!(expanded["rarity"], "very rare");
    }

    #[test]
    fn test_import_result_summary() {
        let mut result = ImportResult::default();
        result.sources_imported.push("PHB".to_string());
        result.sources_imported.push("DMG".to_string());
        result.sources_failed.push(("XGE".to_string(), "Test error".to_string()));
        result.entity_counts.insert("monster".to_string(), 100);
        result.entity_counts.insert("spell".to_string(), 50);
        result.total_entities = 150;

        let summary = result.summary();
        assert!(summary.contains("2 sources"));
        assert!(summary.contains("1 failed"));
        assert!(summary.contains("150 total entities"));
        assert!(summary.contains("monster: 100"));
        assert!(summary.contains("spell: 50"));
        assert!(summary.contains("XGE: Test error"));
    }
}
