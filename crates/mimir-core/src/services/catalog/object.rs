//! Object Service
//!
//! Service layer for accessing object catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Object, ObjectFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing object catalog data.
pub struct ObjectService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> ObjectService<'a> {
    /// Create a new object service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all objects from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Object>> {
        dal::list_objects_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all objects (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Object>> {
        self.search_paginated(&ObjectFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }

    /// List objects by type.
    pub fn list_by_type(&mut self, object_type: &str) -> ServiceResult<Vec<Object>> {
        dal::list_objects_by_type(self.conn, object_type).map_err(ServiceError::from)
    }
}

impl<'a> CatalogEntityService for ObjectService<'a> {
    type Entity = Object;
    type Filter = ObjectFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_objects_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_object_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_object_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_object_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_objects(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_objects_by_source(self.conn, source).map_err(ServiceError::from)
    }
}
