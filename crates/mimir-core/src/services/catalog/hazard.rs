//! Hazard Service
//!
//! Service layer for accessing hazard catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Hazard, HazardFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing hazard catalog data.
pub struct HazardService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> HazardService<'a> {
    /// Create a new hazard service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all hazards from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Hazard>> {
        dal::list_hazards_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all hazards (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Hazard>> {
        self.search_paginated(&HazardFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }
}

impl<'a> CatalogEntityService for HazardService<'a> {
    type Entity = Hazard;
    type Filter = HazardFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_hazards_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_hazard_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_hazard_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_hazard_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_hazards(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_hazards_by_source(self.conn, source).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_hazards;
    use crate::models::catalog::NewHazard;
    use crate::test_utils::setup_test_db_with_sources;

    fn insert_test_hazards(conn: &mut SqliteConnection) {
        let hazards = vec![
            NewHazard::new("Brown Mold", "DMG", r#"{"name":"Brown Mold"}"#),
            NewHazard::new("Green Slime", "DMG", r#"{"name":"Green Slime"}"#),
            NewHazard::new("Yellow Mold", "DMG", r#"{"name":"Yellow Mold"}"#),
        ];
        insert_hazards(conn, &hazards).expect("Failed to insert test hazards");
    }

    #[test]
    fn test_hazard_service_search() {
        let mut conn = setup_test_db_with_sources();
        insert_test_hazards(&mut conn);

        let mut service = HazardService::new(&mut conn);

        let results = service
            .search(&HazardFilter::default())
            .expect("Search failed");
        assert_eq!(results.len(), 3);

        let filter = HazardFilter::new().with_name_contains("mold");
        let results = service.search(&filter).expect("Search failed");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_hazard_service_get() {
        let mut conn = setup_test_db_with_sources();
        insert_test_hazards(&mut conn);

        let mut service = HazardService::new(&mut conn);

        let mold = service
            .get_by_name_and_source("Brown Mold", "DMG")
            .expect("Query failed")
            .expect("Brown Mold not found");
        assert_eq!(mold.name, "Brown Mold");
    }

    #[test]
    fn test_hazard_service_count() {
        let mut conn = setup_test_db_with_sources();
        insert_test_hazards(&mut conn);

        let mut service = HazardService::new(&mut conn);

        assert_eq!(service.count().expect("Count failed"), 3);
    }
}
