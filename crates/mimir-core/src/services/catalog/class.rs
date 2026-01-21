//! Class Service
//!
//! Service layer for accessing class catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Class, ClassFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing class catalog data.
pub struct ClassService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> ClassService<'a> {
    /// Create a new class service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all classes from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Class>> {
        dal::list_classes_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all classes (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Class>> {
        self.search_paginated(&ClassFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }
}

impl<'a> CatalogEntityService for ClassService<'a> {
    type Entity = Class;
    type Filter = ClassFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_classes_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_class_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_class_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_class_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_classes(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_classes_by_source(self.conn, source).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_classes;
    use crate::models::catalog::NewClass;
    use crate::test_utils::setup_test_db_with_sources;

    fn insert_test_classes(conn: &mut SqliteConnection) {
        let classes = vec![
            NewClass::new("Fighter", "PHB", r#"{"name":"Fighter"}"#),
            NewClass::new("Wizard", "PHB", r#"{"name":"Wizard"}"#),
            NewClass::new("Cleric", "PHB", r#"{"name":"Cleric"}"#),
            NewClass::new("Rogue", "PHB", r#"{"name":"Rogue"}"#),
            NewClass::new("Blood Hunter", "CR", r#"{"name":"Blood Hunter"}"#),
        ];
        insert_classes(conn, &classes).expect("Failed to insert test classes");
    }

    #[test]
    fn test_class_service_search() {
        let mut conn = setup_test_db_with_sources();
        insert_test_classes(&mut conn);

        let mut service = ClassService::new(&mut conn);

        // Search all
        let results = service.search(&ClassFilter::default()).expect("Search failed");
        assert_eq!(results.len(), 5);

        // Search by name
        let filter = ClassFilter::new().with_name_contains("er");
        let results = service.search(&filter).expect("Search failed");
        assert_eq!(results.len(), 3); // Fighter, Cleric, and Blood Hunter
    }

    #[test]
    fn test_class_service_get() {
        let mut conn = setup_test_db_with_sources();
        insert_test_classes(&mut conn);

        let mut service = ClassService::new(&mut conn);

        let wizard = service
            .get_by_name_and_source("Wizard", "PHB")
            .expect("Query failed")
            .expect("Wizard not found");
        assert_eq!(wizard.name, "Wizard");
    }

    #[test]
    fn test_class_service_list_sources() {
        let mut conn = setup_test_db_with_sources();
        insert_test_classes(&mut conn);

        let mut service = ClassService::new(&mut conn);
        let sources = service.list_sources().expect("List sources failed");

        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&"PHB".to_string()));
        assert!(sources.contains(&"CR".to_string()));
    }

    #[test]
    fn test_class_service_count() {
        let mut conn = setup_test_db_with_sources();
        insert_test_classes(&mut conn);

        let mut service = ClassService::new(&mut conn);

        assert_eq!(service.count().expect("Count failed"), 5);
        assert_eq!(service.count_by_source("PHB").expect("Count failed"), 4);
        assert_eq!(service.count_by_source("CR").expect("Count failed"), 1);
    }
}
