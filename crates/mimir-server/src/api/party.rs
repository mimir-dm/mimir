//! Player notes and party sharing (MIMIR-T-0718, ADR MIMIR-A-0010).
//! A player writes their own character's notes; the DM reads them (on the
//! character and per module). A player shares their sheet with the party
//! (the DM may set it too); the party list shows who shares.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use diesel::SqliteConnection;
use mimir_core::dal::campaign::get_map;
use mimir_core::models::campaign::PlayerNote;
use mimir_core::services::{CharacterService, ModuleService, PartyService, ServiceError};
use mimir_wire as wire;

use super::scope;
use crate::auth::Caller;
use crate::error::{ApiError, JsonBody};
use crate::live::{Change, Hub};
use crate::state::AppState;

fn note(conn: &mut SqliteConnection, n: PlayerNote, character_name: String) -> wire::PlayerNote {
    let module_name = n
        .module_id
        .as_deref()
        .and_then(|m| ModuleService::new(conn).get(m).ok().flatten())
        .map(|m| m.name);
    wire::PlayerNote {
        id: n.id,
        character_id: n.character_id,
        character_name,
        module_id: n.module_id,
        module_name,
        title: n.title,
        content: n.content,
        updated_at: n.updated_at,
    }
}

fn name_of(conn: &mut SqliteConnection, character_id: &str) -> Result<String, ApiError> {
    Ok(CharacterService::new(conn)
        .get(character_id)?
        .ok_or_else(|| ServiceError::not_found("Character", character_id))?
        .name)
}

fn notify(live: &Hub, conn: &mut SqliteConnection, character_id: &str) {
    if let Ok(Some(c)) = CharacterService::new(conn).get(character_id) {
        live.publish(Change::Character {
            campaign_id: c.campaign_id,
            character_id: character_id.to_string(),
        });
    }
}

/// `GET /characters/{id}/notes`: the DM, or the character's player.
pub async fn list(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
) -> Result<Json<Vec<wire::PlayerNote>>, ApiError> {
    scope::character(&caller, &id)?;
    state
        .with_db(move |conn| {
            let name = name_of(conn, &id)?;
            let notes = PartyService::new(conn).notes(&id)?;
            Ok(notes
                .into_iter()
                .map(|n| note(conn, n, name.clone()))
                .collect())
        })
        .await
        .map(Json)
}

/// The module of the map on a campaign's display, if any.
fn shown_module(conn: &mut SqliteConnection, live: &Hub, campaign_id: &str) -> Option<String> {
    let map_id = live.display(campaign_id).map_id?;
    get_map(conn, &map_id).ok()?.module_id
}

/// `POST /characters/{id}/notes`: the character's player. A note with no
/// module is about the module whose map is on the display (the encounter
/// the player is in).
pub async fn add(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewNote>,
) -> Result<(StatusCode, Json<wire::PlayerNote>), ApiError> {
    scope::own_player(&caller, &id)?;
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let module = match (&body.module_id, &caller) {
                (Some(m), _) => Some(m.clone()),
                (None, Caller::Player(p)) => shown_module(conn, &live, &p.campaign_id),
                (None, Caller::Dm) => None,
            };
            let n = PartyService::new(conn).add_note(
                &id,
                module.as_deref(),
                &body.title,
                &body.content,
            )?;
            let name = name_of(conn, &id)?;
            notify(&live, conn, &id);
            Ok((StatusCode::CREATED, Json(note(conn, n, name))))
        })
        .await
}

fn own_note(
    caller: &Caller,
    conn: &mut SqliteConnection,
    id: &str,
) -> Result<PlayerNote, ApiError> {
    let n = PartyService::new(conn).note(id)?;
    scope::own_player(caller, &n.character_id)?;
    Ok(n)
}

/// `PATCH /notes/{id}`: the note's player.
pub async fn patch(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
    JsonBody(p): JsonBody<wire::NotePatch>,
) -> Result<Json<wire::PlayerNote>, ApiError> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let old = own_note(&caller, conn, &id)?;
            let n = PartyService::new(conn).update_note(
                &id,
                p.title.as_deref(),
                p.content.as_deref(),
                p.module_id.as_ref().map(|m| m.as_deref()),
            )?;
            let name = name_of(conn, &old.character_id)?;
            notify(&live, conn, &old.character_id);
            Ok(note(conn, n, name))
        })
        .await
        .map(Json)
}

/// `DELETE /notes/{id}`: the note's player.
pub async fn delete(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let old = own_note(&caller, conn, &id)?;
            PartyService::new(conn).delete_note(&id)?;
            notify(&live, conn, &old.character_id);
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

/// `GET /modules/{id}/notes` (DM): the players' notes about a module.
pub async fn module_notes(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<wire::PlayerNote>>, ApiError> {
    state
        .with_db(move |conn| {
            let notes = PartyService::new(conn).module_notes(&id)?;
            Ok(notes
                .into_iter()
                .map(|(n, name)| note(conn, n, name))
                .collect())
        })
        .await
        .map(Json)
}

/// `PUT /characters/{id}/sharing`: the character's player or the DM.
pub async fn set_sharing(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::Sharing>,
) -> Result<Json<wire::Sharing>, ApiError> {
    scope::character(&caller, &id)?;
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let shared = PartyService::new(conn).set_shared(&id, body.shared)?;
            notify(&live, conn, &id);
            Ok(wire::Sharing { shared })
        })
        .await
        .map(Json)
}

/// `GET /campaigns/{id}/party`: the player characters and who shares.
pub async fn party(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
) -> Result<Json<Vec<wire::PartyMember>>, ApiError> {
    scope::campaign(&caller, &id)?;
    let me = match &caller {
        Caller::Player(p) => Some(p.character_id.clone()),
        Caller::Dm => None,
    };
    state
        .with_db(move |conn| {
            Ok(PartyService::new(conn)
                .party(&id)?
                .into_iter()
                .map(|m| wire::PartyMember {
                    me: me.as_deref() == Some(m.character.id.as_str()),
                    character_id: m.character.id,
                    name: m.character.name,
                    player_name: m.character.player_name,
                    race_name: m.character.race_name,
                    shared: m.shared,
                })
                .collect())
        })
        .await
        .map(Json)
}
