//! Combat Tracker Commands (COLLIERY-I-0468)
//!
//! Thin Tauri wrappers over `CombatService` — all combat rules live in
//! mimir-core. Live-play state: these have no MCP counterparts.

use mimir_core::services::{
    CombatEntryView, CombatService, CombatState, DamageResult, ServiceResult,
};
use serde::Serialize;
use tauri::State;

use crate::commands::{to_api_response, ApiResponse};
use crate::state::AppState;

/// Open a connection and run one service call.
fn run<T: Serialize>(
    state: &State<'_, AppState>,
    call: impl FnOnce(&mut CombatService) -> ServiceResult<T>,
) -> ApiResponse<T> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };
    to_api_response(call(&mut CombatService::new(&mut db)))
}

/// The module's active combat, if any.
#[tauri::command]
pub fn get_active_combat(
    state: State<'_, AppState>,
    module_id: String,
) -> ApiResponse<Option<CombatState>> {
    run(&state, |s| s.active(&module_id))
}

/// Resume the module's active combat, or start one.
#[tauri::command]
pub fn start_combat(state: State<'_, AppState>, module_id: String) -> ApiResponse<CombatState> {
    run(&state, |s| s.start(&module_id))
}

/// End a combat.
#[tauri::command]
pub fn end_combat(state: State<'_, AppState>, session_id: String) -> ApiResponse<()> {
    run(&state, |s| s.end(&session_id))
}

/// A combat with its entries in turn order.
#[tauri::command]
pub fn get_combat(state: State<'_, AppState>, session_id: String) -> ApiResponse<CombatState> {
    run(&state, |s| s.state(&session_id))
}

/// Advance to the next turn (and round).
#[tauri::command]
pub fn combat_next_turn(
    state: State<'_, AppState>,
    session_id: String,
) -> ApiResponse<CombatState> {
    run(&state, |s| s.next_turn(&session_id))
}

/// Go back one turn.
#[tauri::command]
pub fn combat_previous_turn(
    state: State<'_, AppState>,
    session_id: String,
) -> ApiResponse<CombatState> {
    run(&state, |s| s.previous_turn(&session_id))
}

/// Add a module monster group (default: its quantity) to a combat.
#[tauri::command]
pub fn add_combat_module_monster(
    state: State<'_, AppState>,
    session_id: String,
    module_monster_id: String,
    count: Option<i32>,
) -> ApiResponse<Vec<CombatEntryView>> {
    run(&state, |s| {
        s.add_module_monster(&session_id, &module_monster_id, count)
    })
}

/// Add a module NPC to a combat.
#[tauri::command]
pub fn add_combat_module_npc(
    state: State<'_, AppState>,
    session_id: String,
    npc_id: String,
) -> ApiResponse<CombatEntryView> {
    run(&state, |s| s.add_module_npc(&session_id, &npc_id))
}

/// Add a character to a combat.
#[tauri::command]
pub fn add_combat_character(
    state: State<'_, AppState>,
    session_id: String,
    character_id: String,
) -> ApiResponse<CombatEntryView> {
    run(&state, |s| s.add_character(&session_id, &character_id))
}

/// Add a custom entry to a combat.
#[tauri::command]
pub fn add_combat_custom(
    state: State<'_, AppState>,
    session_id: String,
    name: String,
    max_hp: Option<i32>,
    initiative: Option<i32>,
) -> ApiResponse<CombatEntryView> {
    run(&state, |s| {
        s.add_custom(&session_id, &name, max_hp, initiative)
    })
}

/// Remove an entry from its combat.
#[tauri::command]
pub fn remove_combat_entry(
    state: State<'_, AppState>,
    entry_id: String,
) -> ApiResponse<CombatState> {
    run(&state, |s| s.remove_entry(&entry_id))
}

/// Set or clear an entry's initiative.
#[tauri::command]
pub fn set_combat_initiative(
    state: State<'_, AppState>,
    entry_id: String,
    initiative: Option<i32>,
) -> ApiResponse<CombatEntryView> {
    run(&state, |s| s.set_initiative(&entry_id, initiative))
}

/// Set or clear an entry's max HP.
#[tauri::command]
pub fn set_combat_max_hp(
    state: State<'_, AppState>,
    entry_id: String,
    max_hp: Option<i32>,
) -> ApiResponse<CombatEntryView> {
    run(&state, |s| s.set_max_hp(&entry_id, max_hp))
}

/// Link or unlink an entry to a map token.
#[tauri::command]
pub fn link_combat_token(
    state: State<'_, AppState>,
    entry_id: String,
    token_id: Option<String>,
) -> ApiResponse<CombatEntryView> {
    run(&state, |s| s.link_token(&entry_id, token_id.as_deref()))
}

/// Link the fight's unlinked monster and NPC entries to their tokens on a map.
#[tauri::command]
pub fn link_combat_tokens(
    state: State<'_, AppState>,
    session_id: String,
    map_id: String,
) -> ApiResponse<CombatState> {
    run(&state, |s| s.link_tokens(&session_id, &map_id))
}

/// Damage an entry; reports a concentration save DC when it concentrates.
#[tauri::command]
pub fn combat_damage(
    state: State<'_, AppState>,
    entry_id: String,
    amount: i32,
) -> ApiResponse<DamageResult> {
    run(&state, |s| s.damage(&entry_id, amount))
}

/// Heal an entry.
#[tauri::command]
pub fn combat_heal(
    state: State<'_, AppState>,
    entry_id: String,
    amount: i32,
) -> ApiResponse<CombatEntryView> {
    run(&state, |s| s.heal(&entry_id, amount))
}

/// Set an entry's temporary HP.
#[tauri::command]
pub fn set_combat_temp_hp(
    state: State<'_, AppState>,
    entry_id: String,
    amount: i32,
) -> ApiResponse<CombatEntryView> {
    run(&state, |s| s.set_temp_hp(&entry_id, amount))
}

/// Add an SRD condition to an entry, optionally for some rounds.
#[tauri::command]
pub fn add_combat_condition(
    state: State<'_, AppState>,
    entry_id: String,
    name: String,
    duration_rounds: Option<i32>,
) -> ApiResponse<CombatEntryView> {
    run(&state, |s| {
        s.add_condition(&entry_id, &name, duration_rounds)
    })
}

/// Remove a condition from an entry.
#[tauri::command]
pub fn remove_combat_condition(
    state: State<'_, AppState>,
    entry_id: String,
    name: String,
) -> ApiResponse<CombatEntryView> {
    run(&state, |s| s.remove_condition(&entry_id, &name))
}

/// Set whether an entry is concentrating.
#[tauri::command]
pub fn set_combat_concentration(
    state: State<'_, AppState>,
    entry_id: String,
    concentrating: bool,
) -> ApiResponse<CombatEntryView> {
    run(&state, |s| s.set_concentration(&entry_id, concentrating))
}
