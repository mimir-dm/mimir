//! Player notes and party sharing (MIMIR-T-0718, ADR MIMIR-A-0010).
//! A player writes their own character's notes; the DM reads them (on the
//! character and per module). A player shares their sheet with the party
//! (the DM may set it too); the party list shows who shares.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use diesel::SqliteConnection;
use mimir_core::models::campaign::PlayerNote;
use mimir_core::services::{CharacterService, PartyService, ServiceError};
use mimir_wire as wire;

use super::scope;
use crate::auth::Caller;
use crate::error::{ApiError, JsonBody};
use crate::live::{Change, Hub};
use crate::state::AppState;

fn note(n: PlayerNote, character_name: String) -> wire::PlayerNote {
    wire::PlayerNote {
        id: n.id,
        character_id: n.character_id,
        character_name,
        module_id: n.module_id,
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
            Ok(PartyService::new(conn)
                .notes(&id)?
                .into_iter()
                .map(|n| note(n, name.clone()))
                .collect())
        })
        .await
        .map(Json)
}

/// `POST /characters/{id}/notes`: the character's player.
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
            let n = PartyService::new(conn).add_note(
                &id,
                body.module_id.as_deref(),
                &body.title,
                &body.content,
            )?;
            let name = name_of(conn, &id)?;
            notify(&live, conn, &id);
            Ok((StatusCode::CREATED, Json(note(n, name))))
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
            Ok(note(n, name))
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
            Ok(PartyService::new(conn)
                .module_notes(&id)?
                .into_iter()
                .map(|(n, name)| note(n, name))
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
