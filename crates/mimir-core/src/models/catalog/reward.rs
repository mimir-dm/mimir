//! Reward Model
//!
//! Represents supernatural rewards: blessings, boons, charms, and gifts.

use crate::schema::rewards;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A supernatural reward from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = rewards)]
#[diesel(primary_key(id))]
pub struct Reward {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    /// Reward type (blessing, boon, charm, gift)
    pub reward_type: Option<String>,
    pub data: String,
}

impl Reward {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }

    /// Check if this is a blessing.
    pub fn is_blessing(&self) -> bool {
        self.reward_type
            .as_ref()
            .map_or(false, |t| t.eq_ignore_ascii_case("blessing"))
    }

    /// Check if this is a boon.
    pub fn is_boon(&self) -> bool {
        self.reward_type
            .as_ref()
            .map_or(false, |t| t.eq_ignore_ascii_case("boon"))
    }

    /// Check if this is a charm.
    pub fn is_charm(&self) -> bool {
        self.reward_type
            .as_ref()
            .map_or(false, |t| t.eq_ignore_ascii_case("charm"))
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = rewards)]
pub struct NewReward<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub reward_type: Option<&'a str>,
    pub data: &'a str,
}

impl<'a> NewReward<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self {
            name,
            source,
            reward_type: None,
            data,
        }
    }

    pub fn with_type(mut self, reward_type: &'a str) -> Self {
        self.reward_type = Some(reward_type);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_reward() {
        let reward = NewReward::new("Blessing of Health", "DMG", r#"{"name":"Blessing of Health"}"#)
            .with_type("blessing");
        assert_eq!(reward.name, "Blessing of Health");
        assert_eq!(reward.reward_type, Some("blessing"));
    }
}
