//! Action Service
//!
//! Service layer for accessing action catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Action, ActionFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing action catalog data.
pub struct ActionService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> ActionService<'a> {
    /// Create a new action service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all actions from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Action>> {
        dal::list_actions_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all actions (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Action>> {
        self.search_paginated(&ActionFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }
}

impl<'a> CatalogEntityService for ActionService<'a> {
    type Entity = Action;
    type Filter = ActionFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_actions_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_action_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_action_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_action_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_actions(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_actions_by_source(self.conn, source).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_actions;
    use crate::models::catalog::NewAction;
    use crate::test_utils::setup_test_db_with_sources;

    fn insert_test_actions(conn: &mut SqliteConnection) {
        let actions = vec![
            NewAction::new("Attack", "PHB", r#"{"name":"Attack"}"#),
            NewAction::new("Dash", "PHB", r#"{"name":"Dash"}"#),
            NewAction::new("Dodge", "PHB", r#"{"name":"Dodge"}"#),
            NewAction::new("Help", "PHB", r#"{"name":"Help"}"#),
        ];
        insert_actions(conn, &actions).expect("Failed to insert test actions");
    }

    #[test]
    fn test_action_service_search() {
        let mut conn = setup_test_db_with_sources();
        insert_test_actions(&mut conn);

        let mut service = ActionService::new(&mut conn);

        let results = service
            .search(&ActionFilter::default())
            .expect("Search failed");
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_action_service_get() {
        let mut conn = setup_test_db_with_sources();
        insert_test_actions(&mut conn);

        let mut service = ActionService::new(&mut conn);

        let dash = service
            .get_by_name_and_source("Dash", "PHB")
            .expect("Query failed")
            .expect("Dash not found");
        assert_eq!(dash.name, "Dash");
    }

    #[test]
    fn test_action_service_count() {
        let mut conn = setup_test_db_with_sources();
        insert_test_actions(&mut conn);

        let mut service = ActionService::new(&mut conn);

        assert_eq!(service.count().expect("Count failed"), 4);
    }
}
