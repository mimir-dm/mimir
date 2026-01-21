//! Trap Service
//!
//! Service layer for accessing trap catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Trap, TrapFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing trap catalog data.
pub struct TrapService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> TrapService<'a> {
    /// Create a new trap service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all traps from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Trap>> {
        dal::list_traps_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all traps (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Trap>> {
        self.search_paginated(&TrapFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }

    /// List simple traps.
    pub fn list_simple(&mut self) -> ServiceResult<Vec<Trap>> {
        let filter = TrapFilter::new().with_tier("simple");
        self.search(&filter)
    }

    /// List complex traps.
    pub fn list_complex(&mut self) -> ServiceResult<Vec<Trap>> {
        let filter = TrapFilter::new().with_tier("complex");
        self.search(&filter)
    }
}

impl<'a> CatalogEntityService for TrapService<'a> {
    type Entity = Trap;
    type Filter = TrapFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_traps_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_trap_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_trap_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_trap_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_traps(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_traps_by_source(self.conn, source).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_traps;
    use crate::models::catalog::NewTrap;
    use crate::test_utils::setup_test_db_with_sources;

    fn insert_test_traps(conn: &mut SqliteConnection) {
        let traps = vec![
            NewTrap::new("Pit Trap", "DMG", r#"{"name":"Pit Trap"}"#).with_tier("simple"),
            NewTrap::new("Poison Needle", "DMG", r#"{"name":"Poison Needle"}"#).with_tier("simple"),
            NewTrap::new("Sphere of Annihilation", "DMG", r#"{"name":"Sphere of Annihilation"}"#)
                .with_tier("complex"),
        ];
        insert_traps(conn, &traps).expect("Failed to insert test traps");
    }

    #[test]
    fn test_trap_service_search() {
        let mut conn = setup_test_db_with_sources();
        insert_test_traps(&mut conn);

        let mut service = TrapService::new(&mut conn);

        let results = service.search(&TrapFilter::default()).expect("Search failed");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_trap_service_get() {
        let mut conn = setup_test_db_with_sources();
        insert_test_traps(&mut conn);

        let mut service = TrapService::new(&mut conn);

        let pit = service
            .get_by_name_and_source("Pit Trap", "DMG")
            .expect("Query failed")
            .expect("Pit Trap not found");
        assert_eq!(pit.name, "Pit Trap");
    }

    #[test]
    fn test_trap_service_count() {
        let mut conn = setup_test_db_with_sources();
        insert_test_traps(&mut conn);

        let mut service = TrapService::new(&mut conn);

        assert_eq!(service.count().expect("Count failed"), 3);
    }

    #[test]
    fn test_trap_service_by_tier() {
        let mut conn = setup_test_db_with_sources();
        insert_test_traps(&mut conn);

        let mut service = TrapService::new(&mut conn);

        let simple = service.list_simple().expect("Search failed");
        assert_eq!(simple.len(), 2);

        let complex = service.list_complex().expect("Search failed");
        assert_eq!(complex.len(), 1);
    }
}
