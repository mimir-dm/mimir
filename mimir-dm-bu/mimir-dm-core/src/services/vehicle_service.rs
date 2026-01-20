//! Vehicle catalog service.
//!
//! Provides database-backed vehicle search, retrieval, and import functionality.
//! Supports filtering by name, type, size, terrain, and source.

use crate::error::Result;
use crate::models::catalog::vehicle::{
    CatalogVehicle, NewCatalogVehicle, Vehicle, VehicleData, VehicleFilters, VehicleSummary,
};
use crate::schema::catalog_vehicles;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info};

/// Service for searching and managing vehicles in the catalog.
pub struct VehicleService<'a> {
    /// Database connection reference.
    pub conn: &'a mut SqliteConnection,
}

impl<'a> VehicleService<'a> {
    /// Creates a new vehicle service with the given database connection.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Search vehicles with filters
    pub fn search_vehicles(&mut self, filters: VehicleFilters) -> Result<Vec<VehicleSummary>> {
        use crate::schema::catalog_vehicles::dsl::*;

        let mut query = catalog_vehicles.into_boxed();

        // Filter by name (partial match)
        if let Some(name_filter) = &filters.name {
            if !name_filter.is_empty() {
                let search_pattern = format!("%{}%", name_filter.to_lowercase());
                query = query.filter(name.like(search_pattern));
            }
        }

        // Filter by sources
        if let Some(source_filters) = &filters.sources {
            if !source_filters.is_empty() {
                query = query.filter(source.eq_any(source_filters));
            }
        }

        // Filter by vehicle types
        if let Some(type_filters) = &filters.vehicle_types {
            if !type_filters.is_empty() {
                query = query.filter(vehicle_type.eq_any(type_filters));
            }
        }

        // Filter by sizes
        if let Some(size_filters) = &filters.sizes {
            if !size_filters.is_empty() {
                query = query.filter(size.eq_any(size_filters));
            }
        }

        // Filter by terrains (partial match in comma-separated string)
        if let Some(terrain_filters) = &filters.terrains {
            if !terrain_filters.is_empty() {
                for terrain in terrain_filters {
                    let search_pattern = format!("%{}%", terrain);
                    query = query.filter(terrain_text.like(search_pattern));
                }
            }
        }

        let vehicles = query
            .limit(super::DEFAULT_QUERY_LIMIT)
            .load::<CatalogVehicle>(self.conn)?;

        Ok(vehicles.iter().map(VehicleSummary::from).collect())
    }

    /// Get vehicle by name and source
    pub fn get_vehicle_by_name_and_source(
        &mut self,
        vehicle_name: &str,
        vehicle_source: &str,
    ) -> Result<Option<Vehicle>> {
        use crate::schema::catalog_vehicles::dsl::*;

        let catalog_vehicle = catalog_vehicles
            .filter(name.eq(vehicle_name))
            .filter(source.eq(vehicle_source))
            .first::<CatalogVehicle>(self.conn)
            .optional()?;

        match catalog_vehicle {
            Some(vehicle_record) => {
                let parsed_vehicle: Vehicle =
                    serde_json::from_str(&vehicle_record.full_vehicle_json)?;
                Ok(Some(parsed_vehicle))
            }
            None => Ok(None),
        }
    }

    /// Get all unique vehicle types for filtering
    pub fn get_all_vehicle_types(&mut self) -> Result<Vec<String>> {
        use crate::schema::catalog_vehicles::dsl::*;

        let types: Vec<Option<String>> = catalog_vehicles
            .select(vehicle_type)
            .distinct()
            .filter(vehicle_type.is_not_null())
            .load(self.conn)?;

        let mut result: Vec<String> = types
            .into_iter()
            .flatten()
            .filter(|t| !t.is_empty())
            .collect();

        result.sort();
        Ok(result)
    }

    /// Get all unique sizes for filtering
    pub fn get_all_sizes(&mut self) -> Result<Vec<String>> {
        use crate::schema::catalog_vehicles::dsl::*;

        let sizes: Vec<Option<String>> = catalog_vehicles
            .select(size)
            .distinct()
            .filter(size.is_not_null())
            .load(self.conn)?;

        let mut result: Vec<String> = sizes
            .into_iter()
            .flatten()
            .filter(|s| !s.is_empty())
            .collect();

        result.sort();
        Ok(result)
    }

    /// Get all unique terrains for filtering
    pub fn get_all_terrains(&mut self) -> Result<Vec<String>> {
        use crate::schema::catalog_vehicles::dsl::*;

        let terrain_strings: Vec<Option<String>> = catalog_vehicles
            .select(terrain_text)
            .distinct()
            .filter(terrain_text.is_not_null())
            .load(self.conn)?;

        let mut all_terrains = std::collections::HashSet::new();

        // Parse comma-separated terrains
        for terrain_str in terrain_strings.into_iter().flatten() {
            for terrain in terrain_str.split(',') {
                let trimmed = terrain.trim();
                if !trimmed.is_empty() {
                    all_terrains.insert(trimmed.to_string());
                }
            }
        }

        let mut result: Vec<String> = all_terrains.into_iter().collect();
        result.sort();
        Ok(result)
    }

    /// Get vehicle count by source for statistics
    pub fn get_vehicle_count_by_source(&mut self) -> Result<Vec<(String, i64)>> {
        use crate::schema::catalog_vehicles::dsl::*;

        let counts = catalog_vehicles
            .group_by(source)
            .select((source, diesel::dsl::count_star()))
            .load::<(String, i64)>(self.conn)?;

        Ok(counts)
    }

    /// Import all vehicle data from an uploaded book directory
    pub fn import_vehicles_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        info!(
            "Importing vehicles from book directory: {:?} (source: {})",
            book_dir, source
        );

        let mut total_imported = 0;
        let vehicle_files = Self::find_vehicle_files(book_dir)?;

        if vehicle_files.is_empty() {
            info!("No vehicle files found in book directory");
            return Ok(0);
        }

        info!("Found {} vehicle files to process", vehicle_files.len());

        for vehicle_file in vehicle_files {
            debug!("Processing vehicle file: {:?}", vehicle_file);

            match Self::import_vehicles_from_file(conn, &vehicle_file, source) {
                Ok(count) => {
                    info!("Imported {} vehicles from {:?}", count, vehicle_file);
                    total_imported += count;
                }
                Err(e) => {
                    error!("Failed to import vehicles from {:?}: {}", vehicle_file, e);
                    // Continue processing other files instead of failing completely
                }
            }
        }

        info!("Total vehicles imported: {}", total_imported);
        Ok(total_imported)
    }

    /// Find vehicle files in a book directory (vehicles/*.json files)
    fn find_vehicle_files(book_dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        // Check the vehicles directory
        let vehicles_dir = book_dir.join("vehicles");
        if vehicles_dir.exists() && vehicles_dir.is_dir() {
            let entries = fs::read_dir(&vehicles_dir)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // Skip fluff files for now
                    if !filename.starts_with("fluff-") {
                        files.push(path);
                    }
                }
            }
        }

        if files.is_empty() {
            debug!("No vehicle files found in {:?}", vehicles_dir);
        } else {
            debug!("Found {} vehicle files in {:?}", files.len(), vehicles_dir);
        }

        Ok(files)
    }

    /// Import vehicles from a single JSON file
    fn import_vehicles_from_file(
        conn: &mut SqliteConnection,
        file_path: &Path,
        source: &str,
    ) -> Result<usize> {
        debug!("Reading vehicles from file: {:?}", file_path);

        let content = fs::read_to_string(file_path)?;

        let data: VehicleData = serde_json::from_str(&content)?;

        if let Some(vehicles) = data.vehicle {
            if !vehicles.is_empty() {
                let new_vehicles: Vec<NewCatalogVehicle> = vehicles
                    .iter()
                    .map(|vehicle| {
                        let mut new_vehicle = NewCatalogVehicle::from(vehicle);
                        // Always override the source with the book source to ensure consistency
                        new_vehicle.source = source.to_string();

                        // Also update the source in the full_vehicle_json to maintain consistency
                        if let Ok(mut vehicle_json) = serde_json::from_str::<serde_json::Value>(
                            &new_vehicle.full_vehicle_json,
                        ) {
                            if let Some(obj) = vehicle_json.as_object_mut() {
                                obj.insert(
                                    "source".to_string(),
                                    serde_json::Value::String(source.to_string()),
                                );
                                if let Ok(updated_json) = serde_json::to_string(&vehicle_json) {
                                    new_vehicle.full_vehicle_json = updated_json;
                                }
                            }
                        }

                        new_vehicle
                    })
                    .collect();

                // Use batch insert with conflict resolution for better performance
                for new_vehicle in &new_vehicles {
                    diesel::insert_into(catalog_vehicles::table)
                        .values(new_vehicle)
                        .on_conflict((catalog_vehicles::name, catalog_vehicles::source))
                        .do_nothing()
                        .execute(conn)?;
                }

                info!(
                    "Successfully imported {} vehicles into database",
                    new_vehicles.len()
                );
                Ok(new_vehicles.len())
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    /// Remove all vehicles from a specific source
    pub fn remove_vehicles_from_source(conn: &mut SqliteConnection, source: &str) -> Result<usize> {
        info!("Removing vehicles from source: {}", source);

        let deleted =
            diesel::delete(catalog_vehicles::table.filter(catalog_vehicles::source.eq(source)))
                .execute(conn)?;

        info!("Removed {} vehicles from source: {}", deleted, source);
        Ok(deleted)
    }
}
