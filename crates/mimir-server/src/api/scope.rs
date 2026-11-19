//! What a player may reach (ADR MIMIR-A-0010): their campaign, the map the
//! player display shows (not in blackout), and their character. The DM
//! reaches all.

use std::path::Path;

use diesel::SqliteConnection;
use mimir_core::services::{CharacterService, MapService, PartyService};

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

/// Read a character: the DM any; a player their own, or another player
/// character of their campaign whose sheet is shared.
pub fn read_character(
    caller: &Caller,
    conn: &mut SqliteConnection,
    character_id: &str,
) -> Result<(), ApiError> {
    let Caller::Player(p) = caller else {
        return Ok(());
    };
    if p.character_id == character_id {
        return Ok(());
    }
    let same_campaign = CharacterService::new(conn)
        .get(character_id)?
        .is_some_and(|c| c.is_npc == 0 && c.campaign_id.as_deref() == Some(p.campaign_id.as_str()));
    if same_campaign && PartyService::new(conn).is_shared(character_id)? {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}

/// Write notes: the character's own player only (the DM reads them).
pub fn own_player(caller: &Caller, character_id: &str) -> Result<(), ApiError> {
    match caller {
        Caller::Player(p) if p.character_id == character_id => Ok(()),
        _ => Err(ApiError::forbidden()),
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
