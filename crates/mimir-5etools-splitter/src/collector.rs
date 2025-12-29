//! Book content collection for archive creation.

use crate::filter::SourceFilter;
use crate::magic_variants;
use crate::parser::{self, Book};
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Container for collected book content and associated files.
pub struct BookContent {
    /// The book metadata.
    pub book: Book,
    /// Map of file paths to file contents.
    pub files: HashMap<String, Vec<u8>>,
}

impl BookContent {
    /// Creates a new book content container.
    pub fn new(book: Book) -> Self {
        Self {
            book,
            files: HashMap::new(),
        }
    }

    /// Add JSON content to the archive
    pub fn add_json(&mut self, path: &str, value: &Value) -> Result<()> {
        let json_str = serde_json::to_string_pretty(value)?;
        self.files.insert(path.to_string(), json_str.into_bytes());
        Ok(())
    }

    /// Add raw file content to the archive
    pub fn add_file(&mut self, path: &str, content: Vec<u8>) {
        self.files.insert(path.to_string(), content);
    }
}

/// Collect all content for a specific book
pub fn collect_book_content(book: &Book, repo_path: &Path) -> Result<BookContent> {
    let mut content = BookContent::new(book.clone());
    let source = &book.source;
    let data_dir = repo_path.join("data");

    // Add metadata
    let metadata = json!({
        "name": book.name,
        "id": book.id,
        "source": book.source,
        "group": book.group,
        "published": book.published,
        "author": book.author,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    content.add_json("metadata.json", &metadata)?;

    // Collect book narrative content
    collect_book_files(&mut content, &data_dir, "book", "book", source)?;

    // Collect adventure content
    collect_book_files(&mut content, &data_dir, "adventure", "adventure", source)?;

    // Collect bestiary content
    collect_book_files(&mut content, &data_dir, "bestiary", "bestiary", source)?;

    // Collect spell content
    collect_spell_files(&mut content, &data_dir, source)?;

    // Collect class content with features
    collect_class_content(&mut content, &data_dir, source)?;

    // Collect items (filtered from both items.json and items-base.json)
    collect_filtered_items(&mut content, &data_dir, source)?;

    // Collect races (filtered)
    collect_filtered_races(&mut content, &data_dir, source)?;

    // Collect backgrounds (filtered)
    collect_filtered_backgrounds(&mut content, &data_dir, source)?;

    // Collect feats (filtered)
    collect_filtered_feats(&mut content, &data_dir, source)?;

    // Collect optional features (filtered)
    collect_filtered_optfeatures(&mut content, &data_dir, source)?;

    // Collect additional content types with source filtering
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "actions.json",
        "action",
        "actions",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "conditionsdiseases.json",
        "condition",
        "conditions",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "conditionsdiseases.json",
        "disease",
        "diseases",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "cultsboons.json",
        "cult",
        "cults",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "cultsboons.json",
        "boon",
        "boons",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "deities.json",
        "deity",
        "deities",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "languages.json",
        "language",
        "languages",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "objects.json",
        "object",
        "objects",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "rewards.json",
        "reward",
        "rewards",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "tables.json",
        "table",
        "tables",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "trapshazards.json",
        "trap",
        "traps",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "trapshazards.json",
        "hazard",
        "hazards",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "variantrules.json",
        "variantrule",
        "variantrules",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "vehicles.json",
        "vehicle",
        "vehicles",
    )?;
    collect_filtered_generic(
        &mut content,
        &data_dir,
        source,
        "psionics.json",
        "psionic",
        "psionics",
    )?;

    // Collect images
    collect_images(&mut content, repo_path, source)?;

    Ok(content)
}

/// Collect files for a specific type and source
fn collect_book_files(
    content: &mut BookContent,
    data_dir: &Path,
    dir_name: &str,
    file_prefix: &str,
    source: &str,
) -> Result<()> {
    let files =
        parser::get_matching_files(data_dir.parent().unwrap(), dir_name, file_prefix, source);

    for file_path in files {
        if let Ok(data) = parser::load_json_file(&file_path) {
            let relative_path = file_path
                .strip_prefix(data_dir)
                .unwrap_or(&file_path)
                .to_string_lossy();
            content.add_json(&relative_path, &data)?;
        }
    }

    Ok(())
}

/// Collect spell files with fluff
fn collect_spell_files(content: &mut BookContent, data_dir: &Path, source: &str) -> Result<()> {
    let spells_dir = data_dir.join("spells");
    if !spells_dir.exists() {
        return Ok(());
    }

    // Main spell file
    let spell_file = spells_dir.join(format!("spells-{}.json", source.to_lowercase()));
    if spell_file.exists() {
        if let Ok(data) = parser::load_json_file(&spell_file) {
            content.add_json(
                &format!("spells/spells-{}.json", source.to_lowercase()),
                &data,
            )?;
        }
    }

    // Fluff spell file
    let fluff_file = spells_dir.join(format!("fluff-spells-{}.json", source.to_lowercase()));
    if fluff_file.exists() {
        if let Ok(data) = parser::load_json_file(&fluff_file) {
            content.add_json(
                &format!("spells/fluff-spells-{}.json", source.to_lowercase()),
                &data,
            )?;
        }
    }

    // Spell-class associations from sources.json
    // Extract only the entries for this book's source
    let sources_file = spells_dir.join("sources.json");
    if sources_file.exists() {
        if let Ok(sources_data) = parser::load_json_file(&sources_file) {
            if let Some(source_spells) = sources_data.get(source) {
                // Create a filtered sources.json with just this book's spell-class mappings
                let filtered = json!({
                    source: source_spells.clone()
                });
                content.add_json("spells/sources.json", &filtered)?;
            }
        }
    }

    Ok(())
}

/// Collect class content with associated features
fn collect_class_content(content: &mut BookContent, data_dir: &Path, source: &str) -> Result<()> {
    let class_dir = data_dir.join("class");
    if !class_dir.exists() {
        return Ok(());
    }

    let mut all_classes = Vec::new();
    let mut all_subclasses = Vec::new();
    let mut all_class_features = Vec::new();
    let mut all_subclass_features = Vec::new();

    // Read all class files
    for entry in fs::read_dir(&class_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(data) = parser::load_json_file(&path) {
                // Extract classes matching the source
                if let Some(classes) = data.get("class").and_then(|v| v.as_array()) {
                    for class in classes {
                        if let Some(class_source) = class.get("source").and_then(|v| v.as_str()) {
                            if class_source == source {
                                all_classes.push(class.clone());
                            }
                        }
                    }
                }

                // Extract subclasses matching the source
                if let Some(subclasses) = data.get("subclass").and_then(|v| v.as_array()) {
                    for subclass in subclasses {
                        if let Some(subclass_source) =
                            subclass.get("source").and_then(|v| v.as_str())
                        {
                            if subclass_source == source {
                                all_subclasses.push(subclass.clone());
                            }
                        }
                    }
                }

                // Extract class features for classes from this source
                if let Some(features) = data.get("classFeature").and_then(|v| v.as_array()) {
                    for feature in features {
                        if let Some(feature_source) = feature.get("source").and_then(|v| v.as_str())
                        {
                            if feature_source == source {
                                all_class_features.push(feature.clone());
                            }
                        }
                    }
                }

                // Extract subclass features for subclasses from this source
                if let Some(features) = data.get("subclassFeature").and_then(|v| v.as_array()) {
                    for feature in features {
                        if let Some(feature_source) = feature.get("source").and_then(|v| v.as_str())
                        {
                            if feature_source == source {
                                all_subclass_features.push(feature.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Save class data
    if !all_classes.is_empty() || !all_subclasses.is_empty() {
        let mut result = json!({});
        if !all_classes.is_empty() {
            result["class"] = json!(all_classes);
        }
        if !all_subclasses.is_empty() {
            result["subclass"] = json!(all_subclasses);
        }
        content.add_json(&format!("class/{}.json", source.to_lowercase()), &result)?;
    }

    // Save class features
    if !all_class_features.is_empty() {
        let result = json!({ "classFeature": all_class_features });
        content.add_json(
            &format!("class/features-{}.json", source.to_lowercase()),
            &result,
        )?;
    }

    // Save subclass features
    if !all_subclass_features.is_empty() {
        let result = json!({ "subclassFeature": all_subclass_features });
        content.add_json(
            &format!("class/subclass-features-{}.json", source.to_lowercase()),
            &result,
        )?;
    }

    // Collect class and subclass fluff
    let mut all_class_fluff = Vec::new();
    let mut all_subclass_fluff = Vec::new();

    // Check for fluff files
    for entry in fs::read_dir(&class_dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        if filename.starts_with("fluff-class-") && filename.ends_with(".json") {
            if let Ok(data) = parser::load_json_file(&path) {
                // Filter class fluff by source
                if let Some(class_fluff) = data.get("classFluff").and_then(|v| v.as_array()) {
                    for fluff in class_fluff {
                        if let Some(fluff_source) = fluff.get("source").and_then(|v| v.as_str()) {
                            if fluff_source == source {
                                all_class_fluff.push(fluff.clone());
                            }
                        }
                    }
                }

                // Filter subclass fluff by source
                if let Some(subclass_fluff) = data.get("subclassFluff").and_then(|v| v.as_array()) {
                    for fluff in subclass_fluff {
                        if let Some(fluff_source) = fluff.get("source").and_then(|v| v.as_str()) {
                            if fluff_source == source {
                                all_subclass_fluff.push(fluff.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Save class fluff
    if !all_class_fluff.is_empty() {
        let result = json!({ "classFluff": all_class_fluff });
        content.add_json(
            &format!("class/fluff-{}.json", source.to_lowercase()),
            &result,
        )?;
    }

    // Save subclass fluff
    if !all_subclass_fluff.is_empty() {
        let result = json!({ "subclassFluff": all_subclass_fluff });
        content.add_json(
            &format!("class/subclass-fluff-{}.json", source.to_lowercase()),
            &result,
        )?;
    }

    Ok(())
}

/// Collect filtered items (from both items.json and items-base.json)
fn collect_filtered_items(content: &mut BookContent, data_dir: &Path, source: &str) -> Result<()> {
    let mut all_items = Vec::new();
    let mut base_items = Vec::new();

    // Load itemEntry templates from items-base.json
    let mut item_entry_templates = HashMap::new();
    let items_base_file = data_dir.join("items-base.json");
    if items_base_file.exists() {
        let data = parser::load_json_file(&items_base_file)?;
        if let Some(item_entries) = data.get("itemEntry").and_then(|v| v.as_array()) {
            for entry in item_entries {
                if let (Some(name), Some(source_val)) = (
                    entry.get("name").and_then(|v| v.as_str()),
                    entry.get("source").and_then(|v| v.as_str()),
                ) {
                    let key = format!("{}|{}", name, source_val);
                    item_entry_templates.insert(key, entry.clone());
                }
            }
        }
    }

    // Collect from items.json (magic items, etc.)
    let items_file = data_dir.join("items.json");
    if items_file.exists() {
        let data = parser::load_json_file(&items_file)?;
        let filtered = parser::filter_by_source(&data, source, "item");
        all_items.extend(filtered);

        // Also collect itemGroup entries
        if let Some(item_groups) = data.get("itemGroup").and_then(|v| v.as_array()) {
            for group in item_groups {
                if let Some(group_source) = group.get("source").and_then(|v| v.as_str()) {
                    if group_source == source {
                        let mut group_item = group.clone();
                        // Mark as item group (similar to 5etools)
                        group_item["_isItemGroup"] = json!(true);
                        all_items.push(group_item);
                    }
                }
            }
        }
    }

    // Collect from items-base.json (weapons, armor, basic gear)
    let items_base_file = data_dir.join("items-base.json");
    if items_base_file.exists() {
        let data = parser::load_json_file(&items_base_file)?;
        let filtered = parser::filter_by_source(&data, source, "baseitem");
        base_items.extend(filtered.clone());
        all_items.extend(filtered);
    }

    // For DMG specifically, expand magic variants from ALL base items
    if source == "DMG" {
        match magic_variants::load_magic_variants(data_dir) {
            Ok(variants) => {
                // Add the generic variants themselves to the output
                for variant in &variants {
                    let mut variant_item = json!({
                        "name": variant.name,
                        "source": "DMG",
                        "_category": "Generic Variant",
                        "_isGenericVariant": true
                    });

                    // Add inherited properties to the generic variant
                    if let Some(inherits) = &variant.inherits {
                        if let Some(rarity) = &inherits.rarity {
                            variant_item["rarity"] = json!(rarity);
                        }
                        if let Some(tier) = &inherits.tier {
                            variant_item["tier"] = json!(tier);
                        }
                        if let Some(page) = &inherits.page {
                            variant_item["page"] = json!(page);
                        }
                        if let Some(source) = &inherits.source {
                            variant_item["source"] = json!(source);
                        }
                        if let Some(bonus_weapon) = &inherits.bonus_weapon {
                            variant_item["bonusWeapon"] = json!(bonus_weapon);
                        }
                        if let Some(bonus_weapon_attack) = &inherits.bonus_weapon_attack {
                            variant_item["bonusWeaponAttack"] = json!(bonus_weapon_attack);
                        }
                        if let Some(bonus_ac) = &inherits.bonus_ac {
                            variant_item["bonusAc"] = json!(bonus_ac);
                        }
                        if let Some(bonus_weapon_damage) = &inherits.bonus_weapon_damage {
                            variant_item["bonusWeaponDamage"] = json!(bonus_weapon_damage);
                        }
                    }

                    // For generic variants, use variant.entries if present, otherwise inherits.entries
                    // Process template variables in the entries
                    if let Some(entries) = &variant.entries {
                        let processed =
                            magic_variants::process_entries_templates(entries, &variant_item);
                        variant_item["entries"] = json!(processed);
                    } else if let Some(inherits) = &variant.inherits {
                        if let Some(entries) = &inherits.entries {
                            let processed =
                                magic_variants::process_entries_templates(entries, &variant_item);
                            variant_item["entries"] = json!(processed);
                        }
                    }

                    if let Some(variant_type) = &variant.variant_type {
                        variant_item["type"] = json!(variant_type);
                    }

                    all_items.push(variant_item);
                }

                // Load ALL base items from items-base.json (not just DMG source)
                let items_base_file = data_dir.join("items-base.json");
                if items_base_file.exists() {
                    let data = parser::load_json_file(&items_base_file)?;
                    if let Some(all_base_items) = data.get("baseitem").and_then(|v| v.as_array()) {
                        let all_base_items: Vec<_> = all_base_items.to_vec();

                        match magic_variants::expand_magic_variants(&all_base_items, &variants) {
                            Ok(expanded_items) => {
                                if !expanded_items.is_empty() {
                                    let count = expanded_items.len();
                                    // Add expanded magic items to the main collection
                                    all_items.extend(expanded_items);
                                    println!(
                                        "✨ Generated {} magic item variants for {}",
                                        count, source
                                    );
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to expand magic variants for {}: {}",
                                    source, e
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to load magic variants for {}: {}",
                    source, e
                );
            }
        }
    }

    // Deduplicate items based on name + source combination
    if !all_items.is_empty() {
        let mut seen = HashSet::new();
        let mut deduped_items = Vec::new();

        for item in all_items {
            // Create a unique key from name + source
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let item_source = item
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or(source);
            let key = format!("{}|{}", name, item_source);

            if seen.insert(key) {
                deduped_items.push(item);
            }
        }

        // Resolve {#itemEntry} references
        let resolved_items = resolve_item_entry_references(deduped_items, &item_entry_templates);

        // Log diagnostic info about resolved items
        let items_with_resolved_entries = resolved_items
            .iter()
            .filter(|item| {
                if let Some(entries) = item.get("entries").and_then(|v| v.as_array()) {
                    entries.iter().any(|e| {
                        if let Some(s) = e.as_str() {
                            !s.starts_with("{#itemEntry")
                        } else {
                            true
                        }
                    })
                } else {
                    false
                }
            })
            .count();

        let items_with_unresolved_entries = resolved_items
            .iter()
            .filter(|item| {
                if let Some(entries) = item.get("entries").and_then(|v| v.as_array()) {
                    entries.iter().any(|e| {
                        if let Some(s) = e.as_str() {
                            s.starts_with("{#itemEntry")
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            })
            .count();

        if items_with_resolved_entries > 0 || items_with_unresolved_entries > 0 {
            println!(
                "  📝 Item entry resolution for {}: {} resolved, {} unresolved",
                source, items_with_resolved_entries, items_with_unresolved_entries
            );
        }

        let result = json!({ "item": resolved_items });
        content.add_json(&format!("items/{}.json", source.to_lowercase()), &result)?;
    }

    // Also check for item fluff
    let fluff_file = data_dir.join("fluff-items.json");
    if fluff_file.exists() {
        let data = parser::load_json_file(&fluff_file)?;
        let filtered = data.filter_by_source(source);
        if !filtered.is_empty() {
            let result = json!({ "itemFluff": filtered });
            content.add_json(
                &format!("items/fluff-{}.json", source.to_lowercase()),
                &result,
            )?;
        }
    }

    Ok(())
}

/// Collect filtered races
fn collect_filtered_races(content: &mut BookContent, data_dir: &Path, source: &str) -> Result<()> {
    let races_file = data_dir.join("races.json");
    if !races_file.exists() {
        return Ok(());
    }

    let data = parser::load_json_file(&races_file)?;
    let races = parser::filter_by_source(&data, source, "race");
    let subraces = parser::filter_by_source(&data, source, "subrace");

    // Build a map of base races for parent lookups
    let mut race_map: std::collections::HashMap<String, &serde_json::Value> =
        std::collections::HashMap::new();
    for race in &races {
        if let Some(name) = race.get("name").and_then(|n| n.as_str()) {
            race_map.insert(name.to_string(), race);
        }
    }

    // Merge parent race abilities into subraces
    let merged_subraces: Vec<serde_json::Value> = subraces
        .iter()
        .map(|subrace| {
            let mut merged = subrace.clone();

            // Get parent race name
            if let Some(race_name) = subrace.get("raceName").and_then(|n| n.as_str()) {
                if let Some(parent_race) = race_map.get(race_name) {
                    // Merge abilities from parent race
                    if let Some(parent_abilities) = parent_race.get("ability") {
                        let mut merged_abilities =
                            parent_abilities.as_array().cloned().unwrap_or_default();

                        // Add subrace abilities
                        if let Some(subrace_abilities) =
                            subrace.get("ability").and_then(|a| a.as_array())
                        {
                            merged_abilities.extend(subrace_abilities.clone());
                        }

                        merged["ability"] = serde_json::Value::Array(merged_abilities);
                    }

                    // Inherit size if not specified
                    if merged.get("size").is_none() {
                        if let Some(size) = parent_race.get("size") {
                            merged["size"] = size.clone();
                        }
                    }

                    // Inherit speed if not specified
                    if merged.get("speed").is_none() {
                        if let Some(speed) = parent_race.get("speed") {
                            merged["speed"] = speed.clone();
                        }
                    }

                    // Inherit darkvision if not specified
                    if merged.get("darkvision").is_none() {
                        if let Some(darkvision) = parent_race.get("darkvision") {
                            merged["darkvision"] = darkvision.clone();
                        }
                    }

                    // Inherit languageProficiencies if not specified
                    if merged.get("languageProficiencies").is_none() {
                        if let Some(langs) = parent_race.get("languageProficiencies") {
                            merged["languageProficiencies"] = langs.clone();
                        }
                    }

                    // Inherit resist (damage resistances) if not specified
                    if merged.get("resist").is_none() {
                        if let Some(resist) = parent_race.get("resist") {
                            merged["resist"] = resist.clone();
                        }
                    }

                    // Inherit toolProficiencies if not specified
                    if merged.get("toolProficiencies").is_none() {
                        if let Some(tools) = parent_race.get("toolProficiencies") {
                            merged["toolProficiencies"] = tools.clone();
                        }
                    }

                    // Inherit weaponProficiencies if not specified
                    if merged.get("weaponProficiencies").is_none() {
                        if let Some(weapons) = parent_race.get("weaponProficiencies") {
                            merged["weaponProficiencies"] = weapons.clone();
                        }
                    }

                    // Inherit armorProficiencies if not specified
                    if merged.get("armorProficiencies").is_none() {
                        if let Some(armor) = parent_race.get("armorProficiencies") {
                            merged["armorProficiencies"] = armor.clone();
                        }
                    }
                }
            }

            merged
        })
        .collect();

    if !races.is_empty() || !merged_subraces.is_empty() {
        let result = json!({
            "race": races,
            "subrace": merged_subraces
        });
        content.add_json(&format!("races/{}.json", source.to_lowercase()), &result)?;
    }

    // Also check for race fluff
    let fluff_file = data_dir.join("fluff-races.json");
    if fluff_file.exists() {
        let data = parser::load_json_file(&fluff_file)?;
        let filtered = data.filter_by_source(source);
        if !filtered.is_empty() {
            let result = json!({ "raceFluff": filtered });
            content.add_json(
                &format!("races/fluff-{}.json", source.to_lowercase()),
                &result,
            )?;
        }
    }

    Ok(())
}

/// Collect filtered backgrounds
fn collect_filtered_backgrounds(
    content: &mut BookContent,
    data_dir: &Path,
    source: &str,
) -> Result<()> {
    let bg_file = data_dir.join("backgrounds.json");
    if !bg_file.exists() {
        return Ok(());
    }

    let data = parser::load_json_file(&bg_file)?;
    let filtered = parser::filter_by_source(&data, source, "background");

    if !filtered.is_empty() {
        let result = json!({ "background": filtered });
        content.add_json(
            &format!("backgrounds/{}.json", source.to_lowercase()),
            &result,
        )?;
    }

    // Also check for background fluff
    let fluff_file = data_dir.join("fluff-backgrounds.json");
    if fluff_file.exists() {
        let data = parser::load_json_file(&fluff_file)?;
        let filtered = data.filter_by_source(source);
        if !filtered.is_empty() {
            let result = json!({ "backgroundFluff": filtered });
            content.add_json(
                &format!("backgrounds/fluff-{}.json", source.to_lowercase()),
                &result,
            )?;
        }
    }

    Ok(())
}

/// Collect filtered feats
fn collect_filtered_feats(content: &mut BookContent, data_dir: &Path, source: &str) -> Result<()> {
    let feats_file = data_dir.join("feats.json");
    if !feats_file.exists() {
        return Ok(());
    }

    let data = parser::load_json_file(&feats_file)?;
    let filtered = parser::filter_by_source(&data, source, "feat");

    if !filtered.is_empty() {
        let result = json!({ "feat": filtered });
        content.add_json(&format!("feats/{}.json", source.to_lowercase()), &result)?;
    }

    Ok(())
}

/// Collect filtered optional features
fn collect_filtered_optfeatures(
    content: &mut BookContent,
    data_dir: &Path,
    source: &str,
) -> Result<()> {
    let opt_file = data_dir.join("optionalfeatures.json");
    if !opt_file.exists() {
        return Ok(());
    }

    let data = parser::load_json_file(&opt_file)?;
    let filtered = parser::filter_by_source(&data, source, "optionalfeature");

    if !filtered.is_empty() {
        let result = json!({ "optionalfeature": filtered });
        content.add_json(
            &format!("optionalfeatures/{}.json", source.to_lowercase()),
            &result,
        )?;
    }

    Ok(())
}

/// Generic function to collect and filter content from any JSON file
fn collect_filtered_generic(
    content: &mut BookContent,
    data_dir: &Path,
    source: &str,
    filename: &str,
    json_key: &str,
    output_dir: &str,
) -> Result<()> {
    let file_path = data_dir.join(filename);
    if !file_path.exists() {
        return Ok(());
    }

    let data = parser::load_json_file(&file_path)?;
    let filtered = parser::filter_by_source(&data, source, json_key);

    if !filtered.is_empty() {
        let result = json!({ json_key: filtered });
        content.add_json(
            &format!("{}/{}.json", output_dir, source.to_lowercase()),
            &result,
        )?;
    }

    // Check for corresponding fluff file
    let fluff_filename = format!("fluff-{}", filename);
    let fluff_file = data_dir.join(&fluff_filename);
    if fluff_file.exists() {
        let data = parser::load_json_file(&fluff_file)?;
        let fluff_key = format!("{}Fluff", json_key);
        let filtered = parser::filter_by_source(&data, source, &fluff_key);
        if !filtered.is_empty() {
            let result = json!({ fluff_key: filtered });
            content.add_json(
                &format!("{}/fluff-{}.json", output_dir, source.to_lowercase()),
                &result,
            )?;
        }
    }

    Ok(())
}

/// Collect images related to the book
fn collect_images(content: &mut BookContent, repo_path: &Path, source: &str) -> Result<()> {
    let img_dir = repo_path.join("img");
    if !img_dir.exists() {
        return Ok(());
    }

    // Collect cover image if specified
    if let Some(cover) = &content.book.cover {
        if cover.cover_type == "internal" {
            let cover_path = img_dir.join(&cover.path);
            if cover_path.exists() {
                if let Ok(data) = fs::read(&cover_path) {
                    content.add_file(&format!("img/{}", cover.path), data);
                }
            }
        }
    }

    // Collect book-specific images directory
    let book_img_dir = img_dir.join("book").join(source);
    if book_img_dir.exists() {
        collect_directory_recursive(content, &book_img_dir, &format!("img/book/{}", source))?;
    }

    // Collect adventure images
    let adventure_img_dir = img_dir.join("adventure").join(source);
    if adventure_img_dir.exists() {
        collect_directory_recursive(
            content,
            &adventure_img_dir,
            &format!("img/adventure/{}", source),
        )?;
    }

    // Collect bestiary images
    let bestiary_img_dir = img_dir.join("bestiary").join(source);
    if bestiary_img_dir.exists() {
        collect_directory_recursive(
            content,
            &bestiary_img_dir,
            &format!("img/bestiary/{}", source),
        )?;
    }

    // Collect bestiary token images
    let bestiary_tokens_dir = img_dir.join("bestiary").join("tokens").join(source);
    if bestiary_tokens_dir.exists() {
        collect_directory_recursive(
            content,
            &bestiary_tokens_dir,
            &format!("img/bestiary/tokens/{}", source),
        )?;
    }

    // Collect item images
    let items_img_dir = img_dir.join("items").join(source);
    if items_img_dir.exists() {
        collect_directory_recursive(content, &items_img_dir, &format!("img/items/{}", source))?;
    }

    // Collect backgrounds images
    let backgrounds_img_dir = img_dir.join("backgrounds").join(source);
    if backgrounds_img_dir.exists() {
        collect_directory_recursive(
            content,
            &backgrounds_img_dir,
            &format!("img/backgrounds/{}", source),
        )?;
    }

    // Collect races images
    let races_img_dir = img_dir.join("races").join(source);
    if races_img_dir.exists() {
        collect_directory_recursive(content, &races_img_dir, &format!("img/races/{}", source))?;
    }

    // Collect classes images
    let classes_img_dir = img_dir.join("classes").join(source);
    if classes_img_dir.exists() {
        collect_directory_recursive(
            content,
            &classes_img_dir,
            &format!("img/classes/{}", source),
        )?;
    }

    Ok(())
}

/// Recursively collect all files from a directory
fn collect_directory_recursive(
    content: &mut BookContent,
    dir: &Path,
    base_path: &str,
) -> Result<()> {
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let relative = entry.path().strip_prefix(dir).unwrap_or(entry.path());
            let target_path = format!("{}/{}", base_path, relative.to_string_lossy());

            if let Ok(data) = fs::read(entry.path()) {
                content.add_file(&target_path, data);
            }
        }
    }
    Ok(())
}

/// Resolve {#itemEntry} references in items
fn resolve_item_entry_references(
    items: Vec<Value>,
    item_entry_templates: &HashMap<String, Value>,
) -> Vec<Value> {
    // First, build a map of item names to their entries
    let mut item_entries_map: HashMap<String, Value> = HashMap::new();

    // Collect entries from all items and item groups
    for item in &items {
        if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
            if let Some(entries) = item.get("entries") {
                item_entries_map.insert(name.to_string(), entries.clone());
            }
        }

        // Also check for items within item groups
        if item
            .get("_isItemGroup")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            if let Some(group_items) = item.get("items").and_then(|v| v.as_array()) {
                // Item groups often have entries that apply to all items in the group
                if let Some(group_entries) = item.get("entries") {
                    // Store entries for each item name in the group
                    for group_item_name in group_items {
                        if let Some(item_name) = group_item_name.as_str() {
                            // Extract base name without damage type suffix
                            let base_name = if item_name.contains(" (") {
                                item_name.split(" (").next().unwrap_or(item_name)
                            } else {
                                item_name
                            };

                            // Store both full name and base name
                            item_entries_map.insert(item_name.to_string(), group_entries.clone());
                            if base_name != item_name {
                                item_entries_map
                                    .insert(base_name.to_string(), group_entries.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Special handling for known base items that may not have explicit entries
    // These are commonly referenced items that have standard descriptions
    item_entries_map.insert(
        "Armor of Resistance".to_string(),
        json!(["You have resistance to one type of damage while you wear this armor."]),
    );
    item_entries_map.insert(
        "Potion of Resistance".to_string(),
        json!([
            "When you drink this potion, you gain resistance to one type of damage for 1 hour."
        ]),
    );

    // Now process each item and resolve references
    items
        .into_iter()
        .map(|mut item| {
            // Clone item properties we might need for template expansion
            let item_name = item
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let item_resist = item.get("resist").cloned();
            let item_detail1 = item
                .get("detail1")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if let Some(entries) = item.get_mut("entries").and_then(|v| v.as_array_mut()) {
                let mut resolved_entries = Vec::new();

                for entry in entries.iter() {
                    if let Some(entry_str) = entry.as_str() {
                        // Check if this is an item entry reference
                        if entry_str.starts_with("{#itemEntry ") && entry_str.ends_with("}") {
                            // Extract the referenced item name and source
                            let ref_content = entry_str
                                .trim_start_matches("{#itemEntry ")
                                .trim_end_matches("}");

                            let parts: Vec<&str> = ref_content.split('|').collect();
                            let ref_name = parts.first().unwrap_or(&"");
                            let ref_source = parts.get(1).unwrap_or(&"DMG");

                            // Look up the template
                            let template_key = format!("{}|{}", ref_name, ref_source);
                            if let Some(template) = item_entry_templates.get(&template_key) {
                                // Get the template entries
                                if let Some(template_entries) =
                                    template.get("entriesTemplate").and_then(|v| v.as_array())
                                {
                                    // Process each template entry
                                    for template_entry in template_entries {
                                        if let Some(template_str) = template_entry.as_str() {
                                            // Replace template variables with item properties
                                            let mut processed = template_str.to_string();

                                            // Replace {{item.resist}} with the item's resist value
                                            let resist_value = if let Some(resist) = &item_resist {
                                                if let Some(resist_str) = resist.as_str() {
                                                    resist_str.to_string()
                                                } else if let Some(resist_array) = resist.as_array()
                                                {
                                                    resist_array
                                                        .first()
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string()
                                                } else {
                                                    String::new()
                                                }
                                            } else {
                                                // Try to extract damage type from item name
                                                // e.g., "Armor of Acid Resistance" -> "acid"
                                                // e.g., "Potion of Fire Resistance" -> "fire"
                                                if let Some(name) = &item_name {
                                                    if name.contains(" of ")
                                                        && name.contains(" Resistance")
                                                    {
                                                        let parts: Vec<&str> =
                                                            name.split(" of ").collect();
                                                        if parts.len() > 1 {
                                                            let damage_part =
                                                                parts[1].replace(" Resistance", "");
                                                            damage_part.to_lowercase()
                                                        } else {
                                                            String::new()
                                                        }
                                                    } else {
                                                        String::new()
                                                    }
                                                } else {
                                                    String::new()
                                                }
                                            };

                                            if !resist_value.is_empty() {
                                                processed = processed
                                                    .replace("{{item.resist}}", &resist_value);
                                            }

                                            // Replace {{item.detail1}} with the item's detail1 value
                                            if let Some(detail1) = &item_detail1 {
                                                processed =
                                                    processed.replace("{{item.detail1}}", detail1);
                                            }

                                            resolved_entries.push(json!(processed));
                                        } else {
                                            // Non-string template entries (like sub-entries objects)
                                            resolved_entries.push(template_entry.clone());
                                        }
                                    }
                                } else {
                                    // No template found, check our fallback map
                                    if let Some(referenced_entries) =
                                        item_entries_map.get(*ref_name)
                                    {
                                        if let Some(ref_array) = referenced_entries.as_array() {
                                            resolved_entries.extend(ref_array.clone());
                                        } else {
                                            resolved_entries.push(referenced_entries.clone());
                                        }
                                    } else {
                                        // Can't resolve, keep as-is
                                        resolved_entries.push(entry.clone());
                                    }
                                }
                            } else {
                                // No template found, check our fallback map
                                if let Some(referenced_entries) = item_entries_map.get(*ref_name) {
                                    if let Some(ref_array) = referenced_entries.as_array() {
                                        resolved_entries.extend(ref_array.clone());
                                    } else {
                                        resolved_entries.push(referenced_entries.clone());
                                    }
                                } else {
                                    // Can't resolve, keep as-is
                                    resolved_entries.push(entry.clone());
                                }
                            }
                        } else {
                            resolved_entries.push(entry.clone());
                        }
                    } else {
                        resolved_entries.push(entry.clone());
                    }
                }

                *entries = resolved_entries;
            }

            item
        })
        .collect()
}
