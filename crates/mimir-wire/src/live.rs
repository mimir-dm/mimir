//! Live events over the WebSocket `/ws` (MIMIR-T-0712).
//!
//! One socket per client. The client sends [`ClientMsg`]; the server sends
//! [`ServerMsg`], each a JSON text frame tagged by `type`. Auth is the API's:
//! the DM token (a browser sends it as `?access_token=`). The server sends
//! each client only what its role may see:
//!
//! - the DM gets change notices ([`ServerMsg::MapChanged`] and the like) and
//!   reads the changed part from the REST API;
//! - a player gets the player view of the shown map ([`ServerMsg::PlayerView`])
//!   whenever it changes, the display state, and notices about their own
//!   character.
//!
//! On connect and on `watch` the server sends the current state, so a client
//! that reconnects needs nothing else (the desktop app's `request-state`).

use serde::{Deserialize, Serialize};

use crate::map::PlayerView;
use crate::Role;

/// From the client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    /// Follow a campaign (the DM; a player follows their own campaign).
    Watch { campaign_id: String },
    /// Keep-alive; the server answers `pong`.
    Ping,
}

/// What the player display shows for a campaign.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DisplayState {
    pub campaign_id: String,
    /// The map on the player display; `None`: no map.
    pub map_id: Option<String>,
    /// The display is paused (black).
    pub blackout: bool,
}

/// `PUT /campaigns/{id}/display`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DisplayUpdate {
    #[serde(default)]
    pub map_id: Option<String>,
    #[serde(default)]
    pub blackout: bool,
}

/// The part of a map that changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapPart {
    Map,
    Tokens,
    Fog,
    Lights,
    Markers,
}

/// From the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    /// First message: who the server thinks the client is.
    Hello {
        role: Role,
    },
    /// The display state of the watched campaign (on watch, and on change).
    Display {
        display: DisplayState,
    },
    /// DM: a part of a map changed; read it again.
    MapChanged {
        map_id: String,
        part: MapPart,
    },
    /// Player: the player view of the shown map (on watch and on change).
    PlayerView {
        view: Box<PlayerView>,
    },
    /// DM: the combat of the campaign changed.
    CombatChanged {
        campaign_id: String,
    },
    /// A character changed (the DM: any; a player: their own).
    CharacterChanged {
        character_id: String,
    },
    /// The client missed events; it gets the current state next.
    Resync,
    Pong,
    /// A message the server could not take.
    Error {
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_tagged_by_type() {
        let m = ServerMsg::MapChanged {
            map_id: "m1".into(),
            part: MapPart::Tokens,
        };
        assert_eq!(
            serde_json::to_string(&m).unwrap(),
            r#"{"type":"map_changed","map_id":"m1","part":"tokens"}"#
        );
        let c: ClientMsg = serde_json::from_str(r#"{"type":"watch","campaign_id":"c1"}"#).unwrap();
        assert_eq!(
            c,
            ClientMsg::Watch {
                campaign_id: "c1".into()
            }
        );
        let d = ServerMsg::Display {
            display: DisplayState {
                campaign_id: "c1".into(),
                map_id: None,
                blackout: true,
            },
        };
        let back: ServerMsg = serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
        assert_eq!(back, d);
        assert_eq!(
            serde_json::to_string(&ServerMsg::Pong).unwrap(),
            r#"{"type":"pong"}"#
        );
    }
}
