//! The bodies of the DM's writes on campaign content (MIMIR-T-0720):
//! campaigns, documents, modules, module monsters, maps. A patch leaves an
//! absent field as it is; `null` clears a field that can be empty.

use serde::{Deserialize, Serialize};

use crate::patch::double;

/// The module types: (key, name). The key picks the module's starting
/// documents.
pub const MODULE_TYPES: [(&str, &str); 6] = [
    ("general", "General"),
    ("mystery", "Mystery"),
    ("dungeon", "Dungeon Crawl"),
    ("heist", "Heist"),
    ("horror", "Horror"),
    ("political", "Political Intrigue"),
];

/// The lighting modes of a map: (key, name).
pub const LIGHTING_MODES: [(&str, &str); 3] =
    [("bright", "Bright"), ("dim", "Dim"), ("dark", "Dark")];

/// `POST /campaigns`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewCampaign {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// `PATCH /campaigns/{id}`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CampaignPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<Option<String>>,
}

/// `POST /campaigns/{id}/documents`, `POST /modules/{id}/documents`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewDocument {
    pub title: String,
    #[serde(default)]
    pub content: Option<String>,
    /// Default `user_document`.
    #[serde(default)]
    pub doc_type: Option<String>,
}

/// `PATCH /documents/{id}`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DocumentPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Markdown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// `POST /documents/{id}/move`, `POST /modules/{id}/move`: one place
/// up or down in its list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub direction: Direction,
}

/// `POST /campaigns/{id}/modules`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewModule {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// A key of [`MODULE_TYPES`]; default `general`.
    #[serde(default)]
    pub module_type: Option<String>,
}

/// `PATCH /modules/{id}`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModulePatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<Option<String>>,
}

/// `POST /modules/{id}/monsters`: a catalog monster (name and source) or
/// a homebrew monster (id).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewModuleMonster {
    #[serde(default)]
    pub monster_name: Option<String>,
    #[serde(default)]
    pub monster_source: Option<String>,
    #[serde(default)]
    pub homebrew_monster_id: Option<String>,
    /// Default 1.
    #[serde(default)]
    pub quantity: Option<i32>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// `PATCH /module-monsters/{id}`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModuleMonsterPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// `PATCH /maps/{id}`. (A new map is a multipart upload: the fields
/// `name`, `description`, `module_id`, `lighting_mode` and the `file`.)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<Option<String>>,
    /// A key of [`LIGHTING_MODES`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lighting_mode: Option<String>,
    /// Move the map: a module id, or `null` for the campaign level.
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub module_id: Option<Option<String>>,
    /// Show the whole map to the players (no fog, no sight limits).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reveal_all: Option<bool>,
    /// Walls and closed doors block sight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_of_sight: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_patch_tells_absent_from_null() {
        let p: CampaignPatch = serde_json::from_str(r#"{"name":"A"}"#).unwrap();
        assert_eq!(p.description, None);
        let p: CampaignPatch = serde_json::from_str(r#"{"description":null}"#).unwrap();
        assert_eq!(p.description, Some(None));
        let p: MapPatch = serde_json::from_str(r#"{"module_id":"m1"}"#).unwrap();
        assert_eq!(p.module_id, Some(Some("m1".into())));
    }

    #[test]
    fn directions_are_lowercase() {
        let m: Move = serde_json::from_str(r#"{"direction":"up"}"#).unwrap();
        assert_eq!(m.direction, Direction::Up);
    }
}
