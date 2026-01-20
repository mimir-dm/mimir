//! Deity catalog extraction types
//!
//! Types for deserializing 5etools deity JSON data.

use serde::{Deserialize, Serialize};

/// A D&D 5e deity from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deity {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Deity's title or epithet
    #[serde(default)]
    pub title: Option<String>,

    /// Pantheon the deity belongs to
    #[serde(default)]
    pub pantheon: Option<String>,

    /// Alignment codes (L, N, C, G, E, U, A)
    #[serde(default)]
    pub alignment: Option<Vec<String>>,

    /// Divine domains
    #[serde(default)]
    pub domains: Option<Vec<String>>,

    /// Holy symbol description
    #[serde(default)]
    pub symbol: Option<String>,

    /// Category within pantheon
    #[serde(default)]
    pub category: Option<String>,

    /// Additional source references
    #[serde(default)]
    pub additional_sources: Option<Vec<SourceReference>>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    // Flags
    #[serde(default)]
    pub has_fluff: Option<bool>,
    #[serde(default)]
    pub has_fluff_images: Option<bool>,
}

/// Source reference for additional sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceReference {
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,
}

/// Container for deity data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeityData {
    #[serde(default)]
    pub deity: Vec<Deity>,
}

/// Deity fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeityFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub images: Option<Vec<serde_json::Value>>,
}

/// Container for deity fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeityFluffData {
    #[serde(default)]
    pub deity_fluff: Vec<DeityFluff>,
}

/// Get human-readable alignment from codes.
pub fn format_alignment(codes: &[String]) -> String {
    codes
        .iter()
        .map(|a| match a.as_str() {
            "L" => "Lawful",
            "N" => "Neutral",
            "C" => "Chaotic",
            "G" => "Good",
            "E" => "Evil",
            "U" => "Unaligned",
            "A" => "Any",
            _ => a.as_str(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deity() {
        let json = json!({
            "name": "Tyr",
            "source": "PHB",
            "title": "God of Justice",
            "pantheon": "Faerûnian",
            "alignment": ["L", "G"],
            "domains": ["Order", "War"],
            "symbol": "Balanced scales resting on a warhammer"
        });
        let deity: Deity = serde_json::from_value(json).unwrap();
        assert_eq!(deity.name, "Tyr");
        assert_eq!(deity.title, Some("God of Justice".to_string()));
        assert_eq!(deity.alignment, Some(vec!["L".to_string(), "G".to_string()]));
    }

    #[test]
    fn test_deity_with_entries() {
        let json = json!({
            "name": "Moradin",
            "source": "PHB",
            "pantheon": "Dwarven",
            "entries": ["Moradin, the Soul Forger, is the chief deity of the dwarven pantheon."]
        });
        let deity: Deity = serde_json::from_value(json).unwrap();
        assert_eq!(deity.name, "Moradin");
        assert_eq!(deity.entries.len(), 1);
    }

    #[test]
    fn test_deity_data() {
        let json = json!({
            "deity": [
                {"name": "Tyr", "source": "PHB"},
                {"name": "Moradin", "source": "PHB"}
            ]
        });
        let data: DeityData = serde_json::from_value(json).unwrap();
        assert_eq!(data.deity.len(), 2);
    }

    #[test]
    fn test_format_alignment() {
        let codes = vec!["L".to_string(), "G".to_string()];
        assert_eq!(format_alignment(&codes), "Lawful Good");

        let codes = vec!["C".to_string(), "E".to_string()];
        assert_eq!(format_alignment(&codes), "Chaotic Evil");

        let codes = vec!["N".to_string()];
        assert_eq!(format_alignment(&codes), "Neutral");
    }
}
