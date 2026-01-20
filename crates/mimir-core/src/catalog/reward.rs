//! Reward catalog extraction types
//!
//! Types for deserializing 5etools reward JSON data (blessings, boons, charms).

use super::types::SrdValue;
use serde::{Deserialize, Serialize};

/// A D&D 5e reward from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reward {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Reward type: Blessing, Boon, Charm, etc.
    #[serde(rename = "type", default)]
    pub reward_type: Option<String>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    /// Prerequisites (stored as JSON blob)
    #[serde(default)]
    pub prerequisite: Option<Vec<serde_json::Value>>,

    /// Additional spells granted
    #[serde(default)]
    pub additional_spells: Option<Vec<serde_json::Value>>,

    /// Duration for temporary rewards
    #[serde(default)]
    pub duration: Option<String>,

    // Flags
    #[serde(default)]
    pub basic_rules: Option<bool>,
    #[serde(default)]
    pub has_fluff: Option<bool>,
    #[serde(default)]
    pub has_fluff_images: Option<bool>,

    // SRD status
    #[serde(default)]
    pub srd: Option<SrdValue>,
}

/// Container for reward data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardData {
    #[serde(default)]
    pub reward: Vec<Reward>,
}

/// Reward fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub images: Option<Vec<serde_json::Value>>,
}

/// Container for reward fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardFluffData {
    #[serde(default)]
    pub reward_fluff: Vec<RewardFluff>,
}

/// Get human-readable name for reward type.
pub fn get_reward_type_name(reward_type: Option<&str>) -> &'static str {
    match reward_type {
        Some("Blessing") => "Blessing",
        Some("Boon") => "Epic Boon",
        Some("Charm") => "Charm",
        Some("Feat") => "Feat",
        _ => "Reward",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_blessing() {
        let json = json!({
            "name": "Blessing of Health",
            "source": "DMG",
            "type": "Blessing",
            "entries": ["Your Constitution score increases by 2, up to a maximum of 22."]
        });
        let reward: Reward = serde_json::from_value(json).unwrap();
        assert_eq!(reward.name, "Blessing of Health");
        assert_eq!(reward.reward_type, Some("Blessing".to_string()));
    }

    #[test]
    fn test_epic_boon() {
        let json = json!({
            "name": "Boon of Combat Prowess",
            "source": "DMG",
            "type": "Boon",
            "entries": ["When you miss with a melee weapon attack..."]
        });
        let reward: Reward = serde_json::from_value(json).unwrap();
        assert_eq!(reward.reward_type, Some("Boon".to_string()));
    }

    #[test]
    fn test_charm() {
        let json = json!({
            "name": "Charm of Animal Conjuring",
            "source": "DMG",
            "type": "Charm",
            "duration": "10 days",
            "entries": ["This charm allows you to cast conjure animals..."]
        });
        let reward: Reward = serde_json::from_value(json).unwrap();
        assert_eq!(reward.reward_type, Some("Charm".to_string()));
        assert_eq!(reward.duration, Some("10 days".to_string()));
    }

    #[test]
    fn test_reward_with_prerequisites() {
        let json = json!({
            "name": "Boon of Spell Recall",
            "source": "DMG",
            "type": "Boon",
            "prerequisite": [{"spellcasting": true}],
            "entries": ["You can cast any spell you know..."]
        });
        let reward: Reward = serde_json::from_value(json).unwrap();
        assert!(reward.prerequisite.is_some());
    }

    #[test]
    fn test_reward_data() {
        let json = json!({
            "reward": [
                {"name": "Blessing of Health", "source": "DMG"},
                {"name": "Boon of Combat Prowess", "source": "DMG"}
            ]
        });
        let data: RewardData = serde_json::from_value(json).unwrap();
        assert_eq!(data.reward.len(), 2);
    }

    #[test]
    fn test_reward_type_name() {
        assert_eq!(get_reward_type_name(Some("Blessing")), "Blessing");
        assert_eq!(get_reward_type_name(Some("Boon")), "Epic Boon");
        assert_eq!(get_reward_type_name(None), "Reward");
    }
}
