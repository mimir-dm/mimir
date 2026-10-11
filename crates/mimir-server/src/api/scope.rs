//! What a player may reach (ADR MIMIR-A-0010): their campaign, the map the
//! player display shows (not in blackout), and their character. The DM
//! reaches all.

use std::path::Path;

use diesel::SqliteConnection;
use mimir_core::services::MapService;

use crate::auth::Caller;
use crate::error::ApiError;
use crate::live::Hub;

/// The DM only (for a method that a player route path also has).
pub fn dm(caller: &Caller) -> Result<(), ApiError> {
    match caller {
        Caller::Dm => Ok(()),
        Caller::Player(_) => Err(ApiError::forbidden()),
    }
}

/// The caller's own campaign.
pub fn campaign(caller: &Caller, campaign_id: &str) -> Result<(), ApiError> {
    match caller {
        Caller::Dm => Ok(()),
        Caller::Player(p) if p.campaign_id == campaign_id => Ok(()),
        Caller::Player(_) => Err(ApiError::forbidden()),
    }
}

/// The caller's own character (a player), or any (the DM).
pub fn character(caller: &Caller, character_id: &str) -> Result<(), ApiError> {
    match caller {
        Caller::Dm => Ok(()),
        Caller::Player(p) if p.character_id == character_id => Ok(()),
        Caller::Player(_) => Err(ApiError::forbidden()),
    }
}

/// A map the player display of the caller's campaign shows now.
pub fn shown_map(
    caller: &Caller,
    live: &Hub,
    conn: &mut SqliteConnection,
    app_dir: &Path,
    map_id: &str,
) -> Result<(), ApiError> {
    let Caller::Player(p) = caller else {
        return Ok(());
    };
    let shown = live.display(&p.campaign_id);
    let on_display = !shown.blackout && shown.map_id.as_deref() == Some(map_id);
    let in_campaign = MapService::new(conn, app_dir)
        .get(map_id)?
        .is_some_and(|m| m.campaign_id == p.campaign_id);
    if on_display && in_campaign {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}
