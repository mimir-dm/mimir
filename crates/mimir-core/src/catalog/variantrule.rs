//! Variant Rule catalog extraction types
//!
//! Types for deserializing 5etools variant rule JSON data.

use super::types::SrdValue;
use serde::{Deserialize, Serialize};

/// A D&D 5e variant rule from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariantRule {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Rule type categorization
    #[serde(default)]
    pub rule_type: Option<String>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    /// Search index for the rule
    #[serde(default)]
    pub search: Option<String>,

    // Flags
    #[serde(default)]
    pub basic_rules: Option<bool>,

    // SRD status
    #[serde(default)]
    pub srd: Option<SrdValue>,
}

/// Container for variant rule data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantRuleData {
    #[serde(default)]
    pub variantrule: Vec<VariantRule>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_variant_rule() {
        let json = json!({
            "name": "Flanking",
            "source": "DMG",
            "page": 251,
            "ruleType": "O",
            "entries": ["Optional rule for gaining advantage when flanking..."]
        });
        let rule: VariantRule = serde_json::from_value(json).unwrap();
        assert_eq!(rule.name, "Flanking");
        assert_eq!(rule.source, "DMG");
        assert_eq!(rule.rule_type, Some("O".to_string()));
    }

    #[test]
    fn test_variant_rule_with_srd() {
        let json = json!({
            "name": "Multiclassing",
            "source": "PHB",
            "entries": ["Multiclassing allows you to gain levels in multiple classes."],
            "srd": true
        });
        let rule: VariantRule = serde_json::from_value(json).unwrap();
        assert!(matches!(rule.srd, Some(SrdValue::Flag(true))));
    }

    #[test]
    fn test_variant_rule_data() {
        let json = json!({
            "variantrule": [
                {"name": "Flanking", "source": "DMG"},
                {"name": "Multiclassing", "source": "PHB"}
            ]
        });
        let data: VariantRuleData = serde_json::from_value(json).unwrap();
        assert_eq!(data.variantrule.len(), 2);
    }

    #[test]
    fn test_variant_rule_minimal() {
        let json = json!({
            "name": "Custom Rule",
            "source": "HB"
        });
        let rule: VariantRule = serde_json::from_value(json).unwrap();
        assert_eq!(rule.name, "Custom Rule");
        assert!(rule.entries.is_empty());
        assert!(rule.rule_type.is_none());
    }
}
