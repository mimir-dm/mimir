//! Live events (MIMIR-T-0712): one WebSocket per client at `/ws`.
//!
//! Handlers that change something publish a [`Change`] on the [`Hub`]. Each
//! socket task receives every change and [`decide`]s, from the client's role
//! and the campaign it watches, what to send: the DM gets change notices, a
//! player gets the player view of the shown map. The display state (which
//! map the player display shows, blackout) lives here, in memory: a restart
//! clears it.

use std::collections::HashMap;
use std::sync::Mutex;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Request, State};
use axum::http::{header, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use axum::Extension;

use crate::auth::Caller;
use mimir_wire::{ClientMsg, DisplayState, MapPart, Role, ServerMsg};
use tokio::sync::broadcast;

use crate::api::maps::build_player_view;
use crate::state::AppState;

/// Something changed.
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    Map {
        campaign_id: String,
        map_id: String,
        part: MapPart,
    },
    Display(DisplayState),
    Combat {
        campaign_id: String,
    },
    Character {
        campaign_id: Option<String>,
        character_id: String,
    },
    /// The link of a player character was reissued or revoked.
    SignedOut {
        character_id: String,
    },
}

impl Change {
    fn campaign_id(&self) -> Option<&str> {
        match self {
            Change::Map { campaign_id, .. } | Change::Combat { campaign_id } => Some(campaign_id),
            Change::Display(d) => Some(&d.campaign_id),
            Change::Character { campaign_id, .. } => campaign_id.as_deref(),
            Change::SignedOut { .. } => None,
        }
    }
}

/// Who is on the other end of a socket.
#[derive(Debug, Clone, PartialEq)]
pub enum Viewer {
    Dm,
    /// A player of one campaign, with their character (player links,
    /// MIMIR-T-0716).
    Player {
        campaign_id: String,
        character_id: Option<String>,
    },
}

impl Viewer {
    fn role(&self) -> Role {
        match self {
            Viewer::Dm => Role::Dm,
            Viewer::Player { .. } => Role::Player,
        }
    }
}

/// What a socket task does.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Send(ServerMsg),
    /// Build the player view of this map and send it.
    SendPlayerView(String),
    /// Tell the player their link ended, and close.
    SignOut,
}

/// What to send to `viewer`, who watches `watched`, for `change`. `display`
/// is the display state of the watched campaign.
pub fn decide(
    viewer: &Viewer,
    watched: Option<&str>,
    display: &DisplayState,
    change: &Change,
) -> Vec<Action> {
    // A player whose link ended is signed out, whatever they watch.
    if let (
        Viewer::Player {
            character_id: Some(mine),
            ..
        },
        Change::SignedOut { character_id },
    ) = (viewer, change)
    {
        return if mine == character_id {
            vec![Action::SignOut]
        } else {
            Vec::new()
        };
    }
    let Some(watched) = watched else {
        return Vec::new();
    };
    if change.campaign_id() != Some(watched) {
        return Vec::new();
    }
    match viewer {
        Viewer::Dm => vec![Action::Send(match change.clone() {
            Change::Map { map_id, part, .. } => ServerMsg::MapChanged { map_id, part },
            Change::Display(display) => ServerMsg::Display { display },
            Change::Combat { campaign_id } => ServerMsg::CombatChanged { campaign_id },
            Change::Character { character_id, .. } => ServerMsg::CharacterChanged { character_id },
            Change::SignedOut { .. } => return Vec::new(),
        })],
        Viewer::Player { character_id, .. } => match change {
            Change::Map { map_id, .. } => shown_view(display)
                .filter(|shown| shown == map_id)
                .map(|m| vec![Action::SendPlayerView(m.to_string())])
                .unwrap_or_default(),
            Change::Display(d) => {
                let mut out = vec![Action::Send(ServerMsg::Display { display: d.clone() })];
                out.extend(shown_view(d).map(|m| Action::SendPlayerView(m.to_string())));
                out
            }
            // The turn order is part of the player view, when shown.
            Change::Combat { .. } => match shown_view(display) {
                Some(m) if display.show_initiative => vec![Action::SendPlayerView(m.to_string())],
                _ => Vec::new(),
            },
            Change::Character {
                character_id: changed,
                ..
            } if character_id.as_ref() == Some(changed) => {
                vec![Action::Send(ServerMsg::CharacterChanged {
                    character_id: changed.clone(),
                })]
            }
            Change::Character { .. } | Change::SignedOut { .. } => Vec::new(),
        },
    }
}

/// The map players see now: the shown map, unless the display is black.
fn shown_view(display: &DisplayState) -> Option<&str> {
    if display.blackout {
        None
    } else {
        display.map_id.as_deref()
    }
}

/// The current state, for a client that starts watching a campaign.
pub fn snapshot(viewer: &Viewer, display: &DisplayState) -> Vec<Action> {
    let mut out = vec![Action::Send(ServerMsg::Display {
        display: display.clone(),
    })];
    if matches!(viewer, Viewer::Player { .. }) {
        out.extend(shown_view(display).map(|m| Action::SendPlayerView(m.to_string())));
    }
    out
}

/// The broadcast of changes, and the display states.
pub struct Hub {
    tx: broadcast::Sender<Change>,
    displays: Mutex<HashMap<String, DisplayState>>,
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

impl Hub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            tx,
            displays: Mutex::new(HashMap::new()),
        }
    }

    /// Send a change to every socket. No socket: nothing happens.
    pub fn publish(&self, change: Change) {
        let _ = self.tx.send(change);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Change> {
        self.tx.subscribe()
    }

    /// The display state of a campaign (no map, not black, by default).
    pub fn display(&self, campaign_id: &str) -> DisplayState {
        self.displays
            .lock()
            .expect("display lock")
            .get(campaign_id)
            .cloned()
            .unwrap_or_else(|| DisplayState {
                campaign_id: campaign_id.to_string(),
                ..DisplayState::default()
            })
    }

    /// Set the display state of a campaign and tell the sockets.
    pub fn set_display(&self, display: DisplayState) {
        self.displays
            .lock()
            .expect("display lock")
            .insert(display.campaign_id.clone(), display.clone());
        self.publish(Change::Display(display));
    }
}

/// `GET /ws`: upgrade to the live socket (the DM auth layer runs first).
pub async fn ws(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    upgrade: WebSocketUpgrade,
) -> Response {
    let viewer = match caller {
        Caller::Dm => Viewer::Dm,
        Caller::Player(p) => Viewer::Player {
            campaign_id: p.campaign_id,
            character_id: Some(p.character_id),
        },
    };
    upgrade.on_upgrade(move |socket| run(socket, state, viewer))
}

async fn run(mut socket: WebSocket, state: AppState, viewer: Viewer) {
    let mut changes = state.live.subscribe();
    let mut watched: Option<String> = match &viewer {
        Viewer::Player { campaign_id, .. } => Some(campaign_id.clone()),
        Viewer::Dm => None,
    };
    if send(
        &mut socket,
        &ServerMsg::Hello {
            role: viewer.role(),
        },
    )
    .await
    .is_err()
    {
        return;
    }
    if let Some(c) = &watched {
        let actions = snapshot(&viewer, &state.live.display(c));
        if perform(&mut socket, &state, actions).await.is_err() {
            return;
        }
    }
    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let text = match incoming {
                    Some(Ok(Message::Text(text))) => text,
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(_)) => continue,
                };
                let actions = match serde_json::from_str::<ClientMsg>(&text) {
                    Ok(ClientMsg::Ping) => vec![Action::Send(ServerMsg::Pong)],
                    Ok(ClientMsg::Watch { campaign_id }) => match viewer {
                        Viewer::Dm => {
                            let display = state.live.display(&campaign_id);
                            watched = Some(campaign_id);
                            snapshot(&viewer, &display)
                        }
                        Viewer::Player { .. } => vec![Action::Send(ServerMsg::Error {
                            message: "a player watches their own campaign".into(),
                        })],
                    },
                    Err(e) => vec![Action::Send(ServerMsg::Error { message: format!("bad message: {e}") })],
                };
                if perform(&mut socket, &state, actions).await.is_err() {
                    break;
                }
            }
            change = changes.recv() => {
                let actions = match change {
                    Ok(change) => {
                        let display = watched
                            .as_deref()
                            .map(|c| state.live.display(c))
                            .unwrap_or_default();
                        decide(&viewer, watched.as_deref(), &display, &change)
                    }
                    // Too slow: missed changes. Start again from the state now.
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        let mut out = vec![Action::Send(ServerMsg::Resync)];
                        if let Some(c) = &watched {
                            out.extend(snapshot(&viewer, &state.live.display(c)));
                        }
                        out
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                if perform(&mut socket, &state, actions).await.is_err() {
                    break;
                }
            }
        }
    }
}

async fn perform(socket: &mut WebSocket, state: &AppState, actions: Vec<Action>) -> Result<(), ()> {
    for action in actions {
        let msg = match action {
            Action::Send(msg) => msg,
            Action::SignOut => {
                let _ = send(socket, &ServerMsg::SignedOut).await;
                let _ = socket.send(Message::Close(None)).await;
                // An error ends the socket loop.
                return Err(());
            }
            Action::SendPlayerView(map_id) => {
                let app_dir = state.config.data_dir.clone();
                let live = state.live.clone();
                match state
                    .with_db(move |conn| {
                        let campaign = crate::api::maps::campaign_of(conn, &app_dir, &map_id)?;
                        let show = live.display(&campaign).show_initiative;
                        build_player_view(conn, &app_dir, &map_id, show)
                    })
                    .await
                {
                    Ok(view) => ServerMsg::PlayerView {
                        view: Box::new(view),
                    },
                    Err(e) => ServerMsg::Error { message: e.message },
                }
            }
        };
        send(socket, &msg).await?;
    }
    Ok(())
}

async fn send(socket: &mut WebSocket, msg: &ServerMsg) -> Result<(), ()> {
    let text = serde_json::to_string(msg).map_err(|_| ())?;
    socket
        .send(Message::Text(text.into()))
        .await
        .map_err(|_| ())
}

/// A browser cannot set headers on a WebSocket: copy `?access_token=` into
/// `Authorization` (when there is none) before the auth layer, as Kairos
/// does. Only the `/ws` route has this layer.
pub async fn promote_query_token(mut req: Request, next: Next) -> Response {
    if !req.headers().contains_key(header::AUTHORIZATION) {
        let token = req
            .uri()
            .query()
            .and_then(|q| q.split('&').find_map(|p| p.strip_prefix("access_token=")))
            .filter(|t| !t.is_empty())
            .map(str::to_string);
        if let Some(value) = token.and_then(|t| HeaderValue::from_str(&format!("Bearer {t}")).ok())
        {
            req.headers_mut().insert(header::AUTHORIZATION, value);
        }
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn display(map: Option<&str>, blackout: bool) -> DisplayState {
        DisplayState {
            campaign_id: "c1".into(),
            map_id: map.map(String::from),
            blackout,
            show_initiative: false,
        }
    }

    fn map_change(campaign: &str, map: &str) -> Change {
        Change::Map {
            campaign_id: campaign.into(),
            map_id: map.into(),
            part: MapPart::Tokens,
        }
    }

    fn player(character: Option<&str>) -> Viewer {
        Viewer::Player {
            campaign_id: "c1".into(),
            character_id: character.map(String::from),
        }
    }

    #[test]
    fn nothing_before_watch_or_for_another_campaign() {
        let d = display(Some("m1"), false);
        assert!(decide(&Viewer::Dm, None, &d, &map_change("c1", "m1")).is_empty());
        assert!(decide(&Viewer::Dm, Some("c2"), &d, &map_change("c1", "m1")).is_empty());
        assert!(decide(&player(None), Some("c1"), &d, &map_change("c2", "m1")).is_empty());
    }

    #[test]
    fn the_dm_gets_notices_for_every_map() {
        let d = display(Some("m1"), false);
        assert_eq!(
            decide(&Viewer::Dm, Some("c1"), &d, &map_change("c1", "m2")),
            vec![Action::Send(ServerMsg::MapChanged {
                map_id: "m2".into(),
                part: MapPart::Tokens
            })]
        );
        assert_eq!(
            decide(
                &Viewer::Dm,
                Some("c1"),
                &d,
                &Change::Combat {
                    campaign_id: "c1".into()
                }
            ),
            vec![Action::Send(ServerMsg::CombatChanged {
                campaign_id: "c1".into()
            })]
        );
    }

    #[test]
    fn a_player_gets_the_view_of_the_shown_map_only() {
        let p = player(None);
        assert_eq!(
            decide(
                &p,
                Some("c1"),
                &display(Some("m1"), false),
                &map_change("c1", "m1")
            ),
            vec![Action::SendPlayerView("m1".into())]
        );
        assert!(
            decide(
                &p,
                Some("c1"),
                &display(Some("m1"), false),
                &map_change("c1", "m2")
            )
            .is_empty(),
            "a map that is not shown"
        );
        assert!(
            decide(
                &p,
                Some("c1"),
                &display(Some("m1"), true),
                &map_change("c1", "m1")
            )
            .is_empty(),
            "blackout"
        );
        assert!(decide(
            &p,
            Some("c1"),
            &display(None, false),
            &map_change("c1", "m1")
        )
        .is_empty());
        assert!(
            decide(
                &p,
                Some("c1"),
                &display(Some("m1"), false),
                &Change::Combat {
                    campaign_id: "c1".into()
                }
            )
            .is_empty(),
            "no DM combat notices"
        );
    }

    #[test]
    fn a_player_hears_of_their_own_character_only() {
        let p = player(Some("pc1"));
        let d = display(None, false);
        let change = |id: &str| Change::Character {
            campaign_id: Some("c1".into()),
            character_id: id.into(),
        };
        assert_eq!(
            decide(&p, Some("c1"), &d, &change("pc1")),
            vec![Action::Send(ServerMsg::CharacterChanged {
                character_id: "pc1".into()
            })]
        );
        assert!(decide(&p, Some("c1"), &d, &change("pc2")).is_empty());
        assert_eq!(decide(&Viewer::Dm, Some("c1"), &d, &change("pc2")).len(), 1);
    }

    #[test]
    fn a_display_change_brings_the_view_for_players() {
        let shown = display(Some("m1"), false);
        assert_eq!(
            decide(
                &player(None),
                Some("c1"),
                &shown,
                &Change::Display(shown.clone())
            ),
            vec![
                Action::Send(ServerMsg::Display {
                    display: shown.clone()
                }),
                Action::SendPlayerView("m1".into())
            ]
        );
        let black = display(Some("m1"), true);
        assert_eq!(
            decide(
                &player(None),
                Some("c1"),
                &black,
                &Change::Display(black.clone())
            ),
            vec![Action::Send(ServerMsg::Display {
                display: black.clone()
            })]
        );
        assert_eq!(
            snapshot(&Viewer::Dm, &shown),
            vec![Action::Send(ServerMsg::Display {
                display: shown.clone()
            })]
        );
        assert_eq!(snapshot(&player(None), &shown).len(), 2);
    }
}
