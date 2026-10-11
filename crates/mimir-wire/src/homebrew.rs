//! A campaign's homebrew items, monsters and spells (MIMIR-T-0722). The
//! data is 5etools-shaped JSON; the kinds are "items", "monsters",
//! "spells" (as the catalog's categories).

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const HOMEBREW_KINDS: [(&str, &str); 3] = [
    ("items", "Items"),
    ("monsters", "Monsters"),
    ("spells", "Spells"),
];

/// A row of a kind's list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomebrewSummary {
    pub id: String,
    pub name: String,
    /// A short line: "Very rare wondrous item", "CR 10 aberration",
    /// "3rd-level evocation".
    pub detail: String,
    #[serde(default)]
    pub cloned_from_name: Option<String>,
    #[serde(default)]
    pub cloned_from_source: Option<String>,
    pub updated_at: String,
}

/// One entry: the data, with `name` and `source: "HB"` set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomebrewEntry {
    pub id: String,
    pub name: String,
    pub data: Value,
    #[serde(default)]
    pub cloned_from_name: Option<String>,
    #[serde(default)]
    pub cloned_from_source: Option<String>,
    pub updated_at: String,
}

/// `POST /campaigns/{id}/homebrew/{kind}`: new data, or a copy of a
/// catalog entry (`cloned_from_*`) with `data` merged over it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewHomebrew {
    pub name: String,
    #[serde(default)]
    pub data: Option<Value>,
    #[serde(default)]
    pub cloned_from_name: Option<String>,
    #[serde(default)]
    pub cloned_from_source: Option<String>,
}

/// `PATCH /homebrew/{kind}/{id}`: the data replaces the old.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HomebrewPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// `GET /homebrew/{kind}/{id}/usage`: what uses it (before a delete).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HomebrewUsage {
    pub uses: Vec<String>,
}
