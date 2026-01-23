//! CatalogTable Service
//!
//! Service layer for accessing catalog table data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{CatalogTable, CatalogTableFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing catalog table data.
pub struct CatalogTableService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> CatalogTableService<'a> {
    /// Create a new catalog table service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all catalog tables from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<CatalogTable>> {
        dal::list_catalog_tables_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all catalog tables (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<CatalogTable>> {
        self.search_paginated(&CatalogTableFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }
}

impl<'a> CatalogEntityService for CatalogTableService<'a> {
    type Entity = CatalogTable;
    type Filter = CatalogTableFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_catalog_tables_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_catalog_table_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_catalog_table_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_catalog_table_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_catalog_tables(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_catalog_tables_by_source(self.conn, source).map_err(ServiceError::from)
    }
}
