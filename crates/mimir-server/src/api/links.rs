//! Player links (MIMIR-T-0716, ADR MIMIR-A-0010), DM routes: the status of
//! a player character's link, a new link (it ends the old one), revoke.
//! Ending a link signs its device out: the next request gets 401, and its
//! socket gets `signed_out` and closes.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use mimir_core::services::PlayerLinkService;
use mimir_wire as wire;

use crate::error::ApiError;
use crate::live::Change;
use crate::state::AppState;

/// `GET /characters/{id}/link`
pub async fn status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<wire::LinkStatus>, ApiError> {
    state
        .with_db(move |conn| {
            let created_at = PlayerLinkService::new(conn).created_at(&id)?;
            Ok(wire::LinkStatus {
                active: created_at.is_some(),
                created_at,
            })
        })
        .await
        .map(Json)
}

/// `POST /characters/{id}/link`: a new link; the old one stops working.
pub async fn issue(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<wire::NewLink>), ApiError> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let token = PlayerLinkService::new(conn).issue(&id)?;
            live.publish(Change::SignedOut { character_id: id });
            Ok((
                StatusCode::CREATED,
                Json(wire::NewLink {
                    path: format!("/play/{token}"),
                    token,
                }),
            ))
        })
        .await
}

/// `DELETE /characters/{id}/link`
pub async fn revoke(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            PlayerLinkService::new(conn).revoke(&id)?;
            live.publish(Change::SignedOut { character_id: id });
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}
