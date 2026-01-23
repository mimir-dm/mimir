//! Entity Collection from 5etools Data
//!
//! Provides a generic pattern for collecting D&D entities from 5etools
//! data directories. Supports filtering by source and SRD status.
//!
//! Adapted from mimir-5etools-splitter/src/collector.rs.

use crate::import::discovery::{self, Book, EntityFileInfo, ENTITY_FILE_INFO};
use crate::import::filter::SourceFilter;
use crate::import::srd::{is_srd, process_srd_item, SrdItem};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Key for looking up fluff data.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FluffKey {
    pub entity_type: String,
    pub name: String,
    pub source: String,
}

impl FluffKey {
    pub fn new(entity_type: &str, name: &str, source: &str) -> Self {
        Self {
            entity_type: entity_type.to_string(),
            name: name.to_string(),
            source: source.to_string(),
        }
    }
}

/// Collected entities from a source, organized by entity type.
#[derive(Debug, Default)]
pub struct CollectedEntities {
    /// The source code these entities were collected from (e.g., "PHB", "DMG").
    pub source: String,
    /// Entities organized by type (e.g., "monster" -> [...], "spell" -> [...]).
    pub entities: HashMap<String, Vec<Value>>,
    /// Fluff data indexed by (entity_type, name, source).
    pub fluff: HashMap<FluffKey, Value>,
    /// Total count of entities collected.
    pub total_count: usize,
    /// Book content data (full chapter content for readable books).
    /// The data array contains all sections/chapters with their entries.
    pub book_content: Option<Value>,
    /// Book table of contents from books.json.
    pub book_contents_toc: Option<Value>,
}

impl CollectedEntities {
    /// Create a new empty collection for a source.
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            entities: HashMap::new(),
            fluff: HashMap::new(),
            total_count: 0,
            book_content: None,
            book_contents_toc: None,
        }
    }

    /// Add entities of a specific type.
    pub fn add(&mut self, entity_type: &str, entities: Vec<Value>) {
        let count = entities.len();
        self.entities
            .entry(entity_type.to_string())
            .or_default()
            .extend(entities);
        self.total_count += count;
    }

    /// Get entities of a specific type.
    pub fn get(&self, entity_type: &str) -> Option<&Vec<Value>> {
        self.entities.get(entity_type)
    }

    /// Get count for a specific entity type.
    pub fn count(&self, entity_type: &str) -> usize {
        self.entities.get(entity_type).map_or(0, |v| v.len())
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.total_count == 0
    }

    /// Get all entity types that have been collected.
    pub fn entity_types(&self) -> Vec<&str> {
        self.entities.keys().map(|s| s.as_str()).collect()
    }

    /// Add fluff data for an entity.
    pub fn add_fluff(&mut self, entity_type: &str, name: &str, source: &str, fluff: Value) {
        let key = FluffKey::new(entity_type, name, source);
        self.fluff.insert(key, fluff);
    }

    /// Get fluff data for an entity.
    pub fn get_fluff(&self, entity_type: &str, name: &str, source: &str) -> Option<&Value> {
        let key = FluffKey::new(entity_type, name, source);
        self.fluff.get(&key)
    }

    /// Get fluff count.
    pub fn fluff_count(&self) -> usize {
        self.fluff.len()
    }

    /// Set book content data.
    pub fn set_book_content(&mut self, content: Value) {
        self.book_content = Some(content);
    }

    /// Set book table of contents.
    pub fn set_book_toc(&mut self, toc: Value) {
        self.book_contents_toc = Some(toc);
    }

    /// Check if book content is available.
    pub fn has_book_content(&self) -> bool {
        self.book_content.is_some()
    }
}

/// Collected SRD entities, organized by content type.
#[derive(Debug, Default)]
pub struct CollectedSrdContent {
    /// SRD items organized by content type (e.g., "spells" -> [...]).
    pub content: HashMap<String, Vec<SrdItem>>,
    /// Total count of SRD items collected.
    pub total_count: usize,
}

impl CollectedSrdContent {
    /// Create a new empty SRD collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add SRD items of a specific content type.
    pub fn add(&mut self, content_type: &str, items: Vec<SrdItem>) {
        let count = items.len();
        self.content
            .entry(content_type.to_string())
            .or_default()
            .extend(items);
        self.total_count += count;
    }

    /// Get SRD items of a specific content type.
    pub fn get(&self, content_type: &str) -> Option<&Vec<SrdItem>> {
        self.content.get(content_type)
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.total_count == 0
    }

    /// Get all content types that have been collected.
    pub fn content_types(&self) -> Vec<&str> {
        self.content.keys().map(|s| s.as_str()).collect()
    }
}

/// Collect all entities from a source (book).
///
/// This is the main entry point for collecting content from a specific book
/// or source code.
///
/// # Arguments
/// * `repo_path` - Path to the 5etools data directory
/// * `source` - Source code to collect (e.g., "PHB", "DMG")
///
/// # Example
/// ```ignore
/// let entities = collect_source_entities(&fivetools_path, "PHB")?;
/// println!("Found {} monsters", entities.count("monster"));
/// ```
pub fn collect_source_entities(repo_path: &Path, source: &str) -> Result<CollectedEntities> {
    let mut collected = CollectedEntities::new(source);
    let data_dir = repo_path.join("data");

    // Collect from each known entity file location
    for (entity_type, info) in ENTITY_FILE_INFO {
        let entities = collect_entities_from_files(repo_path, info, source)?;
        if !entities.is_empty() {
            collected.add(entity_type, entities);
        }
    }

    // Also check the common root-level files
    collect_root_level_entities(&mut collected, &data_dir, source)?;

    // Collect fluff data for all entity types
    collect_fluff(&mut collected, &data_dir, source)?;

    // Collect book content (readable chapter content) if available
    collect_book_content(&mut collected, &data_dir, source)?;

    Ok(collected)
}

/// Collect entities from files matching the entity file info pattern.
fn collect_entities_from_files(
    repo_path: &Path,
    info: &EntityFileInfo,
    source: &str,
) -> Result<Vec<Value>> {
    let mut results = Vec::new();

    // Get all matching files for this entity type and source
    let files = discovery::get_matching_files(repo_path, info.dir, info.prefix, source);

    for file_path in files {
        if let Ok(data) = discovery::load_json_file(&file_path) {
            // Filter entities by source from the JSON key
            let filtered = data.filter_key_by_source(info.json_key, source);
            results.extend(filtered);
        }
    }

    Ok(results)
}

/// Collect entities from root-level data files (not in subdirectories).
fn collect_root_level_entities(
    collected: &mut CollectedEntities,
    data_dir: &Path,
    source: &str,
) -> Result<()> {
    // Files that are in the root data directory and contain mixed-source content
    let root_files = [
        ("races.json", &["race", "subrace"][..]),
        ("backgrounds.json", &["background"][..]),
        ("feats.json", &["feat"][..]),
        ("optionalfeatures.json", &["optionalfeature"][..]),
        ("actions.json", &["action"][..]),
        ("conditionsdiseases.json", &["condition", "disease"][..]),
        ("cultsboons.json", &["cult", "boon"][..]),
        ("deities.json", &["deity"][..]),
        ("languages.json", &["language"][..]),
        ("objects.json", &["object"][..]),
        ("rewards.json", &["reward"][..]),
        ("tables.json", &["table"][..]),
        ("trapshazards.json", &["trap", "hazard"][..]),
        ("variantrules.json", &["variantrule"][..]),
        ("vehicles.json", &["vehicle"][..]),
        ("psionics.json", &["psionic"][..]),
        // Items are in root-level files, not a subdirectory
        ("items.json", &["item"][..]),
        ("items-base.json", &["item", "baseitem"][..]),
    ];

    for (filename, entity_types) in root_files {
        let file_path = data_dir.join(filename);
        if file_path.exists() {
            if let Ok(data) = discovery::load_json_file(&file_path) {
                for entity_type in entity_types.iter() {
                    let filtered = data.filter_key_by_source(entity_type, source);
                    if !filtered.is_empty() {
                        collected.add(entity_type, filtered);
                    }
                }
            }
        }
    }

    // Classes are in individual files per class (class-fighter.json, class-wizard.json, etc.)
    // not per source, so we need to scan all class files
    collect_classes_from_directory(collected, data_dir, source)?;

    Ok(())
}

/// Collect classes, subclasses, and their features from the class directory.
///
/// Classes are stored as one file per class (e.g., class-fighter.json),
/// not one file per source. Each class file contains the class with its source,
/// along with classFeature and subclassFeature arrays.
fn collect_classes_from_directory(
    collected: &mut CollectedEntities,
    data_dir: &Path,
    source: &str,
) -> Result<()> {
    let class_dir = data_dir.join("class");
    if !class_dir.exists() {
        return Ok(());
    }

    if let Ok(entries) = std::fs::read_dir(&class_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Only process class-*.json files (not fluff-class-*.json)
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("class-") && filename.ends_with(".json") {
                    if let Ok(data) = discovery::load_json_file(&path) {
                        // Extract classes matching the source
                        let classes = data.filter_key_by_source("class", source);
                        if !classes.is_empty() {
                            collected.add("class", classes);
                        }

                        // Extract subclasses matching the source
                        let subclasses = data.filter_key_by_source("subclass", source);
                        if !subclasses.is_empty() {
                            collected.add("subclass", subclasses);
                        }

                        // Extract class features matching the source
                        let class_features = data.filter_key_by_source("classFeature", source);
                        if !class_features.is_empty() {
                            collected.add("classFeature", class_features);
                        }

                        // Extract subclass features matching the source
                        let subclass_features = data.filter_key_by_source("subclassFeature", source);
                        if !subclass_features.is_empty() {
                            collected.add("subclassFeature", subclass_features);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Collect fluff (lore, flavor text, images) for all entity types from a source.
fn collect_fluff(collected: &mut CollectedEntities, data_dir: &Path, source: &str) -> Result<()> {
    // Fluff file mappings: (fluff_file, fluff_json_key, entity_type)
    // Root-level fluff files
    let root_fluff_files = [
        ("fluff-races.json", "raceFluff", "race"),
        ("fluff-backgrounds.json", "backgroundFluff", "background"),
        ("fluff-feats.json", "featFluff", "feat"),
        ("fluff-items.json", "itemFluff", "item"),
        ("fluff-conditionsdiseases.json", "conditionFluff", "condition"),
        ("fluff-conditionsdiseases.json", "diseaseFluff", "disease"),
        ("fluff-rewards.json", "rewardFluff", "reward"),
        ("fluff-optionalfeatures.json", "optionalfeatureFluff", "optionalfeature"),
    ];

    for (filename, json_key, entity_type) in root_fluff_files {
        let file_path = data_dir.join(filename);
        if file_path.exists() {
            collect_fluff_from_file(collected, &file_path, json_key, entity_type, source)?;
        }
    }

    // Monster fluff - in bestiary subdirectory, named fluff-bestiary-{source}.json
    let monster_fluff_path = data_dir
        .join("bestiary")
        .join(format!("fluff-bestiary-{}.json", source.to_lowercase()));
    if monster_fluff_path.exists() {
        collect_fluff_from_file(collected, &monster_fluff_path, "monsterFluff", "monster", source)?;
    }

    // Spell fluff - in spells subdirectory, named fluff-spells-{source}.json
    let spell_fluff_path = data_dir
        .join("spells")
        .join(format!("fluff-spells-{}.json", source.to_lowercase()));
    if spell_fluff_path.exists() {
        collect_fluff_from_file(collected, &spell_fluff_path, "spellFluff", "spell", source)?;
    }

    // Class fluff - in class subdirectory, named fluff-class-{classname}.json
    collect_class_fluff(collected, data_dir, source)?;

    Ok(())
}

/// Collect fluff from a single fluff file.
fn collect_fluff_from_file(
    collected: &mut CollectedEntities,
    file_path: &Path,
    json_key: &str,
    entity_type: &str,
    source: &str,
) -> Result<()> {
    if let Ok(data) = discovery::load_json_file(file_path) {
        if let Some(fluff_array) = data.get(json_key).and_then(|v| v.as_array()) {
            for fluff_entry in fluff_array {
                // Extract name and source from the fluff entry
                let name = fluff_entry.get("name").and_then(|v| v.as_str());
                let fluff_source = fluff_entry.get("source").and_then(|v| v.as_str());

                if let (Some(name), Some(fluff_source)) = (name, fluff_source) {
                    // Only collect fluff matching the target source
                    if fluff_source == source {
                        collected.add_fluff(entity_type, name, fluff_source, fluff_entry.clone());
                    }
                }
            }
        }
    }
    Ok(())
}

/// Collect class fluff from class directory.
///
/// Class fluff files are named fluff-class-{classname}.json and contain
/// fluff for classes and subclasses.
fn collect_class_fluff(
    collected: &mut CollectedEntities,
    data_dir: &Path,
    source: &str,
) -> Result<()> {
    let class_dir = data_dir.join("class");
    if !class_dir.exists() {
        return Ok(());
    }

    if let Ok(entries) = std::fs::read_dir(&class_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("fluff-class-") && filename.ends_with(".json") {
                    if let Ok(data) = discovery::load_json_file(&path) {
                        // Class fluff
                        if let Some(fluff_array) = data.get("classFluff").and_then(|v| v.as_array())
                        {
                            for fluff_entry in fluff_array {
                                let name = fluff_entry.get("name").and_then(|v| v.as_str());
                                let fluff_source =
                                    fluff_entry.get("source").and_then(|v| v.as_str());

                                if let (Some(name), Some(fluff_source)) = (name, fluff_source) {
                                    if fluff_source == source {
                                        collected.add_fluff(
                                            "class",
                                            name,
                                            fluff_source,
                                            fluff_entry.clone(),
                                        );
                                    }
                                }
                            }
                        }

                        // Subclass fluff
                        if let Some(fluff_array) =
                            data.get("subclassFluff").and_then(|v| v.as_array())
                        {
                            for fluff_entry in fluff_array {
                                let name = fluff_entry.get("name").and_then(|v| v.as_str());
                                let fluff_source =
                                    fluff_entry.get("source").and_then(|v| v.as_str());

                                if let (Some(name), Some(fluff_source)) = (name, fluff_source) {
                                    if fluff_source == source {
                                        collected.add_fluff(
                                            "subclass",
                                            name,
                                            fluff_source,
                                            fluff_entry.clone(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Collect book content (readable chapters/sections) for a source.
///
/// Book content files are located at `data/book/book-{source}.json` and contain
/// the full chapter content with entries that can be rendered in a reader view.
fn collect_book_content(
    collected: &mut CollectedEntities,
    data_dir: &Path,
    source: &str,
) -> Result<()> {
    // Book content file path: data/book/book-{source}.json (lowercase)
    let book_file = data_dir
        .join("book")
        .join(format!("book-{}.json", source.to_lowercase()));

    if !book_file.exists() {
        return Ok(());
    }

    if let Ok(data) = discovery::load_json_file(&book_file) {
        // The book content file has a "data" array containing all sections
        if let Some(book_data) = data.get("data") {
            collected.set_book_content(book_data.clone());
        }
    }

    // Also get the table of contents from books.json metadata
    let books_file = data_dir.join("books.json");
    if books_file.exists() {
        if let Ok(books_data) = discovery::load_json_file(&books_file) {
            if let Some(books_array) = books_data.get("book").and_then(|v| v.as_array()) {
                for book in books_array {
                    // Find the book matching our source
                    if let Some(book_source) = book.get("source").and_then(|v| v.as_str()) {
                        if book_source == source {
                            // Get the contents (table of contents)
                            if let Some(contents) = book.get("contents") {
                                collected.set_book_toc(contents.clone());
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Collect all available books/sources from a 5etools directory.
///
/// Returns metadata for all books that have content available.
pub fn discover_available_sources(repo_path: &Path) -> Result<Vec<Book>> {
    discovery::load_all_books(repo_path)
}

/// Collect entities of a specific type from any source.
///
/// Useful when you want all monsters, all spells, etc. regardless of source.
///
/// # Arguments
/// * `repo_path` - Path to the 5etools data directory
/// * `entity_type` - Type of entity to collect (e.g., "monster", "spell")
pub fn collect_all_of_type(repo_path: &Path, entity_type: &str) -> Result<Vec<Value>> {
    let mut results = Vec::new();
    let data_dir = repo_path.join("data");

    // Find the file info for this entity type
    if let Some((_, info)) = ENTITY_FILE_INFO
        .iter()
        .find(|(name, _)| *name == entity_type)
    {
        // For entities with dedicated directories, scan all files
        let entity_dir = if info.dir == "." {
            data_dir.clone()
        } else {
            data_dir.join(info.dir)
        };

        if entity_dir.exists() {
            // Check for directory-based files
            if info.dir != "." {
                if let Ok(entries) = std::fs::read_dir(&entity_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map_or(false, |ext| ext == "json")
                            && path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .map_or(false, |n| n.starts_with(info.prefix))
                        {
                            if let Ok(data) = discovery::load_json_file(&path) {
                                if let Some(array) =
                                    data.get(info.json_key).and_then(|v| v.as_array())
                                {
                                    results.extend(array.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Check for root-level file
            let root_file = data_dir.join(format!("{}.json", info.prefix));
            if root_file.exists() {
                if let Ok(data) = discovery::load_json_file(&root_file) {
                    if let Some(array) = data.get(info.json_key).and_then(|v| v.as_array()) {
                        results.extend(array.clone());
                    }
                }
            }
        }
    }

    Ok(results)
}

/// Collect all SRD content from a 5etools repository.
///
/// Scans all data files and extracts items marked as SRD content.
/// Items may be transformed (renamed) according to SRD requirements.
pub fn collect_srd_content(repo_path: &Path) -> Result<CollectedSrdContent> {
    let mut collected = CollectedSrdContent::new();
    let data_dir = repo_path.join("data");

    // Data files to scan for SRD content
    let data_files = [
        "bestiary/bestiary-mm.json",
        "bestiary/bestiary-phb.json",
        "items/items.json",
        "items/items-base.json",
        "races.json",
        "backgrounds.json",
        "feats.json",
        "rewards.json",
        "objects.json",
        "trapshazards.json",
        "actions.json",
        "conditionsdiseases.json",
        "cultsboons.json",
        "deities.json",
        "languages.json",
        "tables.json",
        "variantrules.json",
        "vehicles.json",
        "optionalfeatures.json",
    ];

    // Process each data file
    for data_file in &data_files {
        let file_path = data_dir.join(data_file);
        if file_path.exists() {
            if let Ok(data) = discovery::load_json_file(&file_path) {
                let srd_content = crate::import::srd::extract_all_srd_content(&data);
                for (content_type, items) in srd_content {
                    collected.add(&content_type, items);
                }
            }
        }
    }

    // Scan class directory
    let class_dir = data_dir.join("class");
    if class_dir.exists() {
        collect_srd_from_directory(&mut collected, &class_dir, "class-")?;
    }

    // Scan spells directory
    let spells_dir = data_dir.join("spells");
    if spells_dir.exists() {
        collect_srd_from_directory(&mut collected, &spells_dir, "spells-")?;
    }

    Ok(collected)
}

/// Collect SRD content from all JSON files in a directory with a given prefix.
fn collect_srd_from_directory(
    collected: &mut CollectedSrdContent,
    dir: &Path,
    prefix: &str,
) -> Result<()> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path.extension().map_or(false, |ext| ext == "json")
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map_or(false, |n| n.starts_with(prefix))
            {
                if let Ok(data) = discovery::load_json_file(&path) {
                    let srd_content = crate::import::srd::extract_all_srd_content(&data);
                    for (content_type, items) in srd_content {
                        collected.add(&content_type, items);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Filter a list of entities to only include SRD content.
///
/// Useful when you've already collected entities and want to filter them.
pub fn filter_srd_only(entities: &[Value]) -> Vec<SrdItem> {
    entities.iter().filter_map(process_srd_item).collect()
}

/// Check if a single entity is SRD content.
pub fn entity_is_srd(entity: &Value) -> bool {
    is_srd(entity)
}

/// Get a summary of collected entities.
pub fn summarize_collection(collected: &CollectedEntities) -> String {
    let mut summary = format!("Source: {}\n", collected.source);
    summary.push_str(&format!("Total entities: {}\n", collected.total_count));
    summary.push_str("By type:\n");

    let mut types: Vec<_> = collected.entities.iter().collect();
    types.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));

    for (entity_type, entities) in types {
        summary.push_str(&format!("  {}: {}\n", entity_type, entities.len()));
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_collected_entities_new() {
        let collected = CollectedEntities::new("PHB");
        assert_eq!(collected.source, "PHB");
        assert!(collected.is_empty());
        assert_eq!(collected.total_count, 0);
    }

    #[test]
    fn test_collected_entities_add() {
        let mut collected = CollectedEntities::new("PHB");

        collected.add(
            "monster",
            vec![json!({"name": "Goblin"}), json!({"name": "Orc"})],
        );

        assert_eq!(collected.count("monster"), 2);
        assert_eq!(collected.total_count, 2);
        assert!(!collected.is_empty());
    }

    #[test]
    fn test_collected_entities_get() {
        let mut collected = CollectedEntities::new("PHB");
        collected.add("spell", vec![json!({"name": "Fireball"})]);

        let spells = collected.get("spell");
        assert!(spells.is_some());
        assert_eq!(spells.unwrap().len(), 1);

        let monsters = collected.get("monster");
        assert!(monsters.is_none());
    }

    #[test]
    fn test_collected_entities_entity_types() {
        let mut collected = CollectedEntities::new("PHB");
        collected.add("monster", vec![json!({"name": "Goblin"})]);
        collected.add("spell", vec![json!({"name": "Fireball"})]);

        let types = collected.entity_types();
        assert_eq!(types.len(), 2);
        assert!(types.contains(&"monster"));
        assert!(types.contains(&"spell"));
    }

    #[test]
    fn test_collected_srd_content_new() {
        let collected = CollectedSrdContent::new();
        assert!(collected.is_empty());
        assert_eq!(collected.total_count, 0);
    }

    #[test]
    fn test_filter_srd_only() {
        let entities = vec![
            json!({"name": "Fireball", "srd": true}),
            json!({"name": "Custom Spell", "srd": false}),
            json!({"name": "Aid", "basicRules": true}),
            json!({"name": "Proprietary"}),
        ];

        let srd_items = filter_srd_only(&entities);
        assert_eq!(srd_items.len(), 2);
    }

    #[test]
    fn test_entity_is_srd() {
        assert!(entity_is_srd(&json!({"srd": true})));
        assert!(entity_is_srd(&json!({"srd": "Renamed Item"})));
        assert!(entity_is_srd(&json!({"basicRules": true})));
        assert!(!entity_is_srd(&json!({"srd": false})));
        assert!(!entity_is_srd(&json!({"name": "Test"})));
    }

    #[test]
    fn test_summarize_collection() {
        let mut collected = CollectedEntities::new("PHB");
        collected.add(
            "monster",
            vec![json!({"name": "A"}), json!({"name": "B"})],
        );
        collected.add("spell", vec![json!({"name": "C"})]);

        let summary = summarize_collection(&collected);
        assert!(summary.contains("Source: PHB"));
        assert!(summary.contains("Total entities: 3"));
        assert!(summary.contains("monster: 2"));
        assert!(summary.contains("spell: 1"));
    }
}
