use super::types::{Entry, Image};
use crate::schema::catalog_rewards;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reward {
    pub name: String,
    pub source: String,
    pub page: Option<i32>,

    #[serde(rename = "type")]
    pub reward_type: Option<String>, // Blessing, Boon, Charm, etc.

    #[serde(default)]
    pub entries: Vec<Entry>,

    #[serde(rename = "prerequisite")]
    pub prerequisite: Option<Vec<serde_json::Value>>,

    // For boons that grant spells
    #[serde(rename = "additionalSpells")]
    pub additional_spells: Option<Vec<serde_json::Value>>,

    // Duration for temporary rewards
    pub duration: Option<String>,

    #[serde(rename = "basicRules")]
    pub basic_rules: Option<bool>,

    #[serde(rename = "hasFluff")]
    pub has_fluff: Option<bool>,

    #[serde(rename = "hasFluffImages")]
    pub has_fluff_images: Option<bool>,
}

// Summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardSummary {
    pub name: String,
    pub source: String,
    pub reward_type: String,
    pub description: String,
    pub has_prerequisites: bool,
}

impl From<&Reward> for RewardSummary {
    fn from(reward: &Reward) -> Self {
        Self {
            name: reward.name.clone(),
            source: reward.source.clone(),
            reward_type: format_reward_type(&reward.reward_type),
            description: extract_first_entry(&reward.entries),
            has_prerequisites: reward.prerequisite.is_some(),
        }
    }
}

// Fluff data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub images: Vec<Image>,
}

fn format_reward_type(reward_type: &Option<String>) -> String {
    match reward_type.as_deref() {
        Some("Blessing") => "Blessing".to_string(),
        Some("Boon") => "Epic Boon".to_string(),
        Some("Charm") => "Charm".to_string(),
        Some("Feat") => "Feat".to_string(),
        Some(other) => other.to_string(),
        None => "Reward".to_string(),
    }
}

fn extract_first_entry(entries: &[Entry]) -> String {
    entries
        .first()
        .map(|entry| match entry {
            Entry::Text(s) => s.clone(),
            Entry::Object(_) => "Complex reward description".to_string(),
        })
        .unwrap_or_else(|| "—".to_string())
}

// Container for JSON parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct RewardData {
    pub reward: Option<Vec<Reward>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RewardFluffData {
    #[serde(rename = "rewardFluff")]
    pub reward_fluff: Option<Vec<RewardFluff>>,
}

// Database models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = catalog_rewards)]
pub struct CatalogReward {
    pub id: i32,
    pub name: String,
    pub reward_type: String,
    pub description: String,
    pub has_prerequisites: i32, // SQLite INTEGER for boolean
    pub source: String,
    pub full_reward_json: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = catalog_rewards)]
pub struct NewCatalogReward {
    pub name: String,
    pub reward_type: String,
    pub description: String,
    pub has_prerequisites: i32,
    pub source: String,
    pub full_reward_json: String,
}

impl From<Reward> for NewCatalogReward {
    fn from(reward: Reward) -> Self {
        let summary = RewardSummary::from(&reward);
        let json = serde_json::to_string(&reward).unwrap_or_default();

        Self {
            name: summary.name,
            reward_type: summary.reward_type,
            description: summary.description,
            has_prerequisites: if summary.has_prerequisites { 1 } else { 0 },
            source: summary.source,
            full_reward_json: json,
        }
    }
}

// Filter struct for search operations
#[derive(Debug, Default)]
pub struct RewardFilters {
    pub name: Option<String>,
    pub search: Option<String>,
    pub reward_types: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
    pub has_prerequisites: Option<bool>,
}
