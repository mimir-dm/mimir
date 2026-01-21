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
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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

/// Catalog import service for importing 5etools data.
pub struct CatalogImportService<'a> {
    conn: &'a mut SqliteConnection,
    /// Optional path to 5etools img directory.
    source_img_dir: Option<PathBuf>,
    /// Optional path to destination directory where images will be copied.
    dest_img_dir: Option<PathBuf>,
    /// Count of images copied during import.
    images_copied: usize,
}

impl<'a> CatalogImportService<'a> {
    /// Create a new import service with a database connection.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self {
            conn,
            source_img_dir: None,
            dest_img_dir: None,
            images_copied: 0,
        }
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
        let books = discover_available_sources(repo_path)
            .context("Failed to discover available sources")?;

        info!("Found {} source books to import", books.len());

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

        // Add image count to result
        result.images_copied = self.images_copied;

        Ok(result)
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
            "item" => self.import_item(entity, name, source, &data, fluff_ref),
            "class" => self.import_class(name, source, &data, fluff_ref),
            "subclass" => self.import_subclass(entity, name, source, &data, fluff_ref),
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

    fn import_item(&mut self, entity: &Value, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let item_type = entity
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
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

    fn import_class(&mut self, name: &str, source: &str, data: &str, fluff: Option<&str>) -> Result<i32> {
        let mut class = NewClass::new(name, source, data);
        class.fluff = fluff;
        catalog::insert_class(self.conn, &class).context("Failed to insert class")
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
            .get("type")
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
