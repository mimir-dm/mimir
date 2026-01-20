//! Reward catalog service.
//!
//! Provides database-backed reward search, retrieval, and import functionality.
//! Supports filtering by name, type, prerequisites, and source.

use crate::error::Result;
use crate::models::catalog::{
    CatalogReward, NewCatalogReward, Reward, RewardData, RewardFilters, RewardSummary,
};
use crate::schema::catalog_rewards;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Service for searching and managing rewards in the catalog.
pub struct RewardService;

impl RewardService {
    /// Search rewards with optional filters
    pub fn search_rewards(
        conn: &mut SqliteConnection,
        filters: RewardFilters,
    ) -> Result<Vec<RewardSummary>> {
        debug!("Searching rewards with filters: {:?}", filters);

        let mut query = catalog_rewards::table.into_boxed();

        // Apply name filter
        if let Some(name) = filters.name {
            query = query.filter(catalog_rewards::name.eq(name));
        }

        // Apply search filter (searches name, type, and description)
        if let Some(search) = filters.search {
            let search_pattern = format!("%{}%", search.to_lowercase());
            let pattern_clone1 = search_pattern.clone();
            let pattern_clone2 = search_pattern.clone();
            query = query.filter(
                catalog_rewards::name
                    .like(search_pattern)
                    .or(catalog_rewards::reward_type.like(pattern_clone1))
                    .or(catalog_rewards::description.like(pattern_clone2)),
            );
        }

        // Apply reward type filter
        if let Some(reward_types) = filters.reward_types {
            if !reward_types.is_empty() {
                query = query.filter(catalog_rewards::reward_type.eq_any(reward_types));
            }
        }

        // Apply source filter
        if let Some(sources) = filters.sources {
            if !sources.is_empty() {
                query = query.filter(catalog_rewards::source.eq_any(sources));
            }
        }

        // Apply prerequisites filter
        if let Some(has_prerequisites) = filters.has_prerequisites {
            let filter_value = if has_prerequisites { 1 } else { 0 };
            query = query.filter(catalog_rewards::has_prerequisites.eq(filter_value));
        }

        let catalog_rewards: Vec<CatalogReward> =
            query.select(CatalogReward::as_select()).load(conn)?;

        let summaries: Vec<RewardSummary> = catalog_rewards
            .into_iter()
            .map(|cr| RewardSummary {
                name: cr.name,
                source: cr.source,
                reward_type: cr.reward_type,
                description: cr.description,
                has_prerequisites: cr.has_prerequisites != 0,
            })
            .collect();

        debug!("Found {} rewards matching filters", summaries.len());
        Ok(summaries)
    }

    /// Get a specific reward by ID
    pub fn get_reward_by_id(conn: &mut SqliteConnection, reward_id: i32) -> Result<Option<Reward>> {
        debug!("Getting reward by ID: {}", reward_id);

        let catalog_reward: Option<CatalogReward> = catalog_rewards::table
            .find(reward_id)
            .first(conn)
            .optional()?;

        match catalog_reward {
            Some(cr) => {
                // Parse the full JSON back to the original Reward type
                let reward = serde_json::from_str::<Reward>(&cr.full_reward_json)?;
                Ok(Some(reward))
            }
            None => Ok(None),
        }
    }

    /// Get a specific reward by name and source
    pub fn get_reward_by_name_and_source(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<Reward>> {
        debug!("Getting reward by name '{}' and source '{}'", name, source);

        let catalog_reward: Option<CatalogReward> = catalog_rewards::table
            .filter(catalog_rewards::name.eq(name))
            .filter(catalog_rewards::source.eq(source))
            .first(conn)
            .optional()?;

        match catalog_reward {
            Some(cr) => {
                // Parse the full JSON back to the original Reward type
                let reward = serde_json::from_str::<Reward>(&cr.full_reward_json)?;
                Ok(Some(reward))
            }
            None => Ok(None),
        }
    }

    /// Get all unique reward types
    pub fn get_reward_types(conn: &mut SqliteConnection) -> Result<Vec<String>> {
        debug!("Getting all reward types");

        let mut types: Vec<String> = catalog_rewards::table
            .select(catalog_rewards::reward_type)
            .distinct()
            .load(conn)?;

        types.sort();
        Ok(types)
    }

    /// Get all unique sources
    pub fn get_sources(conn: &mut SqliteConnection) -> Result<Vec<String>> {
        debug!("Getting all sources");

        let mut sources: Vec<String> = catalog_rewards::table
            .select(catalog_rewards::source)
            .distinct()
            .load(conn)?;

        sources.sort();
        Ok(sources)
    }

    /// Get total count of rewards
    pub fn get_reward_count(conn: &mut SqliteConnection) -> Result<i64> {
        debug!("Getting reward count");

        catalog_rewards::table
            .count()
            .get_result(conn)
            .map_err(Into::into)
    }

    /// Import all reward data from an uploaded book directory
    pub fn import_rewards_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        info!(
            "Importing rewards from book directory: {:?} (source: {})",
            book_dir, source
        );

        let rewards_dir = book_dir.join("rewards");
        if !rewards_dir.exists() || !rewards_dir.is_dir() {
            debug!("No rewards directory found in book: {:?}", book_dir);
            return Ok(0);
        }

        let mut imported_count = 0;

        // Read all JSON files in the rewards directory
        let entries = fs::read_dir(&rewards_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Skip fluff files and non-JSON files
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("fluff") || !filename.ends_with(".json") {
                    continue;
                }
            }

            debug!("Processing reward file: {:?}", path);

            let content = fs::read_to_string(&path)?;
            let reward_data: RewardData = serde_json::from_str(&content)?;

            if let Some(rewards) = reward_data.reward {
                let new_rewards: Vec<NewCatalogReward> = rewards
                    .into_iter()
                    .map(|mut reward| {
                        reward.source = source.to_string();
                        NewCatalogReward::from(reward)
                    })
                    .collect();

                if !new_rewards.is_empty() {
                    let inserted = diesel::insert_into(catalog_rewards::table)
                        .values(&new_rewards)
                        .execute(conn)?;

                    imported_count += inserted;
                    info!("Imported {} rewards from {:?}", inserted, path);
                }
            }
        }

        info!(
            "Successfully imported {} rewards from source: {}",
            imported_count, source
        );
        Ok(imported_count)
    }

    /// Remove all rewards from a specific source
    pub fn remove_rewards_by_source(conn: &mut SqliteConnection, source: &str) -> Result<usize> {
        info!("Removing rewards from source: {}", source);

        let deleted =
            diesel::delete(catalog_rewards::table.filter(catalog_rewards::source.eq(source)))
                .execute(conn)?;

        info!("Removed {} rewards from source: {}", deleted, source);
        Ok(deleted)
    }
}
