//! The live socket (MIMIR-T-0712) on a real listener: auth, fan-out to
//! watchers, the display state, and the state on (re)connect. The role
//! filter for players is unit-tested in src/live.rs (player links arrive
//! with MIMIR-T-0716).

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use mimir_server::config::Config;
use mimir_server::state::AppState;
use mimir_wire::{self as wire, ClientMsg, MapPart, ServerMsg};
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

const TOKEN: &str = "live-test-token";

struct Server {
    addr: std::net::SocketAddr,
    _dir: tempfile::TempDir,
}

async fn server() -> Server {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir: dir.path().to_path_buf(),
        api_token: Some(TOKEN.into()),
        web_dist: None,
        seed_fixture: false,
    };
    mimir_server::prepare(&config).unwrap();
    let url = config.database_path().to_string_lossy().into_owned();
    let mut conn = mimir_core::db::create_connection(&url).unwrap();
    mimir_core::seed::seed_ui_fixture(&mut conn, &config.data_dir).unwrap();
    let router = mimir_server::router(AppState::new(config));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    Server { addr, _dir: dir }
}

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn connect(s: &Server, token: Option<&str>) -> Result<Socket, String> {
    let url = match token {
        Some(t) => format!("ws://{}/ws?access_token={t}", s.addr),
        None => format!("ws://{}/ws", s.addr),
    };
    connect_async(url)
        .await
        .map(|(ws, _)| ws)
        .map_err(|e| e.to_string())
}

async fn next(ws: &mut Socket) -> ServerMsg {
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .expect("a message within 5 s")
            .expect("open socket")
            .expect("no socket error");
        if let Message::Text(text) = msg {
            return serde_json::from_str(&text).unwrap();
        }
    }
}

async fn nothing_for(ws: &mut Socket, ms: u64) {
    if let Ok(Some(Ok(Message::Text(text)))) =
        tokio::time::timeout(Duration::from_millis(ms), ws.next()).await
    {
        panic!("unexpected message: {text}");
    }
}

async fn tell(ws: &mut Socket, msg: &ClientMsg) {
    ws.send(Message::Text(serde_json::to_string(msg).unwrap().into()))
        .await
        .unwrap();
}

/// An HTTP request with the DM token, over plain TCP (no HTTP client in
/// the dev-dependencies).
async fn http(
    s: &Server,
    method: &str,
    path: &str,
    body: Option<serde_json::Value>,
) -> serde_json::Value {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let body = body.map(|b| b.to_string()).unwrap_or_default();
    let req = format!(
        "{method} /api/v1{path} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {TOKEN}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        s.addr,
        body.len()
    );
    let mut stream = TcpStream::connect(s.addr).await.unwrap();
    stream.write_all(req.as_bytes()).await.unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.unwrap();
    let text = String::from_utf8(raw).unwrap();
    let (head, body) = text.split_once("\r\n\r\n").unwrap();
    assert!(head.starts_with("HTTP/1.1 2"), "{method} {path}: {head}");
    // Bodies here are small and not chunked (axum sets Content-Length).
    serde_json::from_str(body).unwrap_or(serde_json::Value::Null)
}

async fn fixture_ids(s: &Server) -> (String, String, String) {
    let campaigns = http(s, "GET", "/campaigns", None).await;
    let campaign = campaigns[0]["id"].as_str().unwrap().to_string();
    let modules = http(s, "GET", &format!("/campaigns/{campaign}/modules"), None).await;
    let module = modules[0]["id"].as_str().unwrap().to_string();
    let maps = http(s, "GET", &format!("/modules/{module}/maps"), None).await;
    let map = maps[0]["id"].as_str().unwrap().to_string();
    (campaign, module, map)
}

#[tokio::test]
async fn the_socket_needs_the_dm_token() {
    let s = server().await;
    let err = connect(&s, None).await.unwrap_err();
    assert!(err.contains("401"), "{err}");
    let err = connect(&s, Some("wrong")).await.unwrap_err();
    assert!(err.contains("401"), "{err}");
    let mut ws = connect(&s, Some(TOKEN)).await.unwrap();
    assert_eq!(
        next(&mut ws).await,
        ServerMsg::Hello {
            role: wire::Role::Dm
        }
    );
    tell(&mut ws, &ClientMsg::Ping).await;
    assert_eq!(next(&mut ws).await, ServerMsg::Pong);
    ws.send(Message::Text("{\"type\":\"dance\"}".into()))
        .await
        .unwrap();
    assert!(matches!(next(&mut ws).await, ServerMsg::Error { .. }));
}

#[tokio::test]
async fn changes_fan_out_to_the_watchers_of_the_campaign() {
    let s = server().await;
    let (campaign, _, map) = fixture_ids(&s).await;

    let mut a = connect(&s, Some(TOKEN)).await.unwrap();
    let mut b = connect(&s, Some(TOKEN)).await.unwrap();
    let mut other = connect(&s, Some(TOKEN)).await.unwrap();
    for ws in [&mut a, &mut b, &mut other] {
        assert!(matches!(next(ws).await, ServerMsg::Hello { .. }));
    }
    for ws in [&mut a, &mut b] {
        tell(
            ws,
            &ClientMsg::Watch {
                campaign_id: campaign.clone(),
            },
        )
        .await;
        // The state on watch: the display of the campaign.
        let ServerMsg::Display { display } = next(ws).await else {
            panic!("display first")
        };
        assert_eq!(display.campaign_id, campaign);
        assert_eq!(display.map_id, None);
    }
    tell(
        &mut other,
        &ClientMsg::Watch {
            campaign_id: "another-campaign".into(),
        },
    )
    .await;
    assert!(matches!(next(&mut other).await, ServerMsg::Display { .. }));

    // A REST change → a notice to each watcher of the campaign.
    let token = http(
        &s,
        "POST",
        &format!("/maps/{map}/tokens"),
        Some(json!({"grid_x": 1, "grid_y": 1, "label": "Robin"})),
    )
    .await;
    for ws in [&mut a, &mut b] {
        assert_eq!(
            next(ws).await,
            ServerMsg::MapChanged {
                map_id: map.clone(),
                part: MapPart::Tokens
            }
        );
    }
    http(
        &s,
        "PUT",
        &format!("/maps/{map}/fog"),
        Some(json!({"enabled": true})),
    )
    .await;
    http(
        &s,
        "POST",
        &format!("/maps/{map}/lights"),
        Some(json!({"grid_x": 2, "grid_y": 2, "preset": "torch"})),
    )
    .await;
    http(
        &s,
        "POST",
        &format!("/maps/{map}/pois"),
        Some(json!({"name": "Altar", "grid_x": 3, "grid_y": 3})),
    )
    .await;
    let expected: Vec<ServerMsg> = [MapPart::Fog, MapPart::Lights, MapPart::Markers]
        .map(|part| ServerMsg::MapChanged {
            map_id: map.clone(),
            part,
        })
        .to_vec();
    for ws in [&mut a, &mut b] {
        let got = vec![next(ws).await, next(ws).await, next(ws).await];
        assert_eq!(got, expected);
    }
    let id = token["id"].as_str().unwrap();
    http(&s, "DELETE", &format!("/tokens/{id}"), None).await;
    assert_eq!(
        next(&mut b).await,
        ServerMsg::MapChanged {
            map_id: map.clone(),
            part: MapPart::Tokens
        }
    );

    // Not for a socket that watches another campaign.
    nothing_for(&mut other, 300).await;
}

#[tokio::test]
async fn the_display_state_reaches_watchers_and_a_reconnect() {
    let s = server().await;
    let (campaign, _, map) = fixture_ids(&s).await;
    let mut ws = connect(&s, Some(TOKEN)).await.unwrap();
    next(&mut ws).await;
    tell(
        &mut ws,
        &ClientMsg::Watch {
            campaign_id: campaign.clone(),
        },
    )
    .await;
    next(&mut ws).await;

    let shown = http(
        &s,
        "PUT",
        &format!("/campaigns/{campaign}/display"),
        Some(json!({"map_id": map, "blackout": false})),
    )
    .await;
    assert_eq!(shown["map_id"], map);
    let ServerMsg::Display { display } = next(&mut ws).await else {
        panic!("display")
    };
    assert_eq!(display.map_id.as_deref(), Some(map.as_str()));

    // Reconnect: the state comes again on watch, with no request.
    drop(ws);
    let mut again = connect(&s, Some(TOKEN)).await.unwrap();
    next(&mut again).await;
    tell(
        &mut again,
        &ClientMsg::Watch {
            campaign_id: campaign.clone(),
        },
    )
    .await;
    let ServerMsg::Display { display } = next(&mut again).await else {
        panic!("display")
    };
    assert_eq!(display.map_id.as_deref(), Some(map.as_str()));
    assert!(!display.blackout);

    let got = http(&s, "GET", &format!("/campaigns/{campaign}/display"), None).await;
    assert_eq!(got["map_id"], map);
}

#[tokio::test]
async fn combat_changes_reach_the_dm() {
    let s = server().await;
    let (campaign, module, _) = fixture_ids(&s).await;
    let mut ws = connect(&s, Some(TOKEN)).await.unwrap();
    next(&mut ws).await;
    tell(
        &mut ws,
        &ClientMsg::Watch {
            campaign_id: campaign.clone(),
        },
    )
    .await;
    next(&mut ws).await;
    let combat = http(&s, "POST", &format!("/modules/{module}/combat"), None).await;
    assert_eq!(
        next(&mut ws).await,
        ServerMsg::CombatChanged {
            campaign_id: campaign.clone()
        }
    );
    let id = combat["session"]["id"].as_str().unwrap();
    http(
        &s,
        "POST",
        &format!("/combat/{id}/entries"),
        Some(json!({"kind": "custom", "name": "Ghost"})),
    )
    .await;
    assert_eq!(
        next(&mut ws).await,
        ServerMsg::CombatChanged {
            campaign_id: campaign
        }
    );
}

#[tokio::test]
async fn character_changes_reach_the_dm() {
    let s = server().await;
    let (campaign, _, _) = fixture_ids(&s).await;
    let pcs = http(&s, "GET", &format!("/campaigns/{campaign}/pcs"), None).await;
    let pc = pcs[0]["id"].as_str().unwrap().to_string();
    let mut ws = connect(&s, Some(TOKEN)).await.unwrap();
    next(&mut ws).await;
    tell(
        &mut ws,
        &ClientMsg::Watch {
            campaign_id: campaign.clone(),
        },
    )
    .await;
    next(&mut ws).await;
    http(
        &s,
        "PATCH",
        &format!("/characters/{pc}"),
        Some(json!({"bonds": "A promise"})),
    )
    .await;
    assert_eq!(
        next(&mut ws).await,
        ServerMsg::CharacterChanged {
            character_id: pc.clone()
        }
    );
    http(
        &s,
        "POST",
        &format!("/characters/{pc}/inventory"),
        Some(json!({"item_name": "Torch", "item_source": "PHB"})),
    )
    .await;
    assert_eq!(
        next(&mut ws).await,
        ServerMsg::CharacterChanged { character_id: pc }
    );
}
