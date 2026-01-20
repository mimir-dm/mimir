//! Psionic catalog extraction types
//!
//! Types for deserializing 5etools psionic JSON data (UA psionics).

use serde::{Deserialize, Serialize};

/// Psionic power cost range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsionicCost {
    pub min: i32,
    #[serde(default)]
    pub max: Option<i32>,
}

/// Concentration duration for psionic modes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentrationDuration {
    pub duration: i32,
    pub unit: String, // "min", "hr", etc.
}

/// A psionic mode (for disciplines).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsionicMode {
    pub name: String,
    pub cost: PsionicCost,
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,
    #[serde(default)]
    pub concentration: Option<ConcentrationDuration>,
}

/// A D&D 5e psionic power from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Psionic {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Psionic type: D (Discipline) or T (Talent)
    #[serde(rename = "type")]
    pub psionic_type: String,

    /// Psionic order: Avatar, Awakened, Immortal, Nomad, Wu Jen
    #[serde(default)]
    pub order: Option<String>,

    /// Focus benefit for disciplines
    #[serde(default)]
    pub focus: Option<String>,

    /// Modes for disciplines
    #[serde(default)]
    pub modes: Option<Vec<PsionicMode>>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,
}

/// Container for psionic data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsionicData {
    #[serde(default)]
    pub psionic: Vec<Psionic>,
}

/// Get human-readable name for psionic type code.
pub fn get_psionic_type_name(code: &str) -> &'static str {
    match code {
        "D" => "Discipline",
        "T" => "Talent",
        _ => "Unknown",
    }
}

/// Get human-readable name for psionic order.
pub fn get_order_name(order: &str) -> &'static str {
    match order {
        "Avatar" => "Avatar",
        "Awakened" => "Awakened",
        "Immortal" => "Immortal",
        "Nomad" => "Nomad",
        "Wu Jen" => "Wu Jen",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_talent() {
        let json = json!({
            "name": "Beacon",
            "source": "UATheMysticClass",
            "type": "T",
            "entries": ["As a bonus action, you cause bright light to radiate from your body..."]
        });
        let psionic: Psionic = serde_json::from_value(json).unwrap();
        assert_eq!(psionic.name, "Beacon");
        assert_eq!(psionic.psionic_type, "T");
    }

    #[test]
    fn test_discipline() {
        let json = json!({
            "name": "Mastery of Air",
            "source": "UATheMysticClass",
            "type": "D",
            "order": "Wu Jen",
            "focus": "While focused on this discipline, you take no falling damage.",
            "modes": [
                {
                    "name": "Wind Step",
                    "cost": {"min": 1, "max": 7},
                    "entries": ["As part of your move on a turn..."]
                }
            ]
        });
        let psionic: Psionic = serde_json::from_value(json).unwrap();
        assert_eq!(psionic.psionic_type, "D");
        assert_eq!(psionic.order, Some("Wu Jen".to_string()));
        assert!(psionic.focus.is_some());
        let modes = psionic.modes.unwrap();
        assert_eq!(modes.len(), 1);
        assert_eq!(modes[0].cost.min, 1);
        assert_eq!(modes[0].cost.max, Some(7));
    }

    #[test]
    fn test_discipline_with_concentration() {
        let json = json!({
            "name": "Intellect Fortress",
            "source": "UATheMysticClass",
            "type": "D",
            "order": "Awakened",
            "modes": [
                {
                    "name": "Psychic Backlash",
                    "cost": {"min": 2},
                    "concentration": {"duration": 1, "unit": "hr"},
                    "entries": ["As a reaction..."]
                }
            ]
        });
        let psionic: Psionic = serde_json::from_value(json).unwrap();
        let modes = psionic.modes.unwrap();
        let concentration = modes[0].concentration.as_ref().unwrap();
        assert_eq!(concentration.duration, 1);
        assert_eq!(concentration.unit, "hr");
    }

    #[test]
    fn test_psionic_data() {
        let json = json!({
            "psionic": [
                {"name": "Beacon", "source": "UATheMysticClass", "type": "T"},
                {"name": "Mastery of Air", "source": "UATheMysticClass", "type": "D"}
            ]
        });
        let data: PsionicData = serde_json::from_value(json).unwrap();
        assert_eq!(data.psionic.len(), 2);
    }

    #[test]
    fn test_psionic_type_name() {
        assert_eq!(get_psionic_type_name("D"), "Discipline");
        assert_eq!(get_psionic_type_name("T"), "Talent");
    }

    #[test]
    fn test_order_name() {
        assert_eq!(get_order_name("Wu Jen"), "Wu Jen");
        assert_eq!(get_order_name("Avatar"), "Avatar");
    }
}
