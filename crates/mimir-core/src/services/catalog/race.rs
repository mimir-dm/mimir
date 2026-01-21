//! Race Service
//!
//! Service layer for accessing race catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Race, RaceFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing race catalog data.
pub struct RaceService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> RaceService<'a> {
    /// Create a new race service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all races from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Race>> {
        dal::list_races_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all races (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Race>> {
        self.search_paginated(&RaceFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }
}

impl<'a> CatalogEntityService for RaceService<'a> {
    type Entity = Race;
    type Filter = RaceFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_races_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_race_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_race_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_race_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_races(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_races_by_source(self.conn, source).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_races;
    use crate::models::catalog::NewRace;
    use crate::test_utils::setup_test_db_with_sources;

    fn insert_test_races(conn: &mut SqliteConnection) {
        let races = vec![
            NewRace::new("Elf", "PHB", r#"{"name":"Elf"}"#),
            NewRace::new("Dwarf", "PHB", r#"{"name":"Dwarf"}"#),
            NewRace::new("Half-Elf", "PHB", r#"{"name":"Half-Elf"}"#),
            NewRace::new("Tiefling", "PHB", r#"{"name":"Tiefling"}"#),
            NewRace::new("Aarakocra", "MPMM", r#"{"name":"Aarakocra"}"#),
        ];
        insert_races(conn, &races).expect("Failed to insert test races");
    }

    #[test]
    fn test_race_service_search() {
        let mut conn = setup_test_db_with_sources();
        insert_test_races(&mut conn);

        let mut service = RaceService::new(&mut conn);

        // Search all
        let results = service.search(&RaceFilter::default()).expect("Search failed");
        assert_eq!(results.len(), 5);

        // Search by name
        let filter = RaceFilter::new().with_name_contains("elf");
        let results = service.search(&filter).expect("Search failed");
        assert_eq!(results.len(), 2); // Elf and Half-Elf
    }

    #[test]
    fn test_race_service_get() {
        let mut conn = setup_test_db_with_sources();
        insert_test_races(&mut conn);

        let mut service = RaceService::new(&mut conn);

        let elf = service
            .get_by_name_and_source("Elf", "PHB")
            .expect("Query failed")
            .expect("Elf not found");
        assert_eq!(elf.name, "Elf");
    }

    #[test]
    fn test_race_service_list_sources() {
        let mut conn = setup_test_db_with_sources();
        insert_test_races(&mut conn);

        let mut service = RaceService::new(&mut conn);
        let sources = service.list_sources().expect("List sources failed");

        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&"PHB".to_string()));
        assert!(sources.contains(&"MPMM".to_string()));
    }

    #[test]
    fn test_race_service_count() {
        let mut conn = setup_test_db_with_sources();
        insert_test_races(&mut conn);

        let mut service = RaceService::new(&mut conn);

        assert_eq!(service.count().expect("Count failed"), 5);
        assert_eq!(service.count_by_source("PHB").expect("Count failed"), 4);
        assert_eq!(service.count_by_source("MPMM").expect("Count failed"), 1);
    }
}
