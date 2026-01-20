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

/// Collected entities from a source, organized by entity type.
#[derive(Debug, Default)]
pub struct CollectedEntities {
    /// The source code these entities were collected from (e.g., "PHB", "DMG").
    pub source: String,
    /// Entities organized by type (e.g., "monster" -> [...], "spell" -> [...]).
    pub entities: HashMap<String, Vec<Value>>,
    /// Total count of entities collected.
    pub total_count: usize,
}

impl CollectedEntities {
    /// Create a new empty collection for a source.
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            entities: HashMap::new(),
            total_count: 0,
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
