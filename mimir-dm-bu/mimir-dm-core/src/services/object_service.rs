//! Object catalog service.
//!
//! Provides database-backed object search, retrieval, and import functionality.
//! Supports filtering by name, object type, size, and source.

use crate::error::Result;
use crate::models::catalog::{
    CatalogObject, NewCatalogObject, ObjectData, ObjectFilters, ObjectSummary,
};
use crate::schema::catalog_objects;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info};

/// Service for searching and managing objects in the catalog.
pub struct ObjectService;

impl ObjectService {
    /// Searches objects with the given filters.
    pub fn search_objects(
        conn: &mut SqliteConnection,
        filters: ObjectFilters,
    ) -> Result<Vec<ObjectSummary>> {
        debug!("Searching objects with filters: {:?}", filters);

        let mut query = catalog_objects::table.into_boxed();

        // Apply search pattern filter
        if let Some(search_pattern) = filters.search_pattern.clone() {
            if !search_pattern.is_empty() {
                let pattern = format!("%{}%", search_pattern.to_lowercase());
                query = query.filter(
                    catalog_objects::name
                        .like(pattern.clone())
                        .or(catalog_objects::object_type.like(pattern.clone()))
                        .or(catalog_objects::size.like(pattern)),
                );
            }
        }

        // Apply source filter
        if let Some(sources) = filters.sources {
            if !sources.is_empty() {
                query = query.filter(catalog_objects::source.eq_any(sources));
            }
        }

        // Apply object type filter
        if let Some(object_types) = filters.object_types {
            if !object_types.is_empty() {
                query = query.filter(catalog_objects::object_type.eq_any(object_types));
            }
        }

        // Apply size filter
        if let Some(sizes) = filters.sizes {
            if !sizes.is_empty() {
                query = query.filter(catalog_objects::size.eq_any(sizes));
            }
        }

        let objects = query
            .order_by(catalog_objects::name.asc())
            .select(CatalogObject::as_select())
            .load::<CatalogObject>(conn)?;

        debug!("Found {} objects", objects.len());

        let summaries: Vec<ObjectSummary> = objects.iter().map(ObjectSummary::from).collect();

        Ok(summaries)
    }

    /// Gets object details by name and source.
    pub fn get_object_details(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<String>> {
        debug!("Getting object details for: {} ({})", name, source);

        let result = catalog_objects::table
            .filter(catalog_objects::name.eq(name))
            .filter(catalog_objects::source.eq(source))
            .select(catalog_objects::full_object_json)
            .first::<String>(conn)
            .optional()?;

        Ok(result)
    }

    /// Gets all unique source books containing objects.
    pub fn get_object_sources(conn: &mut SqliteConnection) -> Result<Vec<String>> {
        debug!("Getting distinct object sources");

        let sources = catalog_objects::table
            .select(catalog_objects::source)
            .distinct()
            .order_by(catalog_objects::source.asc())
            .load::<String>(conn)?;

        debug!("Found {} object sources", sources.len());
        Ok(sources)
    }

    /// Gets the total count of objects in the catalog.
    pub fn get_object_count(conn: &mut SqliteConnection) -> Result<i64> {
        debug!("Getting total object count");

        let count = catalog_objects::table.count().get_result(conn)?;

        debug!("Total objects: {}", count);
        Ok(count)
    }

    /// Gets all unique object types.
    pub fn get_object_types(conn: &mut SqliteConnection) -> Result<Vec<String>> {
        debug!("Getting distinct object types");

        let types: Vec<String> = catalog_objects::table
            .select(catalog_objects::object_type)
            .distinct()
            .filter(catalog_objects::object_type.is_not_null())
            .order_by(catalog_objects::object_type.asc())
            .load::<Option<String>>(conn)?
            .into_iter()
            .flatten()
            .collect();

        debug!("Found {} distinct object types", types.len());
        Ok(types)
    }

    /// Gets all unique object sizes.
    pub fn get_object_sizes(conn: &mut SqliteConnection) -> Result<Vec<String>> {
        debug!("Getting distinct object sizes");

        let sizes: Vec<String> = catalog_objects::table
            .select(catalog_objects::size)
            .distinct()
            .filter(catalog_objects::size.is_not_null())
            .order_by(catalog_objects::size.asc())
            .load::<Option<String>>(conn)?
            .into_iter()
            .flatten()
            .collect();

        debug!("Found {} distinct object sizes", sizes.len());
        Ok(sizes)
    }

    /// Import all object data from an uploaded book directory
    pub fn import_objects_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        info!(
            "Importing objects from book directory: {:?} (source: {})",
            book_dir, source
        );
        let mut imported_count = 0;

        let objects_dir = book_dir.join("objects");
        if !objects_dir.exists() || !objects_dir.is_dir() {
            debug!("No objects directory found in book: {:?}", book_dir);
            return Ok(0);
        }

        info!("Found objects directory: {:?}", objects_dir);

        // Read all JSON files in the objects directory
        let entries = fs::read_dir(&objects_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            debug!("Processing object file: {}", filename);

            let content = fs::read_to_string(&path)?;

            let object_data: ObjectData = serde_json::from_str(&content)?;

            // Import objects
            if let Some(objects) = object_data.object {
                for obj in &objects {
                    let new_object = NewCatalogObject::from(obj);

                    match diesel::insert_into(catalog_objects::table)
                        .values(&new_object)
                        .on_conflict((catalog_objects::name, catalog_objects::source))
                        .do_update()
                        .set((
                            catalog_objects::object_type.eq(&new_object.object_type),
                            catalog_objects::size.eq(&new_object.size),
                            catalog_objects::ac.eq(&new_object.ac),
                            catalog_objects::hp.eq(&new_object.hp),
                            catalog_objects::full_object_json.eq(&new_object.full_object_json),
                        ))
                        .execute(conn)
                    {
                        Ok(_) => {
                            imported_count += 1;
                            debug!("Imported object: {} ({})", obj.name, source);
                        }
                        Err(e) => {
                            error!("Failed to insert object {}: {}", obj.name, e);
                        }
                    }
                }
            }
        }

        info!(
            "Successfully imported {} objects from source: {}",
            imported_count, source
        );
        Ok(imported_count)
    }

    /// Remove all objects from a specific source
    pub fn remove_objects_by_source(conn: &mut SqliteConnection, source: &str) -> Result<usize> {
        info!("Removing objects from source: {}", source);

        let deleted = diesel::delete(catalog_objects::table)
            .filter(catalog_objects::source.eq(source))
            .execute(conn)?;

        info!("Removed {} objects from source: {}", deleted, source);
        Ok(deleted)
    }
}
