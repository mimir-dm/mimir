//! Subclass Service
//!
//! Service layer for accessing subclass catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::Subclass;
use crate::services::{ServiceError, ServiceResult};

/// Service for accessing subclass catalog data.
pub struct SubclassService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> SubclassService<'a> {
    /// Create a new subclass service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Get a subclass by ID.
    pub fn get(&mut self, id: i32) -> ServiceResult<Subclass> {
        dal::get_subclass(self.conn, id).map_err(ServiceError::from)
    }

    /// Get a subclass by name, class, and source.
    pub fn get_by_name_and_class(
        &mut self,
        name: &str,
        class_name: &str,
        source: &str,
    ) -> ServiceResult<Option<Subclass>> {
        dal::get_subclass_by_name(self.conn, name, class_name, source).map_err(ServiceError::from)
    }

    /// List all subclasses for a specific class.
    pub fn list_by_class(&mut self, class_name: &str) -> ServiceResult<Vec<Subclass>> {
        dal::list_subclasses_by_class(self.conn, class_name).map_err(ServiceError::from)
    }

    /// List all subclasses from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Subclass>> {
        dal::list_subclasses_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all subclasses.
    pub fn list_all(&mut self) -> ServiceResult<Vec<Subclass>> {
        dal::list_subclasses(self.conn).map_err(ServiceError::from)
    }

    /// Count all subclasses.
    pub fn count(&mut self) -> ServiceResult<i64> {
        dal::count_subclasses(self.conn).map_err(ServiceError::from)
    }
}
