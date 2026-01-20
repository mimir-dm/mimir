//! Class and subclass catalog service.
//!
//! Provides database-backed class and subclass search, retrieval, and import functionality.
//! Handles the complex class hierarchy including base classes, subclasses, and class features.

use crate::error::Result;
use crate::models::catalog::class::{
    CatalogClass, CatalogSubclass, Class, ClassData, ClassFeatureData, ClassFilters, ClassFluff,
    ClassFluffData, ClassSummary, NewCatalogClass, NewCatalogClassFeature, NewCatalogSubclass,
    NewCatalogSubclassFeature, Subclass, SubclassFluff,
};
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Service for searching and managing character classes and subclasses.
pub struct ClassService<'a> {
    /// Database connection reference.
    pub conn: &'a mut SqliteConnection,
}

impl<'a> ClassService<'a> {
    /// Create a new class service.
    ///
    /// # Arguments
    /// * `conn` - Mutable reference to the SQLite connection
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Search classes and subclasses with filters.
    ///
    /// Returns both base classes and subclasses in a unified format,
    /// sorted by class name then subclass name.
    ///
    /// # Arguments
    /// * `filters` - Search criteria including name, sources, spellcasting, and abilities
    ///
    /// # Returns
    /// * `Ok(Vec<ClassSummary>)` - List of matching class/subclass summaries
    /// * `Err(DbError)` - If the database query fails
    pub fn search_classes(&mut self, filters: ClassFilters) -> Result<Vec<ClassSummary>> {
        let mut results = Vec::new();

        // First, get base classes
        let base_classes = self.search_base_classes(&filters)?;
        results.extend(base_classes);

        // Then, get subclasses
        let subclass_rows = self.search_subclass_rows(&filters)?;
        results.extend(subclass_rows);

        // Sort by class name first, then by subclass name (None values first)
        results.sort_by(|a, b| {
            match a.name.cmp(&b.name) {
                std::cmp::Ordering::Equal => {
                    // Same class name, sort by subclass_name (None first for base classes)
                    match (&a.subclass_name, &b.subclass_name) {
                        (None, None) => std::cmp::Ordering::Equal,
                        (None, Some(_)) => std::cmp::Ordering::Less,
                        (Some(_), None) => std::cmp::Ordering::Greater,
                        (Some(a_sub), Some(b_sub)) => a_sub.cmp(b_sub),
                    }
                }
                other => other,
            }
        });

        Ok(results)
    }

    /// Search base classes only
    fn search_base_classes(&mut self, filters: &ClassFilters) -> Result<Vec<ClassSummary>> {
        use crate::schema::catalog_classes::dsl::*;

        let mut query = catalog_classes.into_boxed();

        // Filter by name (partial match on class name)
        if let Some(name_filter) = &filters.name {
            if !name_filter.is_empty() {
                query = query.filter(name.like(format!("%{}%", name_filter)));
            }
        }

        // Filter by sources
        if let Some(source_list) = &filters.sources {
            if !source_list.is_empty() {
                query = query.filter(source.eq_any(source_list));
            }
        }

        // Filter by spellcasting ability
        if let Some(has_spell) = filters.has_spellcasting {
            if has_spell {
                query = query.filter(spellcasting_ability.is_not_null());
            } else {
                query = query.filter(spellcasting_ability.is_null());
            }
        }

        // Filter by primary abilities
        if let Some(abilities) = &filters.primary_abilities {
            if !abilities.is_empty() {
                query = query.filter(primary_ability.eq_any(abilities));
            }
        }

        let classes = query
            .select(CatalogClass::as_select())
            .limit(super::DEFAULT_QUERY_LIMIT)
            .load::<CatalogClass>(self.conn)?;

        Ok(classes.iter().map(ClassSummary::from).collect())
    }

    /// Search subclasses and return them as unified rows
    fn search_subclass_rows(&mut self, filters: &ClassFilters) -> Result<Vec<ClassSummary>> {
        use crate::schema::{catalog_classes, catalog_subclasses};

        // Join subclasses with their base classes
        let results = catalog_subclasses::table
            .inner_join(
                catalog_classes::table.on(catalog_subclasses::class_name
                    .eq(catalog_classes::name)
                    .and(catalog_subclasses::class_source.eq(catalog_classes::source))),
            )
            .select((CatalogSubclass::as_select(), CatalogClass::as_select()))
            .load::<(CatalogSubclass, CatalogClass)>(self.conn)?;

        let mut subclass_summaries = Vec::new();

        for (subclass, base_class) in results {
            // Apply filters
            let mut include = true;

            // Filter by name (can match either class name or subclass name)
            if let Some(name_filter) = &filters.name {
                if !name_filter.is_empty() {
                    let matches_class = base_class
                        .name
                        .to_lowercase()
                        .contains(&name_filter.to_lowercase());
                    let matches_subclass = subclass
                        .name
                        .to_lowercase()
                        .contains(&name_filter.to_lowercase());
                    if !matches_class && !matches_subclass {
                        include = false;
                    }
                }
            }

            // Filter by sources
            if let Some(source_list) = &filters.sources {
                if !source_list.is_empty() && !source_list.contains(&subclass.source) {
                    include = false;
                }
            }

            // Filter by spellcasting (check both subclass and base class)
            if let Some(has_spell) = filters.has_spellcasting {
                let has_spellcasting = subclass.spellcasting_ability.is_some()
                    || base_class.spellcasting_ability.is_some();
                if has_spell != has_spellcasting {
                    include = false;
                }
            }

            // Filter by primary abilities (use base class primary ability)
            if let Some(abilities) = &filters.primary_abilities {
                if !abilities.is_empty() {
                    if let Some(ref ability) = base_class.primary_ability {
                        if !abilities.contains(ability) {
                            include = false;
                        }
                    } else {
                        include = false;
                    }
                }
            }

            if include {
                subclass_summaries.push(ClassSummary::from_subclass(&subclass, &base_class));
            }
        }

        Ok(subclass_summaries)
    }

    /// Get a specific class by name and source.
    ///
    /// Includes fluff data (images, descriptions) if available.
    ///
    /// # Arguments
    /// * `class_name` - Exact name of the class
    /// * `class_source` - Source book code (e.g., "PHB", "XGE")
    ///
    /// # Returns
    /// * `Ok(Some(Class))` - The full class data with fluff if found
    /// * `Ok(None)` - If no matching class exists
    /// * `Err(DbError)` - If the database query fails
    pub fn get_class_by_name_and_source(
        &mut self,
        class_name: &str,
        class_source: &str,
    ) -> Result<Option<Class>> {
        use crate::schema::catalog_classes::dsl::*;

        let catalog_class = catalog_classes
            .filter(name.eq(class_name))
            .filter(source.eq(class_source))
            .select(CatalogClass::as_select())
            .first::<CatalogClass>(self.conn)
            .optional()?;

        match catalog_class {
            Some(class_record) => {
                let mut parsed_class: Class = serde_json::from_str(&class_record.full_class_json)?;

                // Add fluff data if available
                if let Some(fluff_json_str) = &class_record.fluff_json {
                    if let Ok(class_fluff) = serde_json::from_str::<ClassFluff>(fluff_json_str) {
                        parsed_class.fluff = Some(class_fluff);
                    }
                }

                Ok(Some(parsed_class))
            }
            None => Ok(None),
        }
    }

    /// Get caster progression for a class by name.
    ///
    /// Returns the first matching class's caster_progression value.
    /// Used for spell slot calculations when source is unknown.
    ///
    /// # Arguments
    /// * `class_name` - Name of the class (case-insensitive)
    ///
    /// # Returns
    /// * `Ok(Some(String))` - Caster progression ("full", "1/2", "1/3", "pact") if found
    /// * `Ok(None)` - If class not found or has no spellcasting
    pub fn get_caster_progression_by_name(
        &mut self,
        class_name_param: &str,
    ) -> Result<Option<String>> {
        use crate::schema::catalog_classes::dsl::*;

        // SQLite LIKE is case-insensitive for ASCII by default
        let result = catalog_classes
            .filter(name.like(class_name_param))
            .select(caster_progression)
            .first::<Option<String>>(self.conn)
            .optional()?;

        Ok(result.flatten())
    }

    /// Get a specific subclass by name, class name, and source.
    ///
    /// Includes fluff data and introductory description if available.
    ///
    /// # Arguments
    /// * `subclass_name` - Exact name of the subclass
    /// * `_class_name` - Name of the parent class (used in filter)
    /// * `_class_source` - Source of the parent class (used in filter)
    ///
    /// # Returns
    /// * `Ok(Some(Subclass))` - The full subclass data if found
    /// * `Ok(None)` - If no matching subclass exists
    /// * `Err(DbError)` - If the database query fails
    pub fn get_subclass_by_name(
        &mut self,
        subclass_name: &str,
        _class_name: &str,
        _class_source: &str,
    ) -> Result<Option<Subclass>> {
        use crate::schema::catalog_subclasses::dsl::*;

        let subclass_record = catalog_subclasses
            .filter(name.eq(subclass_name))
            .filter(crate::schema::catalog_subclasses::class_name.eq(class_name))
            .filter(crate::schema::catalog_subclasses::class_source.eq(class_source))
            .select(CatalogSubclass::as_select())
            .first::<CatalogSubclass>(self.conn)
            .optional()?;

        match subclass_record {
            Some(record) => {
                let mut parsed_subclass: Subclass =
                    serde_json::from_str(&record.full_subclass_json)?;

                // Add fluff data - first try subclass-specific fluff, then fall back to parent class fluff
                let mut fluff_loaded = false;
                if let Some(fluff_json_str) = &record.fluff_json {
                    if let Ok(subclass_fluff) =
                        serde_json::from_str::<SubclassFluff>(fluff_json_str)
                    {
                        parsed_subclass.fluff = Some(subclass_fluff);
                        fluff_loaded = true;
                    }
                }

                // If no subclass fluff, try to get parent class fluff
                if !fluff_loaded {
                    if let Ok(Some(parent_class)) =
                        self.get_class_by_name_and_source(&record.class_name, &record.class_source)
                    {
                        if let Some(class_fluff) = parent_class.fluff {
                            // Convert ClassFluff to SubclassFluff structure
                            let subclass_fluff = SubclassFluff {
                                entries: class_fluff.entries,
                                images: class_fluff.images,
                                name: parsed_subclass.name.clone(),
                                short_name: parsed_subclass.short_name.clone(),
                                class_name: parsed_subclass.class_name.clone(),
                                class_source: parsed_subclass.class_source.clone(),
                                source: parsed_subclass.source.clone(),
                            };
                            parsed_subclass.fluff = Some(subclass_fluff);
                        }
                    }
                }

                // Fetch introductory subclass feature description
                self.populate_subclass_intro_description(&mut parsed_subclass, &record)?;

                Ok(Some(parsed_subclass))
            }
            None => Ok(None),
        }
    }

    /// Get all subclasses for a specific class.
    ///
    /// # Arguments
    /// * `_class_name` - Name of the parent class
    /// * `_class_source` - Source of the parent class
    ///
    /// # Returns
    /// * `Ok(Vec<Subclass>)` - List of subclasses for the class
    /// * `Err(DbError)` - If the database query fails
    pub fn get_subclasses_for_class(
        &mut self,
        _class_name: &str,
        _class_source: &str,
    ) -> Result<Vec<Subclass>> {
        use crate::schema::catalog_subclasses::dsl::*;

        let subclass_records = catalog_subclasses
            .filter(crate::schema::catalog_subclasses::class_name.eq(class_name))
            .filter(crate::schema::catalog_subclasses::class_source.eq(class_source))
            .select(CatalogSubclass::as_select())
            .load::<CatalogSubclass>(self.conn)?;

        let mut result = Vec::new();
        for subclass_record in subclass_records {
            let parsed_subclass: Subclass =
                serde_json::from_str(&subclass_record.full_subclass_json)?;
            result.push(parsed_subclass);
        }

        Ok(result)
    }

    /// Get all distinct source books that contain classes.
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - Sorted list of source book codes
    /// * `Err(DbError)` - If the database query fails
    pub fn get_class_sources(&mut self) -> Result<Vec<String>> {
        use crate::schema::catalog_classes::dsl::*;

        let sources = catalog_classes
            .select(source)
            .distinct()
            .order_by(source)
            .load::<String>(self.conn)?;

        Ok(sources)
    }

    /// Get a class feature by name and class.
    ///
    /// Returns the full feature data including entries (description).
    ///
    /// # Arguments
    /// * `feature_name` - Name of the feature
    /// * `class_name` - Name of the class
    /// * `class_source` - Source book code (e.g., "PHB")
    ///
    /// # Returns
    /// * `Ok(Some(ClassFeature))` - The full feature data if found
    /// * `Ok(None)` - If no matching feature exists
    pub fn get_class_feature(
        &mut self,
        feature_name: &str,
        class_name_param: &str,
        class_source_param: &str,
    ) -> Result<Option<crate::models::catalog::class::ClassFeature>> {
        use crate::schema::catalog_class_features::dsl::*;

        let feature_record = catalog_class_features
            .filter(name.eq(feature_name))
            .filter(class_name.eq(class_name_param))
            .filter(class_source.eq(class_source_param))
            .select(full_feature_json)
            .first::<String>(self.conn)
            .optional()?;

        match feature_record {
            Some(json_str) => {
                let feature: crate::models::catalog::class::ClassFeature =
                    serde_json::from_str(&json_str)?;
                Ok(Some(feature))
            }
            None => Ok(None),
        }
    }

    /// Get a subclass feature by name and subclass.
    ///
    /// Returns the full feature data including entries (description).
    ///
    /// # Arguments
    /// * `feature_name` - Name of the feature
    /// * `class_name` - Name of the class
    /// * `subclass_short_name` - Short name of the subclass
    /// * `source` - Source book code (e.g., "PHB")
    ///
    /// # Returns
    /// * `Ok(Some(SubclassFeature))` - The full feature data if found
    /// * `Ok(None)` - If no matching feature exists
    pub fn get_subclass_feature(
        &mut self,
        feature_name: &str,
        class_name_param: &str,
        subclass_short_name_param: &str,
        source_param: &str,
    ) -> Result<Option<crate::models::catalog::class::SubclassFeature>> {
        use crate::schema::catalog_subclass_features::dsl::*;

        let feature_record = catalog_subclass_features
            .filter(name.eq(feature_name))
            .filter(class_name.eq(class_name_param))
            .filter(subclass_short_name.eq(subclass_short_name_param))
            .filter(source.eq(source_param))
            .select(full_feature_json)
            .first::<String>(self.conn)
            .optional()?;

        match feature_record {
            Some(json_str) => {
                let feature: crate::models::catalog::class::SubclassFeature =
                    serde_json::from_str(&json_str)?;
                Ok(Some(feature))
            }
            None => Ok(None),
        }
    }

    /// Get all class features up to a given level.
    ///
    /// Returns feature names and levels for building FeatureReference entries.
    ///
    /// # Arguments
    /// * `class_name_param` - Name of the class
    /// * `class_source_param` - Source book code (e.g., "PHB")
    /// * `max_level` - Maximum level to include (inclusive)
    ///
    /// # Returns
    /// * `Ok(Vec<(String, i32, String)>)` - List of (feature_name, level, source) tuples
    pub fn get_class_features_up_to_level(
        &mut self,
        class_name_param: &str,
        class_source_param: &str,
        max_level: i32,
    ) -> Result<Vec<(String, i32, String)>> {
        use crate::schema::catalog_class_features::dsl::*;

        let features = catalog_class_features
            .filter(class_name.eq(class_name_param))
            .filter(class_source.eq(class_source_param))
            .filter(level.le(max_level))
            .select((name, level, source))
            .order_by(level)
            .load::<(String, i32, String)>(self.conn)?;

        Ok(features)
    }

    /// Get all subclass features up to a given level.
    ///
    /// Returns feature names and levels for building FeatureReference entries.
    ///
    /// # Arguments
    /// * `class_name_param` - Name of the class
    /// * `subclass_short_name_param` - Short name of the subclass
    /// * `source_param` - Source book code (e.g., "PHB")
    /// * `max_level` - Maximum level to include (inclusive)
    ///
    /// # Returns
    /// * `Ok(Vec<(String, i32, String)>)` - List of (feature_name, level, source) tuples
    pub fn get_subclass_features_up_to_level(
        &mut self,
        class_name_param: &str,
        subclass_short_name_param: &str,
        source_param: &str,
        max_level: i32,
    ) -> Result<Vec<(String, i32, String)>> {
        use crate::schema::catalog_subclass_features::dsl::*;

        let features = catalog_subclass_features
            .filter(class_name.eq(class_name_param))
            .filter(subclass_short_name.eq(subclass_short_name_param))
            .filter(subclass_source.eq(source_param))
            .filter(level.le(max_level))
            .select((name, level, source))
            .order_by(level)
            .load::<(String, i32, String)>(self.conn)?;

        Ok(features)
    }

    /// Get all distinct primary abilities used by classes.
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - Sorted list of ability names (e.g., "Strength", "Dexterity")
    /// * `Err(DbError)` - If the database query fails
    pub fn get_primary_abilities(&mut self) -> Result<Vec<String>> {
        use crate::schema::catalog_classes::dsl::*;

        let abilities = catalog_classes
            .select(primary_ability)
            .distinct()
            .filter(primary_ability.is_not_null())
            .order_by(primary_ability)
            .load::<Option<String>>(self.conn)?
            .into_iter()
            .flatten()
            .collect();

        Ok(abilities)
    }

    /// Get class count grouped by source for statistics.
    ///
    /// # Returns
    /// * `Ok(Vec<(String, i64)>)` - List of (source, count) pairs
    /// * `Err(DbError)` - If the database query fails
    pub fn get_class_count_by_source(&mut self) -> Result<Vec<(String, i64)>> {
        use crate::schema::catalog_classes::dsl::*;

        let counts = catalog_classes
            .group_by(source)
            .select((source, diesel::dsl::count_star()))
            .load::<(String, i64)>(self.conn)?;

        Ok(counts)
    }

    /// Populate subclass intro description from the introductory subclass feature
    fn populate_subclass_intro_description(
        &mut self,
        subclass: &mut Subclass,
        record: &CatalogSubclass,
    ) -> Result<()> {
        use crate::schema::catalog_subclass_features::dsl::*;

        // Look for the introductory subclass feature (usually at level 3, with the same name as the subclass)
        let intro_feature = catalog_subclass_features
            .filter(name.eq(&subclass.name))
            .filter(class_name.eq(&record.class_name))
            .filter(class_source.eq(&record.class_source))
            .filter(subclass_source.eq(&record.source))
            .filter(level.le(3)) // Usually level 1, 2, or 3
            .order_by(level.asc()) // Get the earliest level if multiple matches
            .select(full_feature_json)
            .first::<String>(self.conn)
            .optional()?;

        if let Some(feature_json) = intro_feature {
            if let Ok(feature_data) = serde_json::from_str::<serde_json::Value>(&feature_json) {
                if let Some(entries) = feature_data.get("entries").and_then(|e| e.as_array()) {
                    // Extract the first few text entries as the intro description
                    let mut intro_text = Vec::new();

                    for entry in entries.iter().take(2) {
                        // Take first 2 entries
                        if let Some(text) = entry.as_str() {
                            intro_text.push(text.to_string());
                        }
                    }

                    if !intro_text.is_empty() {
                        subclass.intro_description = Some(intro_text.join(" "));
                    }
                }
            }
        }

        Ok(())
    }

    /// Load class fluff data from corresponding fluff file
    fn load_class_fluff_data(
        book_dir: &Path,
        source: &str,
    ) -> Option<std::collections::HashMap<String, ClassFluff>> {
        // Look for fluff files in class directory
        let search_paths = [
            book_dir
                .join("class")
                .join(format!("fluff-{}.json", source.to_lowercase())),
            book_dir
                .join("classes")
                .join(format!("fluff-{}.json", source.to_lowercase())),
            book_dir
                .join("class")
                .join(format!("fluff-class-{}.json", source.to_lowercase())),
            book_dir
                .join("classes")
                .join(format!("fluff-class-{}.json", source.to_lowercase())),
        ];

        for fluff_file in &search_paths {
            if !fluff_file.exists() {
                continue;
            }

            debug!("Loading class fluff data from: {:?}", fluff_file);

            match fs::read_to_string(fluff_file) {
                Ok(fluff_content) => match serde_json::from_str::<ClassFluffData>(&fluff_content) {
                    Ok(fluff_data) => {
                        let mut fluff_map = std::collections::HashMap::new();

                        if let Some(class_fluff) = fluff_data.class_fluff {
                            for fluff in class_fluff {
                                fluff_map.insert(fluff.name.to_lowercase(), fluff);
                            }
                        }

                        debug!("Loaded class fluff data for {} classes", fluff_map.len());
                        return Some(fluff_map);
                    }
                    Err(e) => {
                        warn!("Failed to parse class fluff file {:?}: {}", fluff_file, e);
                    }
                },
                Err(e) => {
                    warn!("Failed to read class fluff file {:?}: {}", fluff_file, e);
                }
            }
        }

        debug!("No class fluff data found for source: {}", source);
        None
    }

    /// Load subclass fluff data from corresponding fluff file
    fn load_subclass_fluff_data(
        book_dir: &Path,
        source: &str,
    ) -> Option<std::collections::HashMap<String, SubclassFluff>> {
        // Look for subclass fluff files in class directory
        let search_paths = [
            book_dir
                .join("class")
                .join(format!("subclass-fluff-{}.json", source.to_lowercase())),
            book_dir
                .join("classes")
                .join(format!("subclass-fluff-{}.json", source.to_lowercase())),
            book_dir
                .join("class")
                .join(format!("fluff-{}.json", source.to_lowercase())),
            book_dir
                .join("classes")
                .join(format!("fluff-{}.json", source.to_lowercase())),
        ];

        for fluff_file in &search_paths {
            if !fluff_file.exists() {
                continue;
            }

            debug!("Loading subclass fluff data from: {:?}", fluff_file);

            match fs::read_to_string(fluff_file) {
                Ok(fluff_content) => match serde_json::from_str::<ClassFluffData>(&fluff_content) {
                    Ok(fluff_data) => {
                        let mut fluff_map = std::collections::HashMap::new();

                        if let Some(subclass_fluff) = fluff_data.subclass_fluff {
                            for fluff in subclass_fluff {
                                let key = format!(
                                    "{}|{}",
                                    fluff.class_name.to_lowercase(),
                                    fluff.name.to_lowercase()
                                );
                                fluff_map.insert(key, fluff);
                            }
                        }

                        debug!(
                            "Loaded subclass fluff data for {} subclasses",
                            fluff_map.len()
                        );
                        return Some(fluff_map);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to parse subclass fluff file {:?}: {}",
                            fluff_file, e
                        );
                    }
                },
                Err(e) => {
                    warn!("Failed to read subclass fluff file {:?}: {}", fluff_file, e);
                }
            }
        }

        debug!("No subclass fluff data found for source: {}", source);
        None
    }

    /// Import classes and related data from a book directory.
    ///
    /// Imports classes, subclasses, class features, and subclass features
    /// from JSON files in the book directory. Also loads fluff data if available.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `book_dir` - Path to the book directory
    /// * `source` - Source book code to assign to imported data
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of class-related items imported
    /// * `Err(DbError)` - If reading files or database operations fail
    pub fn import_classes_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        use crate::schema::{
            catalog_class_features, catalog_classes, catalog_subclass_features, catalog_subclasses,
        };

        info!("Importing classes from book: {}", source);

        // Load fluff data for classes and subclasses
        let class_fluff_data = Self::load_class_fluff_data(book_dir, source);
        let subclass_fluff_data = Self::load_subclass_fluff_data(book_dir, source);

        let mut total_imported = 0;

        // Search for class files in multiple possible locations
        let search_dirs = [
            book_dir.join("class"),
            book_dir.join("classes"),
            book_dir.join("data"),
            book_dir.to_path_buf(),
        ];

        for search_dir in &search_dirs {
            if !search_dir.exists() {
                continue;
            }

            debug!("Searching for class files in: {:?}", search_dir);

            let entries = fs::read_dir(search_dir)?;

            for entry in entries.flatten() {
                let path = entry.path();

                // Skip if not a JSON file
                if path.extension().is_none_or(|ext| ext != "json") {
                    continue;
                }

                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();

                // Check if this might be a class file based on naming patterns
                let is_main_class_file = search_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == "class" || n == "classes")
                    .unwrap_or(false)
                    && !filename_str.contains("fluff")
                    && !filename_str.contains("feature");

                let is_class_named_file = filename_str.contains("class")
                    && !filename_str.contains("fluff")
                    && !filename_str.contains("feature")
                    && !filename_str.contains("subclass-feature");

                let is_main_book_file = filename_str == format!("{}.json", source.to_lowercase());

                if is_main_class_file || is_class_named_file || is_main_book_file {
                    debug!("Processing class file: {:?}", path);

                    let content = fs::read_to_string(&path)?;

                    // Try to parse as ClassData structure first
                    if let Ok(class_data) = serde_json::from_str::<ClassData>(&content) {
                        // Import classes
                        if !class_data.classes.is_empty() {
                            for class in &class_data.classes {
                                let mut new_class = NewCatalogClass::from(class);
                                new_class.source = source.to_string();

                                // Update source in JSON
                                if let Ok(mut class_json) = serde_json::from_str::<serde_json::Value>(
                                    &new_class.full_class_json,
                                ) {
                                    if let Some(obj) = class_json.as_object_mut() {
                                        obj.insert(
                                            "source".to_string(),
                                            serde_json::Value::String(source.to_string()),
                                        );
                                        if let Ok(updated_json) = serde_json::to_string(&class_json)
                                        {
                                            new_class.full_class_json = updated_json;
                                        }
                                    }
                                }

                                // Add fluff data if available
                                if let Some(ref class_fluff_map) = class_fluff_data {
                                    let class_name_lower = class.name.to_lowercase();
                                    if let Some(class_fluff) =
                                        class_fluff_map.get(&class_name_lower)
                                    {
                                        if let Ok(fluff_json) = serde_json::to_string(class_fluff) {
                                            new_class.fluff_json = Some(fluff_json);
                                        }
                                    }
                                }

                                diesel::insert_into(catalog_classes::table)
                                    .values(&new_class)
                                    .on_conflict((catalog_classes::name, catalog_classes::source))
                                    .do_nothing()
                                    .execute(conn)?;

                                total_imported += 1;
                                debug!("Imported class: {} ({})", class.name, source);
                            }
                        }

                        // Import subclasses
                        if let Some(subclasses) = &class_data.subclass {
                            for subclass in subclasses {
                                let mut new_subclass = NewCatalogSubclass::from(subclass);
                                new_subclass.source = source.to_string();

                                // Update source in JSON
                                if let Ok(mut subclass_json) =
                                    serde_json::from_str::<serde_json::Value>(
                                        &new_subclass.full_subclass_json,
                                    )
                                {
                                    if let Some(obj) = subclass_json.as_object_mut() {
                                        obj.insert(
                                            "source".to_string(),
                                            serde_json::Value::String(source.to_string()),
                                        );
                                        if let Ok(updated_json) =
                                            serde_json::to_string(&subclass_json)
                                        {
                                            new_subclass.full_subclass_json = updated_json;
                                        }
                                    }
                                }

                                // Add fluff data if available
                                if let Some(ref subclass_fluff_map) = subclass_fluff_data {
                                    let subclass_key = format!(
                                        "{}|{}",
                                        subclass.class_name.to_lowercase(),
                                        subclass.name.to_lowercase()
                                    );
                                    if let Some(subclass_fluff) =
                                        subclass_fluff_map.get(&subclass_key)
                                    {
                                        if let Ok(fluff_json) =
                                            serde_json::to_string(subclass_fluff)
                                        {
                                            new_subclass.fluff_json = Some(fluff_json);
                                        }
                                    }
                                }

                                diesel::insert_into(catalog_subclasses::table)
                                    .values(&new_subclass)
                                    .on_conflict((
                                        catalog_subclasses::name,
                                        catalog_subclasses::class_name,
                                        catalog_subclasses::source,
                                    ))
                                    .do_nothing()
                                    .execute(conn)?;

                                debug!("Imported subclass: {} ({})", subclass.name, source);
                            }
                        }

                        // Import class features
                        if let Some(features) = &class_data.class_features {
                            for feature in features {
                                let mut new_feature = NewCatalogClassFeature::from(feature);
                                new_feature.source = source.to_string();

                                // Update source in JSON
                                if let Ok(mut feature_json) =
                                    serde_json::from_str::<serde_json::Value>(
                                        &new_feature.full_feature_json,
                                    )
                                {
                                    if let Some(obj) = feature_json.as_object_mut() {
                                        obj.insert(
                                            "source".to_string(),
                                            serde_json::Value::String(source.to_string()),
                                        );
                                        if let Ok(updated_json) =
                                            serde_json::to_string(&feature_json)
                                        {
                                            new_feature.full_feature_json = updated_json;
                                        }
                                    }
                                }

                                diesel::insert_into(catalog_class_features::table)
                                    .values(&new_feature)
                                    .on_conflict_do_nothing()
                                    .execute(conn)?;

                                debug!("Imported class feature: {} ({})", feature.name, source);
                            }
                        }

                        // Import subclass features
                        if let Some(subclass_features) = &class_data.subclass_features {
                            for feature in subclass_features {
                                let mut new_feature = NewCatalogSubclassFeature::from(feature);
                                new_feature.source = source.to_string();

                                // Update source in JSON
                                if let Ok(mut feature_json) =
                                    serde_json::from_str::<serde_json::Value>(
                                        &new_feature.full_feature_json,
                                    )
                                {
                                    if let Some(obj) = feature_json.as_object_mut() {
                                        obj.insert(
                                            "source".to_string(),
                                            serde_json::Value::String(source.to_string()),
                                        );
                                        if let Ok(updated_json) =
                                            serde_json::to_string(&feature_json)
                                        {
                                            new_feature.full_feature_json = updated_json;
                                        }
                                    }
                                }

                                diesel::insert_into(catalog_subclass_features::table)
                                    .values(&new_feature)
                                    .on_conflict_do_nothing()
                                    .execute(conn)?;

                                debug!("Imported subclass feature: {} ({})", feature.name, source);
                            }
                        }
                    } else if let Ok(classes) = serde_json::from_str::<Vec<Class>>(&content) {
                        // Handle direct array of classes
                        for class in &classes {
                            let mut new_class = NewCatalogClass::from(class);
                            new_class.source = source.to_string();

                            // Update source in JSON
                            if let Ok(mut class_json) = serde_json::from_str::<serde_json::Value>(
                                &new_class.full_class_json,
                            ) {
                                if let Some(obj) = class_json.as_object_mut() {
                                    obj.insert(
                                        "source".to_string(),
                                        serde_json::Value::String(source.to_string()),
                                    );
                                    if let Ok(updated_json) = serde_json::to_string(&class_json) {
                                        new_class.full_class_json = updated_json;
                                    }
                                }
                            }

                            diesel::insert_into(catalog_classes::table)
                                .values(&new_class)
                                .on_conflict((catalog_classes::name, catalog_classes::source))
                                .do_nothing()
                                .execute(conn)?;

                            total_imported += 1;
                            debug!("Imported class: {} ({})", class.name, source);
                        }
                    }
                }

                // Check for separate feature files
                let is_feature_file = filename_str.starts_with("features-")
                    || filename_str.starts_with("class-features-")
                    || (filename_str.contains("feature")
                        && !filename_str.contains("fluff")
                        && !filename_str.starts_with("subclass-features"));

                let is_subclass_feature_file = filename_str.starts_with("subclass-features-");

                if is_feature_file {
                    debug!("Processing class feature file: {:?}", path);

                    let content = fs::read_to_string(&path)?;

                    if let Ok(feature_data) = serde_json::from_str::<ClassFeatureData>(&content) {
                        if let Some(features) = &feature_data.class_feature {
                            for feature in features {
                                let mut new_feature = NewCatalogClassFeature::from(feature);
                                new_feature.source = source.to_string();

                                // Update source in JSON
                                if let Ok(mut feature_json) =
                                    serde_json::from_str::<serde_json::Value>(
                                        &new_feature.full_feature_json,
                                    )
                                {
                                    if let Some(obj) = feature_json.as_object_mut() {
                                        obj.insert(
                                            "source".to_string(),
                                            serde_json::Value::String(source.to_string()),
                                        );
                                        if let Ok(updated_json) =
                                            serde_json::to_string(&feature_json)
                                        {
                                            new_feature.full_feature_json = updated_json;
                                        }
                                    }
                                }

                                diesel::insert_into(catalog_class_features::table)
                                    .values(&new_feature)
                                    .on_conflict_do_nothing()
                                    .execute(conn)?;

                                debug!("Imported class feature: {} ({})", feature.name, source);
                            }
                        }
                    }
                }

                if is_subclass_feature_file {
                    debug!("Processing subclass feature file: {:?}", path);

                    let content = fs::read_to_string(&path)?;

                    if let Ok(feature_data) = serde_json::from_str::<ClassFeatureData>(&content) {
                        if let Some(subclass_features) = &feature_data.subclass_feature {
                            for feature in subclass_features {
                                let mut new_feature = NewCatalogSubclassFeature::from(feature);
                                new_feature.source = source.to_string();

                                // Update source in JSON
                                if let Ok(mut feature_json) =
                                    serde_json::from_str::<serde_json::Value>(
                                        &new_feature.full_feature_json,
                                    )
                                {
                                    if let Some(obj) = feature_json.as_object_mut() {
                                        obj.insert(
                                            "source".to_string(),
                                            serde_json::Value::String(source.to_string()),
                                        );
                                        if let Ok(updated_json) =
                                            serde_json::to_string(&feature_json)
                                        {
                                            new_feature.full_feature_json = updated_json;
                                        }
                                    }
                                }

                                diesel::insert_into(catalog_subclass_features::table)
                                    .values(&new_feature)
                                    .on_conflict_do_nothing()
                                    .execute(conn)?;

                                debug!("Imported subclass feature: {} ({})", feature.name, source);
                            }
                        }
                    }
                }
            }
        }

        info!(
            "Successfully imported {} total class-related items from source: {}",
            total_imported, source
        );
        Ok(total_imported)
    }

    /// Remove all classes and related data from a specific source.
    ///
    /// Deletes subclass features, class features, subclasses, and classes
    /// in the correct dependency order.
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `source` - Source book code to remove data from
    ///
    /// # Returns
    /// * `Ok(usize)` - Total number of items deleted
    /// * `Err(DbError)` - If the database operations fail
    pub fn remove_classes_from_source(conn: &mut SqliteConnection, source: &str) -> Result<usize> {
        use crate::schema::{
            catalog_class_features, catalog_classes, catalog_subclass_features, catalog_subclasses,
        };

        info!("Removing classes from source: {}", source);

        // Delete in reverse dependency order
        let subclass_features_deleted = diesel::delete(
            catalog_subclass_features::table.filter(catalog_subclass_features::source.eq(source)),
        )
        .execute(conn)?;

        let class_features_deleted = diesel::delete(
            catalog_class_features::table.filter(catalog_class_features::source.eq(source)),
        )
        .execute(conn)?;

        let subclasses_deleted =
            diesel::delete(catalog_subclasses::table.filter(catalog_subclasses::source.eq(source)))
                .execute(conn)?;

        let classes_deleted =
            diesel::delete(catalog_classes::table.filter(catalog_classes::source.eq(source)))
                .execute(conn)?;

        let total_deleted = classes_deleted
            + subclasses_deleted
            + class_features_deleted
            + subclass_features_deleted;
        info!(
            "Removed {} total class-related items from source: {}",
            total_deleted, source
        );
        Ok(total_deleted)
    }
}
