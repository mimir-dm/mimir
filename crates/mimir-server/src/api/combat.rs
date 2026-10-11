//! Combat (MIMIR-T-0715): one session per module over CombatService. Each
//! change goes to the live sockets of the campaign (`Change::Combat`).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use diesel::{Connection, SqliteConnection};
use mimir_core::services::{
    ActiveCondition, CombatEntryView, CombatService, CombatState, DamageResult, HpChange,
    ModuleService, ServiceError,
};
use mimir_wire as wire;

use crate::error::{ApiError, JsonBody};
use crate::live::{Change, Hub};
use crate::state::AppState;

type ApiResult<T> = Result<Json<T>, ApiError>;

pub fn entry(e: CombatEntryView) -> wire::CombatEntry {
    wire::CombatEntry {
        id: e.id,
        source_kind: e.source_kind,
        source_id: e.source_id,
        token_id: e.token_id,
        name: e.display_name,
        initiative: e.initiative,
        dex_modifier: e.dex_modifier,
        max_hp: e.max_hp,
        current_hp: e.current_hp,
        temp_hp: e.temp_hp,
        concentrating: e.is_concentrating,
        conditions: e
            .conditions
            .into_iter()
            .map(
                |ActiveCondition {
                     name,
                     expires_round,
                 }| wire::Condition {
                    name,
                    expires_round,
                },
            )
            .collect(),
        hp_log: e
            .damage_log
            .into_iter()
            .map(
                |HpChange {
                     kind,
                     amount,
                     round,
                 }| wire::HpChange {
                    kind,
                    amount,
                    round,
                },
            )
            .collect(),
    }
}

pub fn combat(s: CombatState) -> wire::Combat {
    wire::Combat {
        session: wire::CombatSession {
            id: s.session.id,
            module_id: s.session.module_id,
            round: s.session.round,
            turn_index: s.session.turn_index,
            status: s.session.status,
        },
        entries: s.entries.into_iter().map(entry).collect(),
        current_entry_id: s.current_entry_id,
    }
}

fn damage_result(d: DamageResult) -> wire::DamageResult {
    wire::DamageResult {
        entry: entry(d.entry),
        concentration_dc: d.concentration_dc,
    }
}

/// The turn order players see: names only, in turn order. Monsters and
/// NPCs show only when their token is one the players see (`seen`); PCs and
/// custom entries always.
pub fn player_initiative(state: &CombatState, seen: &[&str]) -> wire::PlayerInitiative {
    wire::PlayerInitiative {
        round: state.session.round,
        entries: state
            .entries
            .iter()
            .filter(|e| match e.source_kind.as_str() {
                "module_monster" | "module_npc" => {
                    e.token_id.as_deref().is_some_and(|t| seen.contains(&t))
                }
                _ => true,
            })
            .map(|e| wire::PlayerTurn {
                name: e.display_name.clone(),
                current: state.current_entry_id.as_deref() == Some(e.id.as_str()),
            })
            .collect(),
    }
}

/// Tell the sockets of the campaign of this module.
fn notify_module(live: &Hub, conn: &mut SqliteConnection, module_id: &str) {
    if let Ok(Some(m)) = ModuleService::new(conn).get(module_id) {
        live.publish(Change::Combat {
            campaign_id: m.campaign_id,
        });
    }
}

fn notify_session(live: &Hub, conn: &mut SqliteConnection, session_id: &str) {
    if let Ok(state) = CombatService::new(conn).state(session_id) {
        notify_module(live, conn, &state.session.module_id);
    }
}

fn notify_entry(live: &Hub, conn: &mut SqliteConnection, entry_id: &str) {
    if let Ok(session) = CombatService::new(conn).session_of_entry(entry_id) {
        notify_session(live, conn, &session);
    }
}

fn require_module(conn: &mut SqliteConnection, id: &str) -> Result<(), ApiError> {
    ModuleService::new(conn)
        .get(id)?
        .ok_or_else(|| ServiceError::not_found("Module", id))?;
    Ok(())
}

/// `GET /modules/{id}/combat`: the active combat, or `null`.
pub async fn active(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Option<wire::Combat>> {
    state
        .with_db(move |conn| {
            require_module(conn, &id)?;
            Ok(CombatService::new(conn).active(&id)?.map(combat))
        })
        .await
        .map(Json)
}

/// `POST /modules/{id}/combat`: start (or resume the active) combat.
pub async fn start(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<wire::Combat>), ApiError> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            require_module(conn, &id)?;
            let s = CombatService::new(conn).start(&id)?;
            notify_module(&live, conn, &id);
            Ok((StatusCode::CREATED, Json(combat(s))))
        })
        .await
}

/// `GET /combat/{id}`
pub async fn get(State(state): State<AppState>, Path(id): Path<String>) -> ApiResult<wire::Combat> {
    state
        .with_db(move |conn| Ok(combat(CombatService::new(conn).state(&id)?)))
        .await
        .map(Json)
}

/// `DELETE /combat/{id}`: end the combat.
pub async fn end(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let module = CombatService::new(conn).state(&id)?.session.module_id;
            CombatService::new(conn).end(&id)?;
            notify_module(&live, conn, &module);
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

/// `POST /combat/{id}/next`
pub async fn next(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::Combat> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let s = CombatService::new(conn).next_turn(&id)?;
            notify_module(&live, conn, &s.session.module_id);
            Ok(combat(s))
        })
        .await
        .map(Json)
}

/// `POST /combat/{id}/previous`
pub async fn previous(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::Combat> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let s = CombatService::new(conn).previous_turn(&id)?;
            notify_module(&live, conn, &s.session.module_id);
            Ok(combat(s))
        })
        .await
        .map(Json)
}

/// `POST /combat/{id}/entries`: add combatants; the answer is the whole
/// combat (the order changes).
pub async fn add(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewEntry>,
) -> Result<(StatusCode, Json<wire::Combat>), ApiError> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let mut service = CombatService::new(conn);
            match body {
                wire::NewEntry::Monster {
                    module_monster_id,
                    count,
                } => {
                    service.add_module_monster(&id, &module_monster_id, count)?;
                }
                wire::NewEntry::Npc { npc_id } => {
                    service.add_module_npc(&id, &npc_id)?;
                }
                wire::NewEntry::Character { character_id } => {
                    service.add_character(&id, &character_id)?;
                }
                wire::NewEntry::Custom {
                    name,
                    max_hp,
                    initiative,
                } => {
                    service.add_custom(&id, &name, max_hp, initiative)?;
                }
            }
            let s = service.state(&id)?;
            notify_module(&live, conn, &s.session.module_id);
            Ok((StatusCode::CREATED, Json(combat(s))))
        })
        .await
}

/// `POST /combat/{id}/link-tokens`: link monster and NPC entries to the
/// tokens of a map.
pub async fn link_tokens(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::LinkTokens>,
) -> ApiResult<wire::Combat> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let s = CombatService::new(conn).link_tokens(&id, &body.map_id)?;
            notify_module(&live, conn, &s.session.module_id);
            Ok(combat(s))
        })
        .await
        .map(Json)
}

/// `PATCH /combat-entries/{id}`: initiative, max HP, temp HP,
/// concentration, token. The answer is the whole combat (the order can
/// change).
pub async fn patch_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(patch): JsonBody<wire::EntryPatch>,
) -> ApiResult<wire::Combat> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let session = conn.transaction::<_, ApiError, _>(|conn| {
                let mut service = CombatService::new(conn);
                if let Some(initiative) = patch.initiative {
                    service.set_initiative(&id, initiative)?;
                }
                if let Some(max) = patch.max_hp {
                    service.set_max_hp(&id, max)?;
                }
                if let Some(temp) = patch.temp_hp {
                    service.set_temp_hp(&id, temp)?;
                }
                if let Some(on) = patch.concentrating {
                    service.set_concentration(&id, on)?;
                }
                if let Some(token) = patch.token_id {
                    service.link_token(&id, token.as_deref())?;
                }
                Ok(service.session_of_entry(&id)?)
            })?;
            let s = CombatService::new(conn).state(&session)?;
            notify_module(&live, conn, &s.session.module_id);
            Ok(combat(s))
        })
        .await
        .map(Json)
}

/// `DELETE /combat-entries/{id}`: the answer is the whole combat.
pub async fn remove_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::Combat> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let s = CombatService::new(conn).remove_entry(&id)?;
            notify_module(&live, conn, &s.session.module_id);
            Ok(combat(s))
        })
        .await
        .map(Json)
}

/// `POST /combat-entries/{id}/damage`
pub async fn damage(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::HpAmount>,
) -> ApiResult<wire::DamageResult> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let r = CombatService::new(conn).damage(&id, body.amount)?;
            notify_entry(&live, conn, &id);
            Ok(damage_result(r))
        })
        .await
        .map(Json)
}

/// `POST /combat-entries/{id}/heal`
pub async fn heal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::HpAmount>,
) -> ApiResult<wire::CombatEntry> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let e = CombatService::new(conn).heal(&id, body.amount)?;
            notify_entry(&live, conn, &id);
            Ok(entry(e))
        })
        .await
        .map(Json)
}

/// `POST /combat-entries/{id}/conditions`
pub async fn add_condition(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewCondition>,
) -> ApiResult<wire::CombatEntry> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let e =
                CombatService::new(conn).add_condition(&id, &body.name, body.duration_rounds)?;
            notify_entry(&live, conn, &id);
            Ok(entry(e))
        })
        .await
        .map(Json)
}

/// `DELETE /combat-entries/{id}/conditions/{name}`
pub async fn remove_condition(
    State(state): State<AppState>,
    Path((id, name)): Path<(String, String)>,
) -> ApiResult<wire::CombatEntry> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let e = CombatService::new(conn).remove_condition(&id, &name)?;
            notify_entry(&live, conn, &id);
            Ok(entry(e))
        })
        .await
        .map(Json)
}
