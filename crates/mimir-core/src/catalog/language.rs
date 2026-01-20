//! Language catalog extraction types
//!
//! Types for deserializing 5etools language JSON data.

use super::types::SrdValue;
use serde::{Deserialize, Serialize};

/// A D&D 5e language from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Language type: standard, exotic, secret, etc.
    #[serde(rename = "type", default)]
    pub language_type: Option<String>,

    /// Script used
    #[serde(default)]
    pub script: Option<String>,

    /// Typical speakers (may contain 5etools tags)
    #[serde(default)]
    pub typical_speakers: Option<Vec<String>>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    /// Available fonts
    #[serde(default)]
    pub fonts: Option<Vec<String>>,

    /// Dialects of this language
    #[serde(default)]
    pub dialects: Option<Vec<String>>,

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

/// Container for language data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageData {
    #[serde(default)]
    pub language: Vec<Language>,
}

/// Language fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub images: Option<Vec<serde_json::Value>>,
}

/// Container for language fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageFluffData {
    #[serde(default)]
    pub language_fluff: Vec<LanguageFluff>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_standard_language() {
        let json = json!({
            "name": "Common",
            "source": "PHB",
            "type": "standard",
            "script": "Common",
            "typicalSpeakers": ["Humans"],
            "srd": true
        });
        let lang: Language = serde_json::from_value(json).unwrap();
        assert_eq!(lang.name, "Common");
        assert_eq!(lang.language_type, Some("standard".to_string()));
        assert_eq!(lang.script, Some("Common".to_string()));
    }

    #[test]
    fn test_exotic_language() {
        let json = json!({
            "name": "Draconic",
            "source": "PHB",
            "type": "exotic",
            "script": "Draconic",
            "typicalSpeakers": ["Dragons", "Dragonborn"]
        });
        let lang: Language = serde_json::from_value(json).unwrap();
        assert_eq!(lang.language_type, Some("exotic".to_string()));
        assert_eq!(lang.typical_speakers.unwrap().len(), 2);
    }

    #[test]
    fn test_secret_language() {
        let json = json!({
            "name": "Druidic",
            "source": "PHB",
            "type": "secret",
            "typicalSpeakers": ["Druids"]
        });
        let lang: Language = serde_json::from_value(json).unwrap();
        assert_eq!(lang.language_type, Some("secret".to_string()));
    }

    #[test]
    fn test_language_data() {
        let json = json!({
            "language": [
                {"name": "Common", "source": "PHB"},
                {"name": "Elvish", "source": "PHB"}
            ]
        });
        let data: LanguageData = serde_json::from_value(json).unwrap();
        assert_eq!(data.language.len(), 2);
    }
}
