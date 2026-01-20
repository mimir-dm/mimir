//! Cult and Boon catalog extraction types
//!
//! Types for deserializing 5etools cult and demonic boon JSON data.

use serde::{Deserialize, Serialize};

/// Cultist information entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultistInfo {
    pub entry: String,
}

/// Goal information entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalInfo {
    pub entry: String,
}

/// Signature spell information entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureSpellInfo {
    pub entry: String,
}

/// Ability information entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityInfo {
    pub entry: String,
}

/// A D&D 5e cult from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cult {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Cult type: Diabolical, Demonic, Elder Evil, etc.
    #[serde(rename = "type", default)]
    pub cult_type: Option<String>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    /// Cultist information
    #[serde(default)]
    pub cultists: Option<CultistInfo>,

    /// Goal information
    #[serde(default)]
    pub goal: Option<GoalInfo>,

    /// Signature spells
    #[serde(default)]
    pub signature_spells: Option<SignatureSpellInfo>,

    /// Other source references
    #[serde(default)]
    pub other_sources: Option<Vec<serde_json::Value>>,
}

/// A D&D 5e demonic boon from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Boon {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Boon type
    #[serde(rename = "type", default)]
    pub boon_type: Option<String>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    /// Ability information
    #[serde(default)]
    pub ability: Option<AbilityInfo>,

    /// Signature spells
    #[serde(default)]
    pub signature_spells: Option<SignatureSpellInfo>,

    /// Reprinted as reference
    #[serde(default)]
    pub reprinted_as: Option<Vec<serde_json::Value>>,
}

/// Container for cult data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultData {
    #[serde(default)]
    pub cult: Vec<Cult>,
}

/// Container for boon data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoonData {
    #[serde(default)]
    pub boon: Vec<Boon>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cult() {
        let json = json!({
            "name": "Cult of Asmodeus",
            "source": "MTF",
            "type": "Diabolical",
            "entries": ["Asmodeus's followers crave power..."],
            "cultists": {"entry": "Lawyers, bureaucrats, and politicians"},
            "goal": {"entry": "Domination through contracts and corruption"}
        });
        let cult: Cult = serde_json::from_value(json).unwrap();
        assert_eq!(cult.name, "Cult of Asmodeus");
        assert_eq!(cult.cult_type, Some("Diabolical".to_string()));
        assert!(cult.cultists.is_some());
        assert!(cult.goal.is_some());
    }

    #[test]
    fn test_cult_with_signature_spells() {
        let json = json!({
            "name": "Cult of Demogorgon",
            "source": "MTF",
            "type": "Demonic",
            "signatureSpells": {"entry": "confusion, hunger of Hadar"}
        });
        let cult: Cult = serde_json::from_value(json).unwrap();
        assert_eq!(cult.cult_type, Some("Demonic".to_string()));
        assert!(cult.signature_spells.is_some());
    }

    #[test]
    fn test_boon() {
        let json = json!({
            "name": "Boon of Demogorgon",
            "source": "MTF",
            "type": "Demonic",
            "entries": ["Those who receive the boon of Demogorgon..."],
            "ability": {"entry": "You gain telepathy out to 120 feet."}
        });
        let boon: Boon = serde_json::from_value(json).unwrap();
        assert_eq!(boon.name, "Boon of Demogorgon");
        assert!(boon.ability.is_some());
    }

    #[test]
    fn test_cult_data() {
        let json = json!({
            "cult": [
                {"name": "Cult of Asmodeus", "source": "MTF"},
                {"name": "Cult of Baphomet", "source": "MTF"}
            ]
        });
        let data: CultData = serde_json::from_value(json).unwrap();
        assert_eq!(data.cult.len(), 2);
    }

    #[test]
    fn test_boon_data() {
        let json = json!({
            "boon": [
                {"name": "Boon of Demogorgon", "source": "MTF"},
                {"name": "Boon of Orcus", "source": "MTF"}
            ]
        });
        let data: BoonData = serde_json::from_value(json).unwrap();
        assert_eq!(data.boon.len(), 2);
    }
}
