//! Action catalog extraction types
//!
//! Types for deserializing 5etools action JSON data.

use super::types::SrdValue;
use serde::{Deserialize, Serialize};

/// A D&D 5e action from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Time to perform the action
    #[serde(default)]
    pub time: Vec<ActionTime>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    /// Related actions
    #[serde(default)]
    pub see_also_action: Option<Vec<String>>,

    // SRD status
    #[serde(default)]
    pub srd: Option<SrdValue>,
    #[serde(default)]
    pub basic_rules: Option<bool>,
}

/// Action time specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionTime {
    /// Structured time (e.g., 1 action, 1 bonus action)
    Structured {
        number: u32,
        unit: String,
        #[serde(default)]
        condition: Option<String>,
    },
    /// Simple string (e.g., "Varies", "Free")
    Simple(String),
}

/// Container for action data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionData {
    #[serde(default)]
    pub action: Vec<Action>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_action_structured_time() {
        let json = json!({
            "name": "Attack",
            "source": "PHB",
            "time": [{"number": 1, "unit": "action"}],
            "entries": ["The most common action to take in combat is the Attack action."],
            "srd": true
        });
        let action: Action = serde_json::from_value(json).unwrap();
        assert_eq!(action.name, "Attack");
        assert_eq!(action.time.len(), 1);
    }

    #[test]
    fn test_action_reaction() {
        let json = json!({
            "name": "Opportunity Attack",
            "source": "PHB",
            "time": [{"number": 1, "unit": "reaction", "condition": "when a creature leaves your reach"}],
            "entries": ["You can make an opportunity attack..."]
        });
        let action: Action = serde_json::from_value(json).unwrap();
        if let ActionTime::Structured { unit, condition, .. } = &action.time[0] {
            assert_eq!(unit, "reaction");
            assert!(condition.is_some());
        } else {
            panic!("Expected Structured variant");
        }
    }

    #[test]
    fn test_action_with_see_also() {
        let json = json!({
            "name": "Dash",
            "source": "PHB",
            "time": [{"number": 1, "unit": "action"}],
            "entries": ["When you take the Dash action..."],
            "seeAlsoAction": ["Disengage", "Dodge"]
        });
        let action: Action = serde_json::from_value(json).unwrap();
        assert_eq!(action.see_also_action.unwrap().len(), 2);
    }

    #[test]
    fn test_action_data() {
        let json = json!({
            "action": [
                {"name": "Attack", "source": "PHB"},
                {"name": "Dash", "source": "PHB"}
            ]
        });
        let data: ActionData = serde_json::from_value(json).unwrap();
        assert_eq!(data.action.len(), 2);
    }
}
