//! Condition and Disease catalog extraction types
//!
//! Types for deserializing 5etools condition and disease JSON data.

use super::types::SrdValue;
use serde::{Deserialize, Serialize};

/// A D&D 5e condition from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    // Flags
    #[serde(default)]
    pub basic_rules: Option<bool>,
    #[serde(default)]
    pub has_fluff_images: Option<bool>,

    // Other sources
    #[serde(default)]
    pub other_sources: Option<Vec<serde_json::Value>>,

    // SRD status
    #[serde(default)]
    pub srd: Option<SrdValue>,
}

/// A D&D 5e disease from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Disease {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    // SRD status
    #[serde(default)]
    pub srd: Option<SrdValue>,
}

/// Container for condition data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionData {
    #[serde(default)]
    pub condition: Vec<Condition>,
}

/// Container for disease data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseData {
    #[serde(default)]
    pub disease: Vec<Disease>,
}

/// Condition fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub images: Option<Vec<serde_json::Value>>,
}

/// Container for condition fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionFluffData {
    #[serde(default)]
    pub condition_fluff: Vec<ConditionFluff>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_condition() {
        let json = json!({
            "name": "Blinded",
            "source": "PHB",
            "page": 290,
            "entries": [
                "A blinded creature can't see and automatically fails any ability check that requires sight.",
                "Attack rolls against the creature have advantage."
            ],
            "srd": true
        });
        let condition: Condition = serde_json::from_value(json).unwrap();
        assert_eq!(condition.name, "Blinded");
        assert_eq!(condition.entries.len(), 2);
        assert!(matches!(condition.srd, Some(SrdValue::Flag(true))));
    }

    #[test]
    fn test_disease() {
        let json = json!({
            "name": "Cackle Fever",
            "source": "DMG",
            "page": 257,
            "entries": ["This disease targets humanoids..."]
        });
        let disease: Disease = serde_json::from_value(json).unwrap();
        assert_eq!(disease.name, "Cackle Fever");
    }

    #[test]
    fn test_condition_data() {
        let json = json!({
            "condition": [
                {"name": "Blinded", "source": "PHB"},
                {"name": "Charmed", "source": "PHB"}
            ]
        });
        let data: ConditionData = serde_json::from_value(json).unwrap();
        assert_eq!(data.condition.len(), 2);
    }

    #[test]
    fn test_disease_data() {
        let json = json!({
            "disease": [
                {"name": "Cackle Fever", "source": "DMG"},
                {"name": "Sewer Plague", "source": "DMG"}
            ]
        });
        let data: DiseaseData = serde_json::from_value(json).unwrap();
        assert_eq!(data.disease.len(), 2);
    }
}
