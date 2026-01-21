//! Background Service
//!
//! Service layer for accessing background catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Background, BackgroundFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing background catalog data.
pub struct BackgroundService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> BackgroundService<'a> {
    /// Create a new background service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all backgrounds from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Background>> {
        dal::list_backgrounds_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all backgrounds (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Background>> {
        self.search_paginated(&BackgroundFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }
}

impl<'a> CatalogEntityService for BackgroundService<'a> {
    type Entity = Background;
    type Filter = BackgroundFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_backgrounds_paginated(self.conn, filter, limit, offset)
            .map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_background_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_background_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_background_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_backgrounds(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_backgrounds_by_source(self.conn, source).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_backgrounds;
    use crate::models::catalog::NewBackground;
    use crate::test_utils::setup_test_db_with_sources;

    fn insert_test_backgrounds(conn: &mut SqliteConnection) {
        let backgrounds = vec![
            NewBackground::new("Acolyte", "PHB", r#"{"name":"Acolyte"}"#),
            NewBackground::new("Criminal", "PHB", r#"{"name":"Criminal"}"#),
            NewBackground::new("Noble", "PHB", r#"{"name":"Noble"}"#),
            NewBackground::new("Soldier", "PHB", r#"{"name":"Soldier"}"#),
            NewBackground::new("Smuggler", "GoS", r#"{"name":"Smuggler"}"#),
        ];
        insert_backgrounds(conn, &backgrounds).expect("Failed to insert test backgrounds");
    }

    #[test]
    fn test_background_service_search() {
        let mut conn = setup_test_db_with_sources();
        insert_test_backgrounds(&mut conn);

        let mut service = BackgroundService::new(&mut conn);

        // Search all
        let results = service
            .search(&BackgroundFilter::default())
            .expect("Search failed");
        assert_eq!(results.len(), 5);

        // Search by name
        let filter = BackgroundFilter::new().with_name_contains("ol");
        let results = service.search(&filter).expect("Search failed");
        assert_eq!(results.len(), 2); // Noble and Soldier
    }

    #[test]
    fn test_background_service_get() {
        let mut conn = setup_test_db_with_sources();
        insert_test_backgrounds(&mut conn);

        let mut service = BackgroundService::new(&mut conn);

        let acolyte = service
            .get_by_name_and_source("Acolyte", "PHB")
            .expect("Query failed")
            .expect("Acolyte not found");
        assert_eq!(acolyte.name, "Acolyte");
    }

    #[test]
    fn test_background_service_list_sources() {
        let mut conn = setup_test_db_with_sources();
        insert_test_backgrounds(&mut conn);

        let mut service = BackgroundService::new(&mut conn);
        let sources = service.list_sources().expect("List sources failed");

        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&"PHB".to_string()));
        assert!(sources.contains(&"GoS".to_string()));
    }

    #[test]
    fn test_background_service_count() {
        let mut conn = setup_test_db_with_sources();
        insert_test_backgrounds(&mut conn);

        let mut service = BackgroundService::new(&mut conn);

        assert_eq!(service.count().expect("Count failed"), 5);
        assert_eq!(service.count_by_source("PHB").expect("Count failed"), 4);
        assert_eq!(service.count_by_source("GoS").expect("Count failed"), 1);
    }
}
