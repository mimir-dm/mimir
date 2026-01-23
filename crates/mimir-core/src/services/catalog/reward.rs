//! Reward Service
//!
//! Service layer for accessing reward catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Reward, RewardFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing reward catalog data.
pub struct RewardService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> RewardService<'a> {
    /// Create a new reward service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all rewards from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Reward>> {
        dal::list_rewards_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all rewards (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Reward>> {
        self.search_paginated(&RewardFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }

    /// List rewards by type.
    pub fn list_by_type(&mut self, reward_type: &str) -> ServiceResult<Vec<Reward>> {
        dal::list_rewards_by_type(self.conn, reward_type).map_err(ServiceError::from)
    }
}

impl<'a> CatalogEntityService for RewardService<'a> {
    type Entity = Reward;
    type Filter = RewardFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_rewards_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_reward_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_reward_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_reward_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_rewards(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_rewards_by_source(self.conn, source).map_err(ServiceError::from)
    }
}
