//! Psionic Service
//!
//! Service layer for accessing psionic catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Psionic, PsionicFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing psionic catalog data.
pub struct PsionicService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> PsionicService<'a> {
    /// Create a new psionic service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all psionics from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Psionic>> {
        dal::list_psionics_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all psionics (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Psionic>> {
        self.search_paginated(&PsionicFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }

    /// List psionics by type.
    pub fn list_by_type(&mut self, psionic_type: &str) -> ServiceResult<Vec<Psionic>> {
        dal::list_psionics_by_type(self.conn, psionic_type).map_err(ServiceError::from)
    }

    /// List psionics by order.
    pub fn list_by_order(&mut self, order: &str) -> ServiceResult<Vec<Psionic>> {
        dal::list_psionics_by_order(self.conn, order).map_err(ServiceError::from)
    }
}

impl<'a> CatalogEntityService for PsionicService<'a> {
    type Entity = Psionic;
    type Filter = PsionicFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_psionics_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_psionic_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_psionic_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_psionic_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_psionics(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_psionics_by_source(self.conn, source).map_err(ServiceError::from)
    }
}
