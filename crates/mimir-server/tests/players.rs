//! Player access (MIMIR-T-0716, ADR MIMIR-A-0010): link tokens, the player
//! role on every route, the player's scope, revoke and reissue, and the
//! socket.

use std::time::Duration;

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use futures_util::{SinkExt, StreamExt};
use http_body_util::BodyExt;
use mimir_server::api::{routes, Access};
use mimir_server::config::Config;
use mimir_server::state::AppState;
use mimir_wire::{self as wire, ClientMsg, ServerMsg};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;
use tower::ServiceExt;

const DM: &str = "dm-token-for-tests";

struct F {
    router: axum::Router,
    _dir: tempfile::TempDir,
    campaign: String,
    map: String,
    pc: String,
    player: String,
}

async fn fixture() -> F {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir: dir.path().to_path_buf(),
        api_token: Some(DM.into()),
        web_dist: None,
        seed_fixture: false,
    };
    mimir_server::prepare(&config).unwrap();
    let url = config.database_path().to_string_lossy().into_owned();
    let mut conn = mimir_core::db::create_connection(&url).unwrap();
    mimir_core::seed::seed_ui_fixture(&mut conn, &config.data_dir).unwrap();
    let mut f = F {
        router: mimir_server::router(AppState::new(config)),
        _dir: dir,
        campaign: String::new(),
        map: String::new(),
        pc: String::new(),
        player: String::new(),
    };
    let c = f.json(Method::GET, "/campaigns", Some(DM), None).await;
    f.campaign = c[0]["id"].as_str().unwrap().into();
    let m = f
        .json(
            Method::GET,
            &format!("/campaigns/{}/modules", f.campaign),
            Some(DM),
            None,
        )
        .await;
    let module = m[0]["id"].as_str().unwrap().to_string();
    let maps = f
        .json(
            Method::GET,
            &format!("/modules/{module}/maps"),
            Some(DM),
            None,
        )
        .await;
    f.map = maps[0]["id"].as_str().unwrap().into();
    let pcs = f
        .json(
            Method::GET,
            &format!("/campaigns/{}/pcs", f.campaign),
            Some(DM),
            None,
        )
        .await;
    f.pc = pcs[0]["id"].as_str().unwrap().into();
    let link = f
        .json(
            Method::POST,
            &format!("/characters/{}/link", f.pc),
            Some(DM),
            None,
        )
        .await;
    f.player = link["token"].as_str().unwrap().into();
    assert_eq!(link["path"], format!("/play/{}", f.player));
    f
}

impl F {
    async fn send(
        &self,
        method: Method,
        path: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Vec<u8>) {
        let mut req = Request::builder()
            .method(method)
            .uri(format!("/api/v1{path}"));
        if let Some(t) = token {
            req = req.header(header::AUTHORIZATION, format!("Bearer {t}"));
        }
        let body = match body {
            Some(v) => {
                req = req.header(header::CONTENT_TYPE, "application/json");
                Body::from(v.to_string())
            }
            None => Body::empty(),
        };
        let res = self
            .router
            .clone()
            .oneshot(req.body(body).unwrap())
            .await
            .unwrap();
        let status = res.status();
        (
            status,
            res.into_body().collect().await.unwrap().to_bytes().to_vec(),
        )
    }

    async fn json(
        &self,
        method: Method,
        path: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> Value {
        let (status, bytes) = self.send(method.clone(), path, token, body).await;
        assert!(
            status.is_success(),
            "{method} {path}: {status} {}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    async fn status(&self, method: Method, path: &str, token: Option<&str>) -> StatusCode {
        self.send(method, path, token, None).await.0
    }

    async fn show(&self, map: Option<&str>, blackout: bool) {
        self.json(
            Method::PUT,
            &format!("/campaigns/{}/display", self.campaign),
            Some(DM),
            Some(json!({"map_id": map, "blackout": blackout})),
        )
        .await;
    }
}

#[tokio::test]
async fn a_player_token_reaches_only_player_routes() {
    let f = fixture().await;
    let session = f.json(Method::GET, "/session", Some(&f.player), None).await;
    assert_eq!(session["role"], "player");
    assert_eq!(session["character_id"], f.pc.as_str());
    assert_eq!(session["campaign_id"], f.campaign.as_str());

    for route in routes() {
        let path = fill(route.path);
        for method in route.methods {
            let m: Method = method.parse().unwrap();
            let status = f.status(m, &path, Some(&f.player)).await;
            match route.access {
                Access::Dm => assert_eq!(status, StatusCode::FORBIDDEN, "{method} {path}: DM only"),
                Access::Player => assert!(
                    status != StatusCode::UNAUTHORIZED,
                    "{method} {path}: a player route took the player token ({status})"
                ),
            }
        }
    }
}

#[tokio::test]
async fn a_player_sees_only_the_shown_map_of_their_campaign() {
    let f = fixture().await;
    let p = Some(f.player.as_str());
    let view = format!("/maps/{}/player-view", f.map);
    // Not shown: refused.
    for path in [
        view.clone(),
        format!("/maps/{}/image", f.map),
        format!("/maps/{}/geometry", f.map),
    ] {
        assert_eq!(
            f.status(Method::GET, &path, p).await,
            StatusCode::FORBIDDEN,
            "{path}"
        );
    }
    f.show(Some(&f.map), false).await;
    let v: wire::PlayerView =
        serde_json::from_value(f.json(Method::GET, &view, p, None).await).unwrap();
    assert_eq!(v.map.id, f.map);
    assert_eq!(
        f.status(Method::GET, &format!("/maps/{}/image", f.map), p)
            .await,
        StatusCode::OK
    );
    assert_eq!(
        f.status(Method::GET, &format!("/maps/{}/geometry", f.map), p)
            .await,
        StatusCode::OK
    );
    // Blackout: refused again.
    f.show(Some(&f.map), true).await;
    assert_eq!(f.status(Method::GET, &view, p).await, StatusCode::FORBIDDEN);
    f.show(Some(&f.map), false).await;

    // The display of their campaign, not another; they cannot change it.
    let d = f
        .json(
            Method::GET,
            &format!("/campaigns/{}/display", f.campaign),
            p,
            None,
        )
        .await;
    assert_eq!(d["map_id"], f.map.as_str());
    assert_eq!(
        f.status(Method::GET, "/campaigns/other/display", p).await,
        StatusCode::FORBIDDEN
    );
    let (status, _) = f
        .send(
            Method::PUT,
            &format!("/campaigns/{}/display", f.campaign),
            p,
            Some(json!({"map_id": null})),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Token art: not for a hidden token.
    let tokens = f
        .json(
            Method::GET,
            &format!("/maps/{}/tokens", f.map),
            Some(DM),
            None,
        )
        .await;
    let hidden = tokens
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["visible_to_players"] == false);
    if let Some(t) = hidden {
        let path = format!("/tokens/{}/image", t["id"].as_str().unwrap());
        assert_eq!(f.status(Method::GET, &path, p).await, StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn reissue_and_revoke_sign_the_device_out() {
    let f = fixture().await;
    let link = format!("/characters/{}/link", f.pc);
    let s = f.json(Method::GET, &link, Some(DM), None).await;
    assert_eq!(s["active"], true);

    let fresh = f.json(Method::POST, &link, Some(DM), None).await;
    let second = fresh["token"].as_str().unwrap().to_string();
    assert_eq!(
        f.status(Method::GET, "/session", Some(&f.player)).await,
        StatusCode::UNAUTHORIZED,
        "old link"
    );
    assert_eq!(
        f.status(Method::GET, "/session", Some(&second)).await,
        StatusCode::OK
    );

    assert_eq!(
        f.status(Method::DELETE, &link, Some(DM)).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        f.status(Method::GET, "/session", Some(&second)).await,
        StatusCode::UNAUTHORIZED
    );
    let s = f.json(Method::GET, &link, Some(DM), None).await;
    assert_eq!(s["active"], false);

    // An NPC gets no link.
    let npcs = f
        .json(
            Method::GET,
            &format!("/campaigns/{}/npcs", f.campaign),
            Some(DM),
            None,
        )
        .await;
    let npc = npcs[0]["id"].as_str().unwrap();
    let (status, _) = f
        .send(
            Method::POST,
            &format!("/characters/{npc}/link"),
            Some(DM),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// The next server message; `None` when the socket closed.
async fn next(ws: &mut Socket) -> Option<ServerMsg> {
    loop {
        let m = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .unwrap();
        match m {
            Some(Ok(Message::Text(t))) => return Some(serde_json::from_str(&t).unwrap()),
            Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return None,
            Some(Ok(_)) => continue,
        }
    }
}

#[tokio::test]
async fn the_player_socket_gets_the_view_and_is_closed_on_revoke() {
    let f = fixture().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = f.router.clone();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    f.show(Some(&f.map), false).await;
    let (mut ws, _) =
        tokio_tungstenite::connect_async(format!("ws://{addr}/ws?access_token={}", f.player))
            .await
            .unwrap();
    assert_eq!(
        next(&mut ws).await,
        Some(ServerMsg::Hello {
            role: wire::Role::Player
        })
    );
    // Their campaign without asking: the display, then the view.
    assert!(matches!(
        next(&mut ws).await,
        Some(ServerMsg::Display { .. })
    ));
    match next(&mut ws).await {
        Some(ServerMsg::PlayerView { view }) => assert_eq!(view.map.id, f.map),
        other => panic!("expected the player view, got {other:?}"),
    }
    // A change on the shown map: a new view.
    f.json(
        Method::POST,
        &format!("/maps/{}/pois", f.map),
        Some(DM),
        Some(json!({"name": "Altar", "grid_x": 1, "grid_y": 1, "visible": true})),
    )
    .await;
    assert!(matches!(
        next(&mut ws).await,
        Some(ServerMsg::PlayerView { .. })
    ));
    // Revoke: signed out and closed.
    f.status(
        Method::DELETE,
        &format!("/characters/{}/link", f.pc),
        Some(DM),
    )
    .await;
    assert_eq!(next(&mut ws).await, Some(ServerMsg::SignedOut));
    assert_eq!(next(&mut ws).await, None, "closed");
    let _ = ws
        .send(Message::Text(
            serde_json::to_string(&ClientMsg::Ping).unwrap().into(),
        ))
        .await;
    // Reconnecting with the revoked token fails.
    let again =
        tokio_tungstenite::connect_async(format!("ws://{addr}/ws?access_token={}", f.player)).await;
    assert!(again.unwrap_err().to_string().contains("401"));
}

/// A route path with each `{param}` filled with "x".
fn fill(path: &str) -> String {
    path.split('/')
        .map(|seg| if seg.starts_with('{') { "x" } else { seg })
        .collect::<Vec<_>>()
        .join("/")
}
