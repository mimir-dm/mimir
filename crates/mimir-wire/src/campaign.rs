//! Campaign content, read side (MIMIR-T-0708): what the campaign list, the
//! dashboard and the module view show. Lists return summaries; a document
//! is returned whole (with its markdown) by itself.

use serde::{Deserialize, Serialize};

/// A campaign.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CampaignSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// When the campaign was archived; `None` for an active campaign.
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A module of a campaign.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleSummary {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub description: Option<String>,
    /// The order of the module in the campaign (1, 2, …).
    pub module_number: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// A document in a list: no content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub id: String,
    pub campaign_id: String,
    /// `None` for a campaign-level document.
    pub module_id: Option<String>,
    pub title: String,
    /// Free text: "note", "session", "npc", "location", …
    pub doc_type: String,
    pub sort_order: i32,
    pub updated_at: String,
}

/// A whole document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub campaign_id: String,
    pub module_id: Option<String>,
    pub title: String,
    pub doc_type: String,
    pub sort_order: i32,
    /// Markdown.
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// One class of a character.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassLevel {
    pub class_name: String,
    pub subclass_name: Option<String>,
    pub level: i32,
}

/// A campaign character (PC or NPC) in a list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterSummary {
    pub id: String,
    pub campaign_id: Option<String>,
    pub name: String,
    pub is_npc: bool,
    /// The player, for a PC.
    pub player_name: Option<String>,
    pub race_name: Option<String>,
    pub background_name: Option<String>,
    pub classes: Vec<ClassLevel>,
    /// The sum of the class levels.
    pub level: i32,
    /// NPC fields.
    pub role: Option<String>,
    pub location: Option<String>,
    pub faction: Option<String>,
    pub updated_at: String,
}

/// An NPC of a module (not a campaign character).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleNpcSummary {
    pub id: String,
    pub module_id: String,
    pub name: String,
    pub role: Option<String>,
    pub description: Option<String>,
}

/// A monster group of a module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleMonsterSummary {
    pub id: String,
    pub module_id: String,
    /// The name to show: the display name, else the monster's name.
    pub name: String,
    /// The catalog monster; `None` for homebrew.
    pub monster_name: Option<String>,
    pub monster_source: Option<String>,
    /// The homebrew monster; `None` for a catalog monster.
    pub homebrew_monster_id: Option<String>,
    pub quantity: i32,
    pub notes: Option<String>,
}

/// A map in a list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapSummary {
    pub id: String,
    pub campaign_id: String,
    /// `None` for a campaign-level map.
    pub module_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    /// "bright", "dim" or "dark".
    pub lighting_mode: String,
    pub fog_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_summary_round_trips() {
        let c = CharacterSummary {
            id: "pc1".into(),
            campaign_id: Some("c1".into()),
            name: "Robin".into(),
            is_npc: false,
            player_name: Some("Sam".into()),
            race_name: Some("Elf".into()),
            background_name: None,
            classes: vec![ClassLevel {
                class_name: "Wizard".into(),
                subclass_name: None,
                level: 3,
            }],
            level: 3,
            role: None,
            location: None,
            faction: None,
            updated_at: "2026-10-10T00:00:00Z".into(),
        };
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["is_npc"], false);
        assert_eq!(json["classes"][0]["class_name"], "Wizard");
        assert_eq!(serde_json::from_value::<CharacterSummary>(json).unwrap(), c);
    }

    #[test]
    fn map_summary_has_a_boolean_fog_flag() {
        let m = MapSummary {
            id: "m1".into(),
            campaign_id: "c1".into(),
            module_id: None,
            name: "Cave".into(),
            description: None,
            sort_order: 1,
            lighting_mode: "dim".into(),
            fog_enabled: true,
        };
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains(r#""fog_enabled":true"#));
        assert_eq!(serde_json::from_str::<MapSummary>(&json).unwrap(), m);
    }
}
