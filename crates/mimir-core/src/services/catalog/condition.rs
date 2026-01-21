//! Condition Service
//!
//! Service layer for accessing condition catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Condition, ConditionFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing condition catalog data.
pub struct ConditionService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> ConditionService<'a> {
    /// Create a new condition service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all conditions from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Condition>> {
        dal::list_conditions_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all conditions (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Condition>> {
        self.search_paginated(&ConditionFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }
}

impl<'a> CatalogEntityService for ConditionService<'a> {
    type Entity = Condition;
    type Filter = ConditionFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_conditions_paginated(self.conn, filter, limit, offset)
            .map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_condition_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_condition_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_condition_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_conditions(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_conditions_by_source(self.conn, source).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_conditions;
    use crate::models::catalog::NewCondition;
    use crate::test_utils::setup_test_db_with_sources;

    fn insert_test_conditions(conn: &mut SqliteConnection) {
        let conditions = vec![
            NewCondition::new("Blinded", "PHB", r#"{"name":"Blinded"}"#),
            NewCondition::new("Charmed", "PHB", r#"{"name":"Charmed"}"#),
            NewCondition::new("Deafened", "PHB", r#"{"name":"Deafened"}"#),
            NewCondition::new("Frightened", "PHB", r#"{"name":"Frightened"}"#),
        ];
        insert_conditions(conn, &conditions).expect("Failed to insert test conditions");
    }

    #[test]
    fn test_condition_service_search() {
        let mut conn = setup_test_db_with_sources();
        insert_test_conditions(&mut conn);

        let mut service = ConditionService::new(&mut conn);

        let results = service
            .search(&ConditionFilter::default())
            .expect("Search failed");
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_condition_service_get() {
        let mut conn = setup_test_db_with_sources();
        insert_test_conditions(&mut conn);

        let mut service = ConditionService::new(&mut conn);

        let blinded = service
            .get_by_name_and_source("Blinded", "PHB")
            .expect("Query failed")
            .expect("Blinded not found");
        assert_eq!(blinded.name, "Blinded");
    }

    #[test]
    fn test_condition_service_count() {
        let mut conn = setup_test_db_with_sources();
        insert_test_conditions(&mut conn);

        let mut service = ConditionService::new(&mut conn);

        assert_eq!(service.count().expect("Count failed"), 4);
    }
}
