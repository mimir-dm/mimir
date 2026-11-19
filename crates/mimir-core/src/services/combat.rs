//! Combat tracker service (COLLIERY-I-0468, MIMIR-T-0676).
//!
//! All combat rules live here: initiative order, turns and rounds, HP with
//! temporary HP, conditions with durations, and concentration checks. The
//! state is stored per module (one active fight), so a DM can close the DM Map
//! window or restart the app and resume. Live-play state: no MCP tools.

use diesel::{Connection, SqliteConnection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::dal::catalog as catalog_dal;
use crate::models::campaign::{
    Character, CombatEntry, CombatSession, NewCombatEntry, NewCombatSession, TokenPlacement,
};
use crate::services::{ServiceError, ServiceResult};
use crate::utils::now_rfc3339;

/// The 5e SRD conditions a combat entry can carry (stored lowercase).
pub const SRD_CONDITIONS: &[&str] = &[
    "blinded",
    "charmed",
    "deafened",
    "exhaustion",
    "frightened",
    "grappled",
    "incapacitated",
    "invisible",
    "paralyzed",
    "petrified",
    "poisoned",
    "prone",
    "restrained",
    "stunned",
    "unconscious",
];

/// How many HP changes an entry remembers.
const DAMAGE_LOG_LEN: usize = 3;

/// A condition on a combat entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveCondition {
    pub name: String,
    /// Last round the condition applies; `None` lasts until removed.
    pub expires_round: Option<i32>,
}

/// One HP change in an entry's recent history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HpChange {
    /// "damage" or "heal".
    pub kind: String,
    pub amount: i32,
    pub round: i32,
}

/// A combat entry with its JSON columns parsed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CombatEntryView {
    pub id: String,
    pub source_kind: String,
    pub source_id: Option<String>,
    pub token_id: Option<String>,
    pub display_name: String,
    pub initiative: Option<i32>,
    pub dex_modifier: Option<i32>,
    pub max_hp: Option<i32>,
    pub current_hp: Option<i32>,
    pub temp_hp: i32,
    pub is_concentrating: bool,
    pub conditions: Vec<ActiveCondition>,
    pub damage_log: Vec<HpChange>,
}

impl From<CombatEntry> for CombatEntryView {
    fn from(e: CombatEntry) -> Self {
        Self {
            conditions: serde_json::from_str(&e.conditions).unwrap_or_default(),
            damage_log: serde_json::from_str(&e.damage_log).unwrap_or_default(),
            is_concentrating: e.is_concentrating != 0,
            id: e.id,
            source_kind: e.source_kind,
            source_id: e.source_id,
            token_id: e.token_id,
            display_name: e.display_name,
            initiative: e.initiative,
            dex_modifier: e.dex_modifier,
            max_hp: e.max_hp,
            current_hp: e.current_hp,
            temp_hp: e.temp_hp,
        }
    }
}

/// A fight: the session, its entries in turn order, and whose turn it is.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CombatState {
    pub session: CombatSession,
    pub entries: Vec<CombatEntryView>,
    pub current_entry_id: Option<String>,
}

/// The entry after damage, and the concentration save DC if it concentrates.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DamageResult {
    pub entry: CombatEntryView,
    pub concentration_dc: Option<i32>,
}

/// Turn order: initiative high to low, then DEX modifier, then name; entries
/// without initiative (or DEX) come last.
fn sort_turn_order(entries: &mut [CombatEntry]) {
    entries.sort_by(|a, b| {
        b.initiative
            .cmp(&a.initiative)
            .then(b.dex_modifier.cmp(&a.dex_modifier))
            .then(a.display_name.cmp(&b.display_name))
    });
}

/// HP (`hp.average`) and DEX modifier from a 5etools-style stat block.
fn stat_block_hp_and_dex(data: Option<&str>) -> (Option<i32>, Option<i32>) {
    let Some(v) = data.and_then(|d| serde_json::from_str::<Value>(d).ok()) else {
        return (None, None);
    };
    let hp = v
        .get("hp")
        .and_then(|h| h.get("average"))
        .and_then(Value::as_i64)
        .map(|n| n as i32);
    let dex = v
        .get("dex")
        .and_then(Value::as_i64)
        .map(|d| Character::ability_modifier(d as i32));
    (hp, dex)
}

/// Combat tracker operations.
pub struct CombatService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> CombatService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    // --- Sessions --------------------------------------------------------

    /// The module's active fight, if any.
    pub fn active(&mut self, module_id: &str) -> ServiceResult<Option<CombatState>> {
        match dal::get_active_combat_session(self.conn, module_id)? {
            Some(session) => Ok(Some(self.state(&session.id)?)),
            None => Ok(None),
        }
    }

    /// Resume the module's active fight, or start one.
    pub fn start(&mut self, module_id: &str) -> ServiceResult<CombatState> {
        if let Some(state) = self.active(module_id)? {
            return Ok(state);
        }
        if dal::get_module_optional(self.conn, module_id)?.is_none() {
            return Err(ServiceError::not_found("Module", module_id));
        }
        let id = Uuid::new_v4().to_string();
        dal::insert_combat_session(self.conn, &NewCombatSession { id: &id, module_id })?;
        self.state(&id)
    }

    /// End a fight. Its entries are kept with the ended session.
    pub fn end(&mut self, session_id: &str) -> ServiceResult<()> {
        let mut session = self.active_session(session_id)?;
        session.status = "ended".to_string();
        session.updated_at = now_rfc3339();
        dal::save_combat_session(self.conn, &session)?;
        Ok(())
    }

    /// The fight with its entries in turn order.
    pub fn state(&mut self, session_id: &str) -> ServiceResult<CombatState> {
        let session = dal::get_combat_session_optional(self.conn, session_id)?
            .ok_or_else(|| ServiceError::not_found("CombatSession", session_id))?;
        let entries = self.ordered_entries(session_id)?;
        let current_entry_id = entries
            .get(session.turn_index.max(0) as usize)
            .map(|e| e.id.clone());
        Ok(CombatState {
            session,
            entries: entries.into_iter().map(CombatEntryView::from).collect(),
            current_entry_id,
        })
    }

    /// Advance to the next turn; after the last entry, start the next round
    /// and drop conditions that have run out.
    pub fn next_turn(&mut self, session_id: &str) -> ServiceResult<CombatState> {
        let mut session = self.active_session(session_id)?;
        let mut entries = self.ordered_entries(session_id)?;
        if entries.is_empty() {
            return self.state(session_id);
        }
        self.conn.transaction::<_, ServiceError, _>(|conn| {
            session.turn_index += 1;
            if session.turn_index as usize >= entries.len() {
                session.turn_index = 0;
                session.round += 1;
                let now = now_rfc3339();
                for entry in entries.iter_mut() {
                    let mut conditions: Vec<ActiveCondition> =
                        serde_json::from_str(&entry.conditions).unwrap_or_default();
                    let before = conditions.len();
                    conditions.retain(|c| c.expires_round.is_none_or(|r| r >= session.round));
                    if conditions.len() != before {
                        entry.conditions = to_json(&conditions);
                        entry.updated_at = now.clone();
                        dal::save_combat_entry(conn, entry)?;
                    }
                }
            }
            session.updated_at = now_rfc3339();
            dal::save_combat_session(conn, &session)?;
            Ok(())
        })?;
        self.state(session_id)
    }

    /// Go back one turn (into the previous round if needed). Expired
    /// conditions do not come back.
    pub fn previous_turn(&mut self, session_id: &str) -> ServiceResult<CombatState> {
        let mut session = self.active_session(session_id)?;
        let count = self.ordered_entries(session_id)?.len() as i32;
        if session.turn_index > 0 {
            session.turn_index -= 1;
        } else if session.round > 1 && count > 0 {
            session.round -= 1;
            session.turn_index = count - 1;
        } else {
            return self.state(session_id);
        }
        session.updated_at = now_rfc3339();
        dal::save_combat_session(self.conn, &session)?;
        self.state(session_id)
    }

    // --- Adding and removing entries ------------------------------------

    /// Add a module monster group: `count` entries (default: the group's
    /// quantity), numbered after any already in the fight. HP and DEX come
    /// from the catalog or homebrew stat block.
    pub fn add_module_monster(
        &mut self,
        session_id: &str,
        module_monster_id: &str,
        count: Option<i32>,
    ) -> ServiceResult<Vec<CombatEntryView>> {
        self.active_session(session_id)?;
        let mm = dal::get_module_monster_optional(self.conn, module_monster_id)?
            .ok_or_else(|| ServiceError::not_found("ModuleMonster", module_monster_id))?;
        let count = count.unwrap_or(mm.quantity);
        if count < 1 {
            return Err(ServiceError::validation("count must be at least 1"));
        }

        let (base_name, data) = match &mm.homebrew_monster_id {
            Some(hb_id) => {
                let hb = dal::get_campaign_homebrew_monster(self.conn, hb_id)?;
                (hb.name, Some(hb.data))
            }
            None => {
                let name = mm.monster_name.clone().unwrap_or_default();
                let source = mm.monster_source.clone().unwrap_or_default();
                let data =
                    catalog_dal::get_monster_by_name(self.conn, &name, &source)?.map(|m| m.data);
                (name, data)
            }
        };
        let base_name = mm.display_name.clone().unwrap_or(base_name);
        let (hp, dex) = stat_block_hp_and_dex(data.as_deref());

        let already = self
            .ordered_entries(session_id)?
            .iter()
            .filter(|e| e.source_id.as_deref() == Some(module_monster_id))
            .count() as i32;
        let numbered = already + count > 1;

        let mut ids = Vec::new();
        self.conn.transaction::<_, ServiceError, _>(|conn| {
            for i in 1..=count {
                let id = Uuid::new_v4().to_string();
                let name = if numbered {
                    format!("{} {}", base_name, already + i)
                } else {
                    base_name.clone()
                };
                dal::insert_combat_entry(
                    conn,
                    &NewCombatEntry {
                        id: &id,
                        session_id,
                        source_kind: "module_monster",
                        source_id: Some(module_monster_id),
                        token_id: None,
                        display_name: &name,
                        initiative: None,
                        dex_modifier: dex,
                        max_hp: hp,
                        current_hp: hp,
                    },
                )?;
                ids.push(id);
            }
            Ok(())
        })?;
        ids.iter().map(|id| self.view(id)).collect()
    }

    /// Add a module NPC. HP and DEX come from its stat block when it has one.
    pub fn add_module_npc(
        &mut self,
        session_id: &str,
        npc_id: &str,
    ) -> ServiceResult<CombatEntryView> {
        self.active_session(session_id)?;
        let npc = dal::get_module_npc_optional(self.conn, npc_id)?
            .ok_or_else(|| ServiceError::not_found("ModuleNpc", npc_id))?;
        let (hp, dex) = stat_block_hp_and_dex(npc.stat_block.as_deref());
        self.insert(
            session_id,
            "module_npc",
            Some(npc_id),
            &npc.name,
            None,
            dex,
            hp,
        )
    }

    /// Add a character. Characters have no HP column; set it with `set_max_hp`.
    pub fn add_character(
        &mut self,
        session_id: &str,
        character_id: &str,
    ) -> ServiceResult<CombatEntryView> {
        self.active_session(session_id)?;
        let c = dal::get_character_optional(self.conn, character_id)?
            .ok_or_else(|| ServiceError::not_found("Character", character_id))?;
        let dex = Character::ability_modifier(c.dexterity);
        self.insert(
            session_id,
            "character",
            Some(character_id),
            &c.name,
            None,
            Some(dex),
            None,
        )
    }

    /// Add a custom entry (lair action, summoned creature, ...).
    pub fn add_custom(
        &mut self,
        session_id: &str,
        name: &str,
        max_hp: Option<i32>,
        initiative: Option<i32>,
    ) -> ServiceResult<CombatEntryView> {
        self.active_session(session_id)?;
        let name = name.trim();
        if name.is_empty() {
            return Err(ServiceError::validation("name must not be empty"));
        }
        if max_hp.is_some_and(|hp| hp < 1) {
            return Err(ServiceError::validation("max HP must be at least 1"));
        }
        self.insert(session_id, "custom", None, name, initiative, None, max_hp)
    }

    /// Remove an entry; the current turn index stays in range.
    pub fn remove_entry(&mut self, entry_id: &str) -> ServiceResult<CombatState> {
        let entry = self.entry(entry_id)?;
        let mut session = self.active_session(&entry.session_id)?;
        dal::delete_combat_entry(self.conn, entry_id)?;
        let remaining = self.ordered_entries(&session.id)?.len() as i32;
        let clamped = session.turn_index.min((remaining - 1).max(0));
        if clamped != session.turn_index {
            session.turn_index = clamped;
            session.updated_at = now_rfc3339();
            dal::save_combat_session(self.conn, &session)?;
        }
        self.state(&session.id)
    }

    // --- Entry changes ----------------------------------------------------

    pub fn set_initiative(
        &mut self,
        entry_id: &str,
        initiative: Option<i32>,
    ) -> ServiceResult<CombatEntryView> {
        self.update(entry_id, |e, _| {
            e.initiative = initiative;
            Ok(())
        })
    }

    /// Set (or clear) max HP; current HP is filled in or capped to match.
    pub fn set_max_hp(
        &mut self,
        entry_id: &str,
        max_hp: Option<i32>,
    ) -> ServiceResult<CombatEntryView> {
        if max_hp.is_some_and(|hp| hp < 1) {
            return Err(ServiceError::validation("max HP must be at least 1"));
        }
        self.update(entry_id, |e, _| {
            e.max_hp = max_hp;
            e.current_hp = match (max_hp, e.current_hp) {
                (None, _) => None,
                (Some(max), None) => Some(max),
                (Some(max), Some(cur)) => Some(cur.min(max)),
            };
            Ok(())
        })
    }

    /// Link (or unlink) the entry to a map token.
    pub fn link_token(
        &mut self,
        entry_id: &str,
        token_id: Option<&str>,
    ) -> ServiceResult<CombatEntryView> {
        self.update(entry_id, |e, _| {
            e.token_id = token_id.map(str::to_string);
            Ok(())
        })
    }

    /// Link the fight's unlinked monster and NPC entries to their tokens on
    /// a map. An entry takes a free token of its source: first one whose label
    /// is the entry's name, then one whose label ends in the same number, then
    /// the first free one in label order. Tokens linked already stay linked.
    pub fn link_tokens(&mut self, session_id: &str, map_id: &str) -> ServiceResult<CombatState> {
        self.active_session(session_id)?;
        let mut tokens = dal::list_token_placements(self.conn, map_id)?;
        tokens.sort_by(|a, b| {
            let (la, lb) = (a.label.as_deref(), b.label.as_deref());
            (la.and_then(trailing_number), la, &a.id).cmp(&(
                lb.and_then(trailing_number),
                lb,
                &b.id,
            ))
        });
        let entries = self.ordered_entries(session_id)?;
        let mut taken: Vec<String> = entries.iter().filter_map(|e| e.token_id.clone()).collect();
        let mut changed = Vec::new();
        for mut entry in entries.into_iter().filter(|e| e.token_id.is_none()) {
            let Some(source) = entry.source_id.clone() else {
                continue;
            };
            let candidates: Vec<&TokenPlacement> = tokens
                .iter()
                .filter(|t| !taken.contains(&t.id))
                .filter(|t| match entry.source_kind.as_str() {
                    "module_monster" => t.module_monster_id.as_deref() == Some(&source),
                    "module_npc" => t.module_npc_id.as_deref() == Some(&source),
                    _ => false,
                })
                .collect();
            let number = trailing_number(&entry.display_name);
            let pick = candidates
                .iter()
                .find(|t| t.label.as_deref() == Some(entry.display_name.as_str()))
                .or_else(|| {
                    candidates.iter().find(|t| {
                        number.is_some() && t.label.as_deref().and_then(trailing_number) == number
                    })
                })
                .or_else(|| candidates.first());
            if let Some(token) = pick {
                taken.push(token.id.clone());
                entry.token_id = Some(token.id.clone());
                entry.updated_at = now_rfc3339();
                changed.push(entry);
            }
        }
        self.conn.transaction::<_, ServiceError, _>(|conn| {
            for entry in &changed {
                dal::save_combat_entry(conn, entry)?;
            }
            Ok(())
        })?;
        self.state(session_id)
    }

    /// Damage: temporary HP absorbs it first; HP never drops below 0. When
    /// the entry concentrates, the result carries the save DC.
    pub fn damage(&mut self, entry_id: &str, amount: i32) -> ServiceResult<DamageResult> {
        if amount < 0 {
            return Err(ServiceError::validation("damage must not be negative"));
        }
        let mut dc = None;
        let entry = self.update(entry_id, |e, round| {
            let current = e
                .current_hp
                .ok_or_else(|| ServiceError::validation("set this entry's HP first"))?;
            let absorbed = e.temp_hp.min(amount);
            e.temp_hp -= absorbed;
            e.current_hp = Some((current - (amount - absorbed)).max(0));
            push_log(e, "damage", amount, round);
            if e.is_concentrating != 0 && amount > 0 {
                dc = Some((amount / 2).max(10));
            }
            Ok(())
        })?;
        Ok(DamageResult {
            entry,
            concentration_dc: dc,
        })
    }

    /// Heal: HP never rises above max. The log records the HP restored.
    pub fn heal(&mut self, entry_id: &str, amount: i32) -> ServiceResult<CombatEntryView> {
        if amount < 0 {
            return Err(ServiceError::validation("healing must not be negative"));
        }
        self.update(entry_id, |e, round| {
            let current = e
                .current_hp
                .ok_or_else(|| ServiceError::validation("set this entry's HP first"))?;
            let healed = current + amount;
            let new_hp = e.max_hp.map_or(healed, |max| healed.min(max));
            e.current_hp = Some(new_hp);
            // Log the HP actually restored, not the amount asked for.
            push_log(e, "heal", new_hp - current, round);
            Ok(())
        })
    }

    /// Set temporary HP (they do not stack in 5e; this replaces them).
    pub fn set_temp_hp(&mut self, entry_id: &str, amount: i32) -> ServiceResult<CombatEntryView> {
        if amount < 0 {
            return Err(ServiceError::validation(
                "temporary HP must not be negative",
            ));
        }
        self.update(entry_id, |e, _| {
            e.temp_hp = amount;
            Ok(())
        })
    }

    /// Add an SRD condition, optionally for `duration_rounds` rounds
    /// (including the current one). Adding it again replaces the duration.
    pub fn add_condition(
        &mut self,
        entry_id: &str,
        name: &str,
        duration_rounds: Option<i32>,
    ) -> ServiceResult<CombatEntryView> {
        let name = name.trim().to_lowercase();
        if !SRD_CONDITIONS.contains(&name.as_str()) {
            return Err(ServiceError::validation(format!(
                "Unknown condition '{}'; use one of: {}",
                name,
                SRD_CONDITIONS.join(", ")
            )));
        }
        if duration_rounds.is_some_and(|d| d < 1) {
            return Err(ServiceError::validation(
                "duration must be at least 1 round",
            ));
        }
        self.update(entry_id, |e, round| {
            let mut conditions: Vec<ActiveCondition> =
                serde_json::from_str(&e.conditions).unwrap_or_default();
            conditions.retain(|c| c.name != name);
            conditions.push(ActiveCondition {
                name: name.clone(),
                expires_round: duration_rounds.map(|d| round + d - 1),
            });
            e.conditions = to_json(&conditions);
            Ok(())
        })
    }

    pub fn remove_condition(
        &mut self,
        entry_id: &str,
        name: &str,
    ) -> ServiceResult<CombatEntryView> {
        let name = name.trim().to_lowercase();
        self.update(entry_id, |e, _| {
            let mut conditions: Vec<ActiveCondition> =
                serde_json::from_str(&e.conditions).unwrap_or_default();
            conditions.retain(|c| c.name != name);
            e.conditions = to_json(&conditions);
            Ok(())
        })
    }

    pub fn set_concentration(
        &mut self,
        entry_id: &str,
        concentrating: bool,
    ) -> ServiceResult<CombatEntryView> {
        self.update(entry_id, |e, _| {
            e.is_concentrating = i32::from(concentrating);
            Ok(())
        })
    }

    /// The session an entry belongs to.
    pub fn session_of_entry(&mut self, entry_id: &str) -> ServiceResult<String> {
        Ok(self.entry(entry_id)?.session_id)
    }

    // --- Helpers -----------------------------------------------------------

    /// The session if it exists and is still active.
    fn active_session(&mut self, session_id: &str) -> ServiceResult<CombatSession> {
        let session = dal::get_combat_session_optional(self.conn, session_id)?
            .ok_or_else(|| ServiceError::not_found("CombatSession", session_id))?;
        if !session.is_active() {
            return Err(ServiceError::validation("this combat has ended"));
        }
        Ok(session)
    }

    fn ordered_entries(&mut self, session_id: &str) -> ServiceResult<Vec<CombatEntry>> {
        let mut entries = dal::list_combat_entries(self.conn, session_id)?;
        sort_turn_order(&mut entries);
        Ok(entries)
    }

    fn entry(&mut self, entry_id: &str) -> ServiceResult<CombatEntry> {
        dal::get_combat_entry_optional(self.conn, entry_id)?
            .ok_or_else(|| ServiceError::not_found("CombatEntry", entry_id))
    }

    fn view(&mut self, entry_id: &str) -> ServiceResult<CombatEntryView> {
        Ok(self.entry(entry_id)?.into())
    }

    /// Load an entry of an active fight, change it, save it.
    fn update(
        &mut self,
        entry_id: &str,
        change: impl FnOnce(&mut CombatEntry, i32) -> ServiceResult<()>,
    ) -> ServiceResult<CombatEntryView> {
        let mut entry = self.entry(entry_id)?;
        let session = self.active_session(&entry.session_id)?;
        change(&mut entry, session.round)?;
        entry.updated_at = now_rfc3339();
        dal::save_combat_entry(self.conn, &entry)?;
        Ok(entry.into())
    }

    #[allow(clippy::too_many_arguments)] // one argument per entry column
    fn insert(
        &mut self,
        session_id: &str,
        source_kind: &str,
        source_id: Option<&str>,
        name: &str,
        initiative: Option<i32>,
        dex_modifier: Option<i32>,
        max_hp: Option<i32>,
    ) -> ServiceResult<CombatEntryView> {
        let id = Uuid::new_v4().to_string();
        dal::insert_combat_entry(
            self.conn,
            &NewCombatEntry {
                id: &id,
                session_id,
                source_kind,
                source_id,
                token_id: None,
                display_name: name,
                initiative,
                dex_modifier,
                max_hp,
                current_hp: max_hp,
            },
        )?;
        self.view(&id)
    }
}

/// The number a label ends in: "Goblin 3" is 3.
fn trailing_number(label: &str) -> Option<u32> {
    label.rsplit(' ').next()?.parse().ok()
}

fn push_log(e: &mut CombatEntry, kind: &str, amount: i32, round: i32) {
    let mut log: Vec<HpChange> = serde_json::from_str(&e.damage_log).unwrap_or_default();
    log.push(HpChange {
        kind: kind.to_string(),
        amount,
        round,
    });
    let excess = log.len().saturating_sub(DAMAGE_LOG_LEN);
    log.drain(..excess);
    e.damage_log = to_json(&log);
}

fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::campaign as dal;
    use crate::models::campaign::{
        NewCampaign, NewCharacter, NewModule, NewModuleMonster, NewModuleNpc,
    };
    use crate::seed::srd::seed_srd_catalog;
    use crate::test_utils::setup_test_db;
    use diesel::SqliteConnection;

    const MODULE: &str = "mod-1";

    fn setup() -> SqliteConnection {
        let mut conn = setup_test_db();
        seed_srd_catalog(&mut conn).unwrap();
        dal::insert_campaign(&mut conn, &NewCampaign::new("camp-1", "Test")).unwrap();
        dal::insert_module(&mut conn, &NewModule::new(MODULE, "camp-1", "Cave", 1)).unwrap();
        // Goblin: HP 7 (2d6), DEX 14 (+2). Wolf: HP 11, DEX 15 (+2). Ogre: HP 59, DEX 8 (-1).
        dal::insert_module_monster(
            &mut conn,
            &NewModuleMonster::new("mm-goblin", MODULE, "Goblin", "MM").with_quantity(3),
        )
        .unwrap();
        dal::insert_module_monster(
            &mut conn,
            &NewModuleMonster::new("mm-ogre", MODULE, "Ogre", "MM").with_display_name("Big Grum"),
        )
        .unwrap();
        dal::insert_module_monster(
            &mut conn,
            &NewModuleMonster::new("mm-wolf", MODULE, "Wolf", "MM"),
        )
        .unwrap();
        dal::insert_character(
            &mut conn,
            &NewCharacter::new_pc("pc-1", Some("camp-1"), "Thorin", "Alice")
                .with_ability_scores(16, 12, 16, 10, 12, 8),
        )
        .unwrap();
        dal::insert_module_npc(&mut conn, &NewModuleNpc::new("npc-1", MODULE, "Sildar")).unwrap();
        conn
    }

    fn names(state: &CombatState) -> Vec<String> {
        state
            .entries
            .iter()
            .map(|e| e.display_name.clone())
            .collect()
    }

    fn entry<'s>(state: &'s CombatState, name: &str) -> &'s CombatEntryView {
        state
            .entries
            .iter()
            .find(|e| e.display_name == name)
            .unwrap()
    }

    #[test]
    fn start_resumes_the_active_session_and_allows_one_per_module() {
        let mut conn = setup();
        let first = CombatService::new(&mut conn).start(MODULE).unwrap();
        // A new service instance (e.g. after an app restart) resumes it.
        let again = CombatService::new(&mut conn).start(MODULE).unwrap();
        assert_eq!(first.session.id, again.session.id);
        assert_eq!(again.session.round, 1);

        CombatService::new(&mut conn)
            .end(&first.session.id)
            .unwrap();
        assert!(CombatService::new(&mut conn)
            .active(MODULE)
            .unwrap()
            .is_none());
        let next = CombatService::new(&mut conn).start(MODULE).unwrap();
        assert_ne!(
            next.session.id, first.session.id,
            "a new fight after the old one ended"
        );
    }

    #[test]
    fn module_monsters_add_one_numbered_entry_each_with_stat_block_hp() {
        let mut conn = setup();
        let mut svc = CombatService::new(&mut conn);
        let s = svc.start(MODULE).unwrap().session.id;

        let added = svc.add_module_monster(&s, "mm-goblin", None).unwrap();
        assert_eq!(added.len(), 3, "quantity 3 adds three goblins");
        let ogre = svc.add_module_monster(&s, "mm-ogre", None).unwrap();
        let wolf = svc.add_module_monster(&s, "mm-wolf", None).unwrap();

        let state = svc.state(&s).unwrap();
        let mut got = names(&state);
        got.sort();
        assert_eq!(
            got,
            ["Big Grum", "Goblin 1", "Goblin 2", "Goblin 3", "Wolf"]
        );
        let g = entry(&state, "Goblin 2");
        assert_eq!(
            (g.max_hp, g.current_hp, g.dex_modifier),
            (Some(7), Some(7), Some(2))
        );
        assert_eq!(
            ogre[0].max_hp,
            Some(59),
            "display name override keeps stat-block HP"
        );
        assert_eq!(ogre[0].dex_modifier, Some(-1));
        assert_eq!(
            wolf[0].display_name, "Wolf",
            "a single monster is not numbered"
        );

        // An explicit count overrides the quantity.
        assert_eq!(
            svc.add_module_monster(&s, "mm-goblin", Some(1))
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn characters_npcs_and_custom_entries() {
        let mut conn = setup();
        let mut svc = CombatService::new(&mut conn);
        let s = svc.start(MODULE).unwrap().session.id;

        let pc = svc.add_character(&s, "pc-1").unwrap();
        assert_eq!(pc.display_name, "Thorin");
        assert_eq!(pc.dex_modifier, Some(1));
        assert_eq!(
            pc.max_hp, None,
            "characters have no HP column; set it by hand"
        );
        let pc = svc.set_max_hp(&pc.id, Some(44)).unwrap();
        assert_eq!((pc.max_hp, pc.current_hp), (Some(44), Some(44)));

        let npc = svc.add_module_npc(&s, "npc-1").unwrap();
        assert_eq!(
            (npc.display_name.as_str(), npc.source_kind.as_str()),
            ("Sildar", "module_npc")
        );

        let custom = svc.add_custom(&s, "Lair action", None, Some(20)).unwrap();
        assert_eq!((custom.initiative, custom.max_hp), (Some(20), None));

        assert!(matches!(
            svc.add_character(&s, "nope"),
            Err(ServiceError::NotFound { .. })
        ));
        assert!(matches!(
            svc.add_custom(&s, "  ", None, None),
            Err(ServiceError::Validation(_))
        ));
    }

    #[test]
    fn turn_order_is_initiative_then_dex_then_name() {
        let mut conn = setup();
        let mut svc = CombatService::new(&mut conn);
        let s = svc.start(MODULE).unwrap().session.id;
        let ogre = svc
            .add_module_monster(&s, "mm-ogre", None)
            .unwrap()
            .remove(0); // DEX -1
        let wolf = svc
            .add_module_monster(&s, "mm-wolf", None)
            .unwrap()
            .remove(0); // DEX +2
        let pc = svc.add_character(&s, "pc-1").unwrap(); // DEX +1
        let lair = svc.add_custom(&s, "Lair action", None, None).unwrap();

        svc.set_initiative(&ogre.id, Some(12)).unwrap();
        svc.set_initiative(&wolf.id, Some(12)).unwrap();
        svc.set_initiative(&pc.id, Some(18)).unwrap();
        // Lair action has no initiative yet: last.
        let state = svc.state(&s).unwrap();
        assert_eq!(names(&state), ["Thorin", "Wolf", "Big Grum", "Lair action"]);

        svc.set_initiative(&lair.id, Some(20)).unwrap();
        assert_eq!(names(&svc.state(&s).unwrap())[0], "Lair action");
    }

    #[test]
    fn next_turn_wraps_into_the_next_round_and_previous_goes_back() {
        let mut conn = setup();
        let mut svc = CombatService::new(&mut conn);
        let s = svc.start(MODULE).unwrap().session.id;
        let a = svc.add_custom(&s, "A", None, Some(20)).unwrap();
        let b = svc.add_custom(&s, "B", None, Some(10)).unwrap();

        let st = svc.state(&s).unwrap();
        assert_eq!(st.current_entry_id.as_deref(), Some(a.id.as_str()));
        let st = svc.next_turn(&s).unwrap();
        assert_eq!(
            (st.session.round, st.current_entry_id.as_deref()),
            (1, Some(b.id.as_str()))
        );
        let st = svc.next_turn(&s).unwrap();
        assert_eq!(
            (st.session.round, st.current_entry_id.as_deref()),
            (2, Some(a.id.as_str()))
        );
        let st = svc.previous_turn(&s).unwrap();
        assert_eq!(
            (st.session.round, st.current_entry_id.as_deref()),
            (1, Some(b.id.as_str()))
        );
        // Previous at the very start stays put.
        svc.previous_turn(&s).unwrap();
        let st = svc.previous_turn(&s).unwrap();
        assert_eq!((st.session.round, st.session.turn_index), (1, 0));
    }

    #[test]
    fn conditions_validate_names_and_expire_on_the_round_change() {
        let mut conn = setup();
        let mut svc = CombatService::new(&mut conn);
        let s = svc.start(MODULE).unwrap().session.id;
        let a = svc.add_custom(&s, "A", None, Some(20)).unwrap();

        // Lasts this round only.
        svc.add_condition(&a.id, "Frightened", Some(1)).unwrap();
        // No duration: until removed.
        let v = svc.add_condition(&a.id, "prone", None).unwrap();
        let mut got: Vec<_> = v.conditions.iter().map(|c| c.name.clone()).collect();
        got.sort();
        assert_eq!(got, ["frightened", "prone"]);
        assert!(matches!(
            svc.add_condition(&a.id, "sleepy", None),
            Err(ServiceError::Validation(_))
        ));

        // A is alone: the next turn starts round 2, and Frightened ends.
        let st = svc.next_turn(&s).unwrap();
        assert_eq!(st.session.round, 2);
        let names: Vec<_> = entry(&st, "A")
            .conditions
            .iter()
            .map(|c| c.name.clone())
            .collect();
        assert_eq!(names, ["prone"]);

        let v = svc.remove_condition(&a.id, "Prone").unwrap();
        assert!(v.conditions.is_empty());
    }

    #[test]
    fn damage_uses_temp_hp_first_and_hp_stays_in_bounds() {
        let mut conn = setup();
        let mut svc = CombatService::new(&mut conn);
        let s = svc.start(MODULE).unwrap().session.id;
        let g = svc
            .add_module_monster(&s, "mm-goblin", Some(1))
            .unwrap()
            .remove(0); // 7 HP

        svc.set_temp_hp(&g.id, 3).unwrap();
        let r = svc.damage(&g.id, 5).unwrap();
        assert_eq!((r.entry.temp_hp, r.entry.current_hp), (0, Some(5)));
        let r = svc.damage(&g.id, 50).unwrap();
        assert_eq!(r.entry.current_hp, Some(0), "never below 0");
        let v = svc.heal(&g.id, 100).unwrap();
        assert_eq!(v.current_hp, Some(7), "never above max");

        // Last three changes, newest last.
        svc.damage(&g.id, 1).unwrap();
        let v = svc.damage(&g.id, 2).unwrap().entry;
        let log: Vec<_> = v
            .damage_log
            .iter()
            .map(|c| (c.kind.as_str(), c.amount))
            .collect();
        assert_eq!(log, [("heal", 7), ("damage", 1), ("damage", 2)]);

        assert!(matches!(
            svc.damage(&g.id, -1),
            Err(ServiceError::Validation(_))
        ));
        let pc = svc.add_character(&s, "pc-1").unwrap();
        assert!(
            matches!(svc.damage(&pc.id, 1), Err(ServiceError::Validation(_))),
            "no HP set"
        );
    }

    #[test]
    fn damage_to_a_concentrating_creature_reports_the_save_dc() {
        let mut conn = setup();
        let mut svc = CombatService::new(&mut conn);
        let s = svc.start(MODULE).unwrap().session.id;
        let o = svc
            .add_module_monster(&s, "mm-ogre", None)
            .unwrap()
            .remove(0); // 59 HP

        assert_eq!(svc.damage(&o.id, 4).unwrap().concentration_dc, None);
        svc.set_concentration(&o.id, true).unwrap();
        assert_eq!(
            svc.damage(&o.id, 4).unwrap().concentration_dc,
            Some(10),
            "minimum 10"
        );
        assert_eq!(svc.damage(&o.id, 30).unwrap().concentration_dc, Some(15));
        assert_eq!(
            svc.damage(&o.id, 25).unwrap().concentration_dc,
            Some(12),
            "half, rounded down"
        );
    }

    #[test]
    fn removing_entries_keeps_the_turn_in_range() {
        let mut conn = setup();
        let mut svc = CombatService::new(&mut conn);
        let s = svc.start(MODULE).unwrap().session.id;
        let _a = svc.add_custom(&s, "A", None, Some(20)).unwrap();
        let b = svc.add_custom(&s, "B", None, Some(10)).unwrap();
        svc.next_turn(&s).unwrap(); // B's turn (index 1)
        let st = svc.remove_entry(&b.id).unwrap();
        assert_eq!(names(&st), ["A"]);
        assert_eq!(st.session.turn_index, 0);
        assert!(st.current_entry_id.is_some());
    }

    #[test]
    fn deleting_the_module_deletes_its_combat() {
        let mut conn = setup();
        let s = CombatService::new(&mut conn)
            .start(MODULE)
            .unwrap()
            .session
            .id;
        CombatService::new(&mut conn)
            .add_custom(&s, "A", None, None)
            .unwrap();
        dal::delete_module(&mut conn, MODULE).unwrap();
        assert!(dal::get_combat_session_optional(&mut conn, &s)
            .unwrap()
            .is_none());
        assert!(dal::list_combat_entries(&mut conn, &s).unwrap().is_empty());
    }

    #[test]
    fn ended_sessions_refuse_changes() {
        let mut conn = setup();
        let mut svc = CombatService::new(&mut conn);
        let s = svc.start(MODULE).unwrap().session.id;
        svc.end(&s).unwrap();
        assert!(matches!(
            svc.add_custom(&s, "A", None, None),
            Err(ServiceError::Validation(_))
        ));
        assert!(matches!(
            svc.next_turn(&s),
            Err(ServiceError::Validation(_))
        ));
    }

    #[test]
    fn link_tokens_matches_name_then_number_then_first_free() {
        use crate::models::campaign::{NewCampaignAsset, NewMap, NewTokenPlacement};
        let mut conn = setup();
        dal::insert_campaign_asset(
            &mut conn,
            &NewCampaignAsset::for_campaign(
                "asset-1",
                "camp-1",
                "cave.uvtt",
                "application/octet-stream",
                "/b/cave",
            ),
        )
        .unwrap();
        dal::insert_map(
            &mut conn,
            &NewMap::for_module("map-1", "camp-1", MODULE, "Cave", "asset-1"),
        )
        .unwrap();
        let tokens = [
            NewTokenPlacement::for_monster("tp-g3", "map-1", "mm-goblin", 0, 0)
                .with_label("Goblin 3"),
            NewTokenPlacement::for_monster("tp-boss", "map-1", "mm-goblin", 0, 0)
                .with_label("Boss"),
            NewTokenPlacement::for_monster("tp-a1", "map-1", "mm-goblin", 0, 0)
                .with_label("Archer 1"),
            NewTokenPlacement::for_monster("tp-wolf", "map-1", "mm-wolf", 0, 0),
            NewTokenPlacement::for_npc("tp-sildar", "map-1", "npc-1", 0, 0)
                .with_label("Sildar (captive)"),
        ];
        for t in &tokens {
            dal::insert_token_placement(&mut conn, t).unwrap();
        }
        let mut svc = CombatService::new(&mut conn);
        let s = svc.start(MODULE).unwrap().session.id;
        svc.add_module_monster(&s, "mm-goblin", None).unwrap(); // Goblin 1..3
        svc.add_module_monster(&s, "mm-ogre", None).unwrap();
        svc.add_module_npc(&s, "npc-1").unwrap();
        svc.add_custom(&s, "Trap", None, None).unwrap();

        let st = svc.link_tokens(&s, "map-1").unwrap();
        let token = |n: &str| entry(&st, n).token_id.clone();
        // "Goblin 3" by label; "Goblin 1" by number ("Archer 1"); "Goblin 2"
        // takes the first free goblin token, "Boss".
        assert_eq!(token("Goblin 3").as_deref(), Some("tp-g3"));
        assert_eq!(token("Goblin 1").as_deref(), Some("tp-a1"));
        assert_eq!(token("Goblin 2").as_deref(), Some("tp-boss"));
        assert_eq!(token("Sildar").as_deref(), Some("tp-sildar"));
        assert_eq!(token("Big Grum"), None);
        assert_eq!(token("Trap"), None);

        // A later goblin finds no free goblin token; the wolf takes its own.
        svc.add_module_monster(&s, "mm-goblin", Some(1)).unwrap();
        svc.add_module_monster(&s, "mm-wolf", None).unwrap();
        let st = svc.link_tokens(&s, "map-1").unwrap();
        assert_eq!(entry(&st, "Goblin 4").token_id, None);
        assert_eq!(entry(&st, "Wolf").token_id.as_deref(), Some("tp-wolf"));
        assert_eq!(entry(&st, "Goblin 3").token_id.as_deref(), Some("tp-g3"));
    }
}
