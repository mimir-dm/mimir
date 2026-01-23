//! Deity Service
//!
//! Service layer for accessing deity catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Deity, DeityFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing deity catalog data.
pub struct DeityService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> DeityService<'a> {
    /// Create a new deity service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all deities from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Deity>> {
        dal::list_deities_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all deities (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Deity>> {
        self.search_paginated(&DeityFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }

    /// List deities by pantheon.
    pub fn list_by_pantheon(&mut self, pantheon: &str) -> ServiceResult<Vec<Deity>> {
        dal::list_deities_by_pantheon(self.conn, pantheon).map_err(ServiceError::from)
    }
}

impl<'a> CatalogEntityService for DeityService<'a> {
    type Entity = Deity;
    type Filter = DeityFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_deities_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_deity_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_deity_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_deity_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_deities(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_deities_by_source(self.conn, source).map_err(ServiceError::from)
    }
}
