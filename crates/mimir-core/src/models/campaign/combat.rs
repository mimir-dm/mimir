//! Combat tracker models (COLLIERY-I-0468).
//!
//! A `CombatSession` is one fight in a module; its `CombatEntry` rows are the
//! creatures in it. Conditions and the recent-damage log are JSON columns,
//! parsed by `CombatService`.

use crate::schema::{combat_entries, combat_sessions};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// One fight in a module. At most one is `active` per module.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Serialize,
    Deserialize,
)]
#[diesel(table_name = combat_sessions)]
#[diesel(treat_none_as_null = true)]
pub struct CombatSession {
    pub id: String,
    pub module_id: String,
    /// Current round, starting at 1.
    pub round: i32,
    /// Index into the entries in turn order of whose turn it is.
    pub turn_index: i32,
    /// "active" or "ended".
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl CombatSession {
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
}

/// Data for inserting a combat session.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = combat_sessions)]
pub struct NewCombatSession<'a> {
    pub id: &'a str,
    pub module_id: &'a str,
}

/// One creature in a fight.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Serialize,
    Deserialize,
)]
#[diesel(table_name = combat_entries)]
#[diesel(treat_none_as_null = true)]
pub struct CombatEntry {
    pub id: String,
    pub session_id: String,
    /// "module_monster", "module_npc", "character" or "custom".
    pub source_kind: String,
    pub source_id: Option<String>,
    /// Map token this entry is, if linked.
    pub token_id: Option<String>,
    pub display_name: String,
    /// None until rolled.
    pub initiative: Option<i32>,
    /// Dexterity modifier for initiative ties.
    pub dex_modifier: Option<i32>,
    pub max_hp: Option<i32>,
    pub current_hp: Option<i32>,
    pub temp_hp: i32,
    pub is_concentrating: i32,
    /// JSON: `[{"name": "...", "expires_round": n|null}]`.
    pub conditions: String,
    /// JSON: last 3 HP changes `[{"kind": "damage"|"heal", "amount": n, "round": n}]`.
    pub damage_log: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Data for inserting a combat entry.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = combat_entries)]
pub struct NewCombatEntry<'a> {
    pub id: &'a str,
    pub session_id: &'a str,
    pub source_kind: &'a str,
    pub source_id: Option<&'a str>,
    pub token_id: Option<&'a str>,
    pub display_name: &'a str,
    pub initiative: Option<i32>,
    pub dex_modifier: Option<i32>,
    pub max_hp: Option<i32>,
    pub current_hp: Option<i32>,
}
