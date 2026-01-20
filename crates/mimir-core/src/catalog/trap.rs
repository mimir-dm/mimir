//! Trap and Hazard catalog extraction types
//!
//! Types for deserializing 5etools trap and hazard JSON data.

use super::types::SrdValue;
use serde::{Deserialize, Serialize};

/// A D&D 5e trap from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trap {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Trap/hazard type: MECH, MAG, WLD, WTH, ENV
    #[serde(default)]
    pub trap_haz_type: Option<String>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    // Flags
    #[serde(default)]
    pub has_fluff: Option<bool>,
    #[serde(default)]
    pub has_fluff_images: Option<bool>,

    // SRD status
    #[serde(default)]
    pub srd: Option<SrdValue>,
}

/// A D&D 5e hazard from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hazard {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Trap/hazard type: MECH, MAG, WLD, WTH, ENV
    #[serde(default)]
    pub trap_haz_type: Option<String>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    // Flags
    #[serde(default)]
    pub has_fluff: Option<bool>,
    #[serde(default)]
    pub has_fluff_images: Option<bool>,

    // SRD status
    #[serde(default)]
    pub srd: Option<SrdValue>,
}

/// Container for trap data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapData {
    #[serde(default)]
    pub trap: Vec<Trap>,
}

/// Container for hazard data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HazardData {
    #[serde(default)]
    pub hazard: Vec<Hazard>,
}

/// Get human-readable name for trap/hazard type code.
pub fn get_trap_type_name(code: &str) -> &'static str {
    match code {
        "MECH" => "Mechanical",
        "MAG" => "Magical",
        "WLD" => "Wilderness",
        "WTH" => "Weather",
        "ENV" => "Environmental",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mechanical_trap() {
        let json = json!({
            "name": "Pit Trap",
            "source": "DMG",
            "trapHazType": "MECH",
            "entries": ["A pit trap is a simple trap..."]
        });
        let trap: Trap = serde_json::from_value(json).unwrap();
        assert_eq!(trap.name, "Pit Trap");
        assert_eq!(trap.trap_haz_type, Some("MECH".to_string()));
    }

    #[test]
    fn test_magical_trap() {
        let json = json!({
            "name": "Glyph of Warding",
            "source": "PHB",
            "trapHazType": "MAG",
            "entries": ["An almost invisible glyph..."]
        });
        let trap: Trap = serde_json::from_value(json).unwrap();
        assert_eq!(trap.trap_haz_type, Some("MAG".to_string()));
    }

    #[test]
    fn test_hazard() {
        let json = json!({
            "name": "Brown Mold",
            "source": "DMG",
            "trapHazType": "ENV",
            "entries": ["Brown mold feeds on warmth..."]
        });
        let hazard: Hazard = serde_json::from_value(json).unwrap();
        assert_eq!(hazard.name, "Brown Mold");
        assert_eq!(hazard.trap_haz_type, Some("ENV".to_string()));
    }

    #[test]
    fn test_trap_data() {
        let json = json!({
            "trap": [
                {"name": "Pit Trap", "source": "DMG"},
                {"name": "Poison Needle", "source": "DMG"}
            ]
        });
        let data: TrapData = serde_json::from_value(json).unwrap();
        assert_eq!(data.trap.len(), 2);
    }

    #[test]
    fn test_hazard_data() {
        let json = json!({
            "hazard": [
                {"name": "Brown Mold", "source": "DMG"},
                {"name": "Green Slime", "source": "DMG"}
            ]
        });
        let data: HazardData = serde_json::from_value(json).unwrap();
        assert_eq!(data.hazard.len(), 2);
    }

    #[test]
    fn test_trap_type_name() {
        assert_eq!(get_trap_type_name("MECH"), "Mechanical");
        assert_eq!(get_trap_type_name("MAG"), "Magical");
        assert_eq!(get_trap_type_name("ENV"), "Environmental");
    }
}
