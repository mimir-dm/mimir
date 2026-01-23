//! Vehicle Service
//!
//! Service layer for accessing vehicle catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Vehicle, VehicleFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing vehicle catalog data.
pub struct VehicleService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> VehicleService<'a> {
    /// Create a new vehicle service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all vehicles from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Vehicle>> {
        dal::list_vehicles_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all vehicles (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Vehicle>> {
        self.search_paginated(&VehicleFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }

    /// List vehicles by type.
    pub fn list_by_type(&mut self, vehicle_type: &str) -> ServiceResult<Vec<Vehicle>> {
        dal::list_vehicles_by_type(self.conn, vehicle_type).map_err(ServiceError::from)
    }
}

impl<'a> CatalogEntityService for VehicleService<'a> {
    type Entity = Vehicle;
    type Filter = VehicleFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_vehicles_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_vehicle_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_vehicle_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_vehicle_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_vehicles(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_vehicles_by_source(self.conn, source).map_err(ServiceError::from)
    }
}
