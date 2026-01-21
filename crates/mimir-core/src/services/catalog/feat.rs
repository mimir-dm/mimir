//! Feat Service
//!
//! Service layer for accessing feat catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Feat, FeatFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing feat catalog data.
pub struct FeatService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> FeatService<'a> {
    /// Create a new feat service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all feats from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Feat>> {
        dal::list_feats_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all feats (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Feat>> {
        self.search_paginated(&FeatFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }
}

impl<'a> CatalogEntityService for FeatService<'a> {
    type Entity = Feat;
    type Filter = FeatFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_feats_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_feat_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_feat_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_feat_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_feats(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_feats_by_source(self.conn, source).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_feats;
    use crate::models::catalog::NewFeat;
    use crate::test_utils::setup_test_db_with_sources;

    fn insert_test_feats(conn: &mut SqliteConnection) {
        let feats = vec![
            NewFeat::new("Alert", "PHB", r#"{"name":"Alert"}"#),
            NewFeat::new("Sharpshooter", "PHB", r#"{"name":"Sharpshooter"}"#),
            NewFeat::new("Great Weapon Master", "PHB", r#"{"name":"Great Weapon Master"}"#),
            NewFeat::new("Fey Touched", "TCE", r#"{"name":"Fey Touched"}"#),
        ];
        insert_feats(conn, &feats).expect("Failed to insert test feats");
    }

    #[test]
    fn test_feat_service_search() {
        let mut conn = setup_test_db_with_sources();
        insert_test_feats(&mut conn);

        let mut service = FeatService::new(&mut conn);

        let results = service.search(&FeatFilter::default()).expect("Search failed");
        assert_eq!(results.len(), 4);

        let filter = FeatFilter::new().with_name_contains("sharp");
        let results = service.search(&filter).expect("Search failed");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_feat_service_get() {
        let mut conn = setup_test_db_with_sources();
        insert_test_feats(&mut conn);

        let mut service = FeatService::new(&mut conn);

        let alert = service
            .get_by_name_and_source("Alert", "PHB")
            .expect("Query failed")
            .expect("Alert not found");
        assert_eq!(alert.name, "Alert");
    }

    #[test]
    fn test_feat_service_count() {
        let mut conn = setup_test_db_with_sources();
        insert_test_feats(&mut conn);

        let mut service = FeatService::new(&mut conn);

        assert_eq!(service.count().expect("Count failed"), 4);
        assert_eq!(service.count_by_source("PHB").expect("Count failed"), 3);
    }
}
