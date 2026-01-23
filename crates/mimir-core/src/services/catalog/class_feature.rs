//! Class Feature Service
//!
//! Service layer for accessing class feature catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{ClassFeature, ClassFeatureFilter};
use crate::services::{ServiceError, ServiceResult};

/// Service for accessing class feature catalog data.
pub struct ClassFeatureService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> ClassFeatureService<'a> {
    /// Create a new class feature service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Get a class feature by ID.
    pub fn get(&mut self, id: i32) -> ServiceResult<ClassFeature> {
        dal::get_class_feature(self.conn, id).map_err(ServiceError::from)
    }

    /// Get a class feature by name, source, and class.
    pub fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
        class_name: &str,
        class_source: &str,
    ) -> ServiceResult<Option<ClassFeature>> {
        dal::get_class_feature_by_name(self.conn, name, source, class_name, class_source)
            .map_err(ServiceError::from)
    }

    /// Get a class feature by name and class (any source).
    pub fn get_by_name_and_class(
        &mut self,
        name: &str,
        class_name: &str,
    ) -> ServiceResult<Option<ClassFeature>> {
        dal::get_class_feature_by_name_and_class(self.conn, name, class_name)
            .map_err(ServiceError::from)
    }

    /// List all class features for a specific class.
    pub fn list_by_class(
        &mut self,
        class_name: &str,
        class_source: &str,
    ) -> ServiceResult<Vec<ClassFeature>> {
        dal::list_class_features_by_class(self.conn, class_name, class_source)
            .map_err(ServiceError::from)
    }

    /// Search class features with filters.
    pub fn search(&mut self, filter: &ClassFeatureFilter) -> ServiceResult<Vec<ClassFeature>> {
        dal::search_class_features(self.conn, filter).map_err(ServiceError::from)
    }

    /// List all distinct sources.
    pub fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_class_feature_sources(self.conn).map_err(ServiceError::from)
    }

    /// Count all class features.
    pub fn count(&mut self) -> ServiceResult<i64> {
        dal::count_class_features(self.conn).map_err(ServiceError::from)
    }

    /// Count class features by source.
    pub fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_class_features_by_source(self.conn, source).map_err(ServiceError::from)
    }
}
