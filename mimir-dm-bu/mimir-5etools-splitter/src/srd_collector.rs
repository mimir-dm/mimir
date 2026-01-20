//! SRD content collection and archive generation
//!
//! This module handles collecting all SRD content from across the entire
//! 5etools dataset and organizing it into a single SRD archive.

use crate::srd_filter::{extract_all_srd_content, SrdItem};
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Container for all collected SRD content.
pub struct SrdContent {
    /// Map of file paths to file contents.
    pub files: HashMap<String, Vec<u8>>,
    /// Metadata about the SRD archive.
    pub metadata: SrdMetadata,
}

/// Metadata for the SRD archive.
#[derive(Debug)]
pub struct SrdMetadata {
    /// Display name of the archive.
    pub name: String,
    /// Unique identifier.
    pub id: String,
    /// Source code.
    pub source: String,
    /// Description of the archive contents.
    pub description: String,
    /// Version string.
    pub version: String,
    /// Total number of items in the archive.
    pub total_items: usize,
    /// Count of items by content type.
    pub content_summary: HashMap<String, usize>,
}

impl Default for SrdContent {
    fn default() -> Self {
        Self::new()
    }
}

impl SrdContent {
    /// Creates a new empty SRD content container.
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            metadata: SrdMetadata {
                name: "System Reference Document".to_string(),
                id: "srd".to_string(),
                source: "SRD".to_string(),
                description: "Open Game Content from the D&D 5th Edition System Reference Document"
                    .to_string(),
                version: "1.0".to_string(),
                total_items: 0,
                content_summary: HashMap::new(),
            },
        }
    }

    /// Add JSON content to the archive
    pub fn add_json(&mut self, path: &str, value: &Value) -> Result<()> {
        let json_str = serde_json::to_string_pretty(value)?;
        self.files.insert(path.to_string(), json_str.into_bytes());
        Ok(())
    }

    /// Finalize the content by generating metadata
    pub fn finalize(&mut self) -> Result<()> {
        // Create metadata.json
        let metadata_json = json!({
            "name": self.metadata.name,
            "id": self.metadata.id,
            "source": self.metadata.source,
            "description": self.metadata.description,
            "version": self.metadata.version,
            "totalItems": self.metadata.total_items,
            "contentSummary": self.metadata.content_summary,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "legalNotice": "This content is part of the System Reference Document and is Open Game Content under the Open Gaming License v1.0a.",
            "generatedBy": "mimir-5etools-splitter SRD extractor"
        });

        self.add_json("metadata.json", &metadata_json)?;

        // Create legal notice
        let legal_notice = r#"OPEN GAME LICENSE Version 1.0a

This material is Open Game Content, and is licensed for public use under the terms of the Open Game License v1.0a.

This archive contains only content that is part of the System Reference Document (SRD) and is designated as Open Game Content. This content may be freely used, modified, and distributed under the terms of the Open Gaming License.

For the complete Open Gaming License text, please refer to the official Wizards of the Coast website.
"#;

        self.files
            .insert("LICENSE.txt".to_string(), legal_notice.as_bytes().to_vec());

        Ok(())
    }
}

/// Collect all SRD content from a 5etools repository
pub fn collect_srd_content(repo_path: &Path) -> Result<SrdContent> {
    let mut content = SrdContent::new();
    let data_dir = repo_path.join("data");

    // Collect content from all major data files
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

    let mut all_srd_content: HashMap<String, Vec<SrdItem>> = HashMap::new();

    // Process each data file
    for data_file in &data_files {
        let file_path = data_dir.join(data_file);

        if file_path.exists() {
            match fs::read_to_string(&file_path) {
                Ok(file_content) => {
                    match serde_json::from_str::<Value>(&file_content) {
                        Ok(data) => {
                            let srd_items = extract_all_srd_content(&data);

                            // Merge into all_srd_content
                            for (content_type, items) in srd_items {
                                all_srd_content
                                    .entry(content_type)
                                    .or_default()
                                    .extend(items);
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse JSON from {}: {}", data_file, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read file {}: {}", data_file, e);
                }
            }
        }
    }

    // Collect class files from the class directory
    let class_dir = data_dir.join("class");
    if class_dir.exists() {
        if let Ok(entries) = fs::read_dir(&class_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path.extension().is_some_and(|ext| ext == "json")
                    && path
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().starts_with("class-"))
                {
                    match fs::read_to_string(&path) {
                        Ok(file_content) => {
                            match serde_json::from_str::<Value>(&file_content) {
                                Ok(data) => {
                                    let srd_items = extract_all_srd_content(&data);

                                    // Merge into all_srd_content
                                    for (content_type, items) in srd_items {
                                        all_srd_content
                                            .entry(content_type)
                                            .or_default()
                                            .extend(items);
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Warning: Failed to parse JSON from {:?}: {}",
                                        path, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to read file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    }

    // Collect spell files from the spells directory
    let spells_dir = data_dir.join("spells");
    if spells_dir.exists() {
        if let Ok(entries) = fs::read_dir(&spells_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path.extension().is_some_and(|ext| ext == "json")
                    && path
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().starts_with("spells-"))
                {
                    match fs::read_to_string(&path) {
                        Ok(file_content) => {
                            match serde_json::from_str::<Value>(&file_content) {
                                Ok(data) => {
                                    let srd_items = extract_all_srd_content(&data);

                                    // Merge into all_srd_content
                                    for (content_type, items) in srd_items {
                                        all_srd_content
                                            .entry(content_type)
                                            .or_default()
                                            .extend(items);
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Warning: Failed to parse JSON from {:?}: {}",
                                        path, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to read file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    }

    // Generate output files for each content type
    for (content_type, items) in all_srd_content {
        if !items.is_empty() {
            let content_json = create_content_json(&content_type, &items)?;
            let output_path = format!("{}/srd.json", content_type);
            content.add_json(&output_path, &content_json)?;

            // Update metadata
            content
                .metadata
                .content_summary
                .insert(content_type, items.len());
            content.metadata.total_items += items.len();
        }
    }

    // Finalize with metadata and legal files
    content.finalize()?;

    Ok(content)
}

/// Create a JSON structure for a specific content type
fn create_content_json(content_type: &str, items: &[SrdItem]) -> Result<Value> {
    let items_json: Vec<Value> = items.iter().map(|item| item.data.clone()).collect();

    // Map content types to their JSON structure keys
    let json_key = match content_type {
        "spells" => "spell",
        "monsters" => "monster",
        "items" => "item",
        "races" => "race",
        "classes" => "class",
        "subclasses" => "subclass",
        "backgrounds" => "background",
        "feats" => "feat",
        "optionalfeatures" => "optionalfeature",
        "rewards" => "reward",
        "objects" => "object",
        "traps" => "trap",
        "hazards" => "hazard",
        "actions" => "action",
        "conditions" => "condition",
        "diseases" => "disease",
        "npcs" => "npc",
        "vehicles" => "vehicle",
        "deities" => "deity",
        "languages" => "language",
        "tables" => "table",
        "variantrules" => "variantrule",
        "cults" => "cult",
        "boons" => "boon",
        _ => content_type, // fallback
    };

    Ok(json!({
        json_key: items_json
    }))
}

/// Generate summary report of SRD content
pub fn generate_srd_summary(content: &SrdContent) -> String {
    let mut summary = String::new();
    summary.push_str("=== SRD Content Summary ===\n\n");
    summary.push_str(&format!("Total Items: {}\n", content.metadata.total_items));
    summary.push_str("Content by Type:\n");

    let mut sorted_content: Vec<_> = content.metadata.content_summary.iter().collect();
    sorted_content.sort_by_key(|(_, count)| *count);
    sorted_content.reverse();

    for (content_type, count) in sorted_content {
        summary.push_str(&format!("  • {}: {} items\n", content_type, count));
    }

    summary.push_str(&format!(
        "\nGenerated: {}\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srd_filter::SrdItem;
    use serde_json::json;

    #[test]
    fn test_create_content_json() {
        let items = vec![
            SrdItem {
                data: json!({"name": "Fireball", "level": 3}),
                original_name: None,
                srd_name: None,
                was_renamed: false,
            },
            SrdItem {
                data: json!({"name": "Magic Missile", "level": 1}),
                original_name: None,
                srd_name: None,
                was_renamed: false,
            },
        ];

        let result = create_content_json("spells", &items).unwrap();
        let spells = result["spell"].as_array().unwrap();

        assert_eq!(spells.len(), 2);
        assert_eq!(spells[0]["name"], "Fireball");
        assert_eq!(spells[1]["name"], "Magic Missile");
    }

    #[test]
    fn test_srd_content_creation() {
        let mut content = SrdContent::new();

        let test_json = json!({"test": "value"});
        content.add_json("test.json", &test_json).unwrap();

        assert!(content.files.contains_key("test.json"));
    }
}
