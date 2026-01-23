//! OptionalFeature Service
//!
//! Service layer for accessing optional feature catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{OptionalFeature, OptionalFeatureFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing optional feature catalog data.
pub struct OptionalFeatureService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> OptionalFeatureService<'a> {
    /// Create a new optional feature service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all optional features from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<OptionalFeature>> {
        dal::list_optional_features_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all optional features (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<OptionalFeature>> {
        self.search_paginated(&OptionalFeatureFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }

    /// List optional features by type.
    pub fn list_by_type(&mut self, feature_type: &str) -> ServiceResult<Vec<OptionalFeature>> {
        dal::list_optional_features_by_type(self.conn, feature_type).map_err(ServiceError::from)
    }
}

impl<'a> CatalogEntityService for OptionalFeatureService<'a> {
    type Entity = OptionalFeature;
    type Filter = OptionalFeatureFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_optional_features_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_optional_feature_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_optional_feature_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_optional_feature_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_optional_features(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_optional_features_by_source(self.conn, source).map_err(ServiceError::from)
    }
}
