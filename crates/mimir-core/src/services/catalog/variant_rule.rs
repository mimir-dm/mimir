//! VariantRule Service
//!
//! Service layer for accessing variant rule catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{VariantRule, VariantRuleFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing variant rule catalog data.
pub struct VariantRuleService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> VariantRuleService<'a> {
    /// Create a new variant rule service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all variant rules from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<VariantRule>> {
        dal::list_variant_rules_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all variant rules (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<VariantRule>> {
        self.search_paginated(&VariantRuleFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }
}

impl<'a> CatalogEntityService for VariantRuleService<'a> {
    type Entity = VariantRule;
    type Filter = VariantRuleFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_variant_rules_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_variant_rule_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_variant_rule_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_variant_rule_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_variant_rules(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_variant_rules_by_source(self.conn, source).map_err(ServiceError::from)
    }
}
