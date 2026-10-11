//! The player display of a campaign (MIMIR-T-0712): which map it shows,
//! and blackout. The state lives in the live hub (memory); a change goes
//! to the sockets of the campaign.

use axum::extract::{Path, State};
use axum::Json;
use mimir_core::services::{CampaignService, MapService, ServiceError};
use mimir_wire as wire;

use crate::error::{ApiError, JsonBody};
use crate::state::AppState;

/// `GET /campaigns/{id}/display`
pub async fn get_display(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<wire::DisplayState>, ApiError> {
    let campaign = id.clone();
    state
        .with_db(move |conn| {
            CampaignService::new(conn)
                .get(&campaign)?
                .ok_or_else(|| ApiError::from(ServiceError::not_found("Campaign", &campaign)))
        })
        .await?;
    Ok(Json(state.live.display(&id)))
}

/// `PUT /campaigns/{id}/display`: show a map of the campaign (or none),
/// black or not.
pub async fn set_display(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(update): JsonBody<wire::DisplayUpdate>,
) -> Result<Json<wire::DisplayState>, ApiError> {
    let app_dir = state.config.data_dir.clone();
    let (campaign, map_id) = (id.clone(), update.map_id.clone());
    state
        .with_db(move |conn| {
            CampaignService::new(conn)
                .get(&campaign)?
                .ok_or_else(|| ApiError::from(ServiceError::not_found("Campaign", &campaign)))?;
            if let Some(map_id) = map_id {
                let map = MapService::new(conn, &app_dir)
                    .get(&map_id)?
                    .ok_or_else(|| ApiError::from(ServiceError::not_found("Map", &map_id)))?;
                if map.campaign_id != campaign {
                    return Err(ServiceError::validation("the map is not in this campaign").into());
                }
            }
            Ok(())
        })
        .await?;
    let display = wire::DisplayState {
        campaign_id: id,
        map_id: update.map_id,
        blackout: update.blackout,
        show_initiative: update.show_initiative,
    };
    state.live.set_display(display.clone());
    Ok(Json(display))
}
