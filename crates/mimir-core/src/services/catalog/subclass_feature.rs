//! Subclass Feature Service
//!
//! Service layer for accessing subclass feature catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{SubclassFeature, SubclassFeatureFilter};
use crate::services::{ServiceError, ServiceResult};

/// Service for accessing subclass feature catalog data.
pub struct SubclassFeatureService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> SubclassFeatureService<'a> {
    /// Create a new subclass feature service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Get a subclass feature by ID.
    pub fn get(&mut self, id: i32) -> ServiceResult<SubclassFeature> {
        dal::get_subclass_feature(self.conn, id).map_err(ServiceError::from)
    }

    /// Get a subclass feature by name and subclass.
    pub fn get_by_name_and_subclass(
        &mut self,
        name: &str,
        subclass_name: &str,
        subclass_source: &str,
    ) -> ServiceResult<Option<SubclassFeature>> {
        dal::get_subclass_feature_by_name(self.conn, name, subclass_name, subclass_source)
            .map_err(ServiceError::from)
    }

    /// List all subclass features for a specific subclass.
    pub fn list_by_subclass(
        &mut self,
        subclass_name: &str,
        subclass_source: &str,
    ) -> ServiceResult<Vec<SubclassFeature>> {
        dal::list_subclass_features_by_subclass(self.conn, subclass_name, subclass_source)
            .map_err(ServiceError::from)
    }

    /// Search subclass features with filters.
    pub fn search(&mut self, filter: &SubclassFeatureFilter) -> ServiceResult<Vec<SubclassFeature>> {
        dal::search_subclass_features(self.conn, filter).map_err(ServiceError::from)
    }

    /// List all distinct sources.
    pub fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_subclass_feature_sources(self.conn).map_err(ServiceError::from)
    }

    /// Count all subclass features.
    pub fn count(&mut self) -> ServiceResult<i64> {
        dal::count_subclass_features(self.conn).map_err(ServiceError::from)
    }

    /// Count subclass features by source.
    pub fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_subclass_features_by_source(self.conn, source).map_err(ServiceError::from)
    }
}
