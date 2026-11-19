//! Player notes and sheet shares (MIMIR-T-0718).

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{player_notes, sheet_shares};

/// A player's note about their encounters (markdown).
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = player_notes)]
pub struct PlayerNote {
    pub id: String,
    pub character_id: String,
    pub module_id: Option<String>,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = player_notes)]
pub struct NewPlayerNote<'a> {
    pub id: &'a str,
    pub character_id: &'a str,
    pub module_id: Option<&'a str>,
    pub title: &'a str,
    pub content: &'a str,
}

#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = player_notes)]
pub struct UpdatePlayerNote {
    pub module_id: Option<Option<String>>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sheet_shares)]
pub struct NewSheetShare<'a> {
    pub character_id: &'a str,
}
