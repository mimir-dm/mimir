//! The map API (MIMIR-T-0711) on the fixture maps, through the real router.
//! Each test seeds its own data dir: the tests write.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use mimir_wire as wire;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tower::ServiceExt;

struct Fixture {
    router: axum::Router,
    dir: tempfile::TempDir,
    /// The module map ("Goblin Hideout").
    map: String,
}

async fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir: dir.path().to_path_buf(),
        api_token: None,
        web_dist: None,
        seed_fixture: false,
    };
    mimir_server::prepare(&config).unwrap();
    let url = config.database_path().to_string_lossy().into_owned();
    let mut conn = mimir_core::db::create_connection(&url).unwrap();
    mimir_core::seed::seed_ui_fixture(&mut conn, &config.data_dir).unwrap();
    let router = mimir_server::router(AppState::new(config));
    let mut f = Fixture {
        router,
        dir,
        map: String::new(),
    };
    let campaigns: Vec<wire::CampaignSummary> = f.ok(Method::GET, "/campaigns", None).await;
    let modules: Vec<wire::ModuleSummary> = f
        .ok(
            Method::GET,
            &format!("/campaigns/{}/modules", campaigns[0].id),
            None,
        )
        .await;
    let maps: Vec<wire::MapSummary> = f
        .ok(
            Method::GET,
            &format!("/modules/{}/maps", modules[0].id),
            None,
        )
        .await;
    f.map = maps[0].id.clone();
    f
}

impl Fixture {
    async fn send(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> (StatusCode, Vec<u8>, String) {
        let mut req = Request::builder()
            .method(method)
            .uri(format!("/api/v1{path}"));
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
        let ctype = res
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default();
        let bytes = res.into_body().collect().await.unwrap().to_bytes().to_vec();
        (status, bytes, ctype)
    }

    async fn ok<T: DeserializeOwned>(&self, method: Method, path: &str, body: Option<Value>) -> T {
        let (status, bytes, _) = self.send(method.clone(), path, body).await;
        assert!(
            status.is_success(),
            "{method} {path}: {status} {}",
            String::from_utf8_lossy(&bytes)
        );
        serde_json::from_slice(&bytes).unwrap()
    }

    async fn status(&self, method: Method, path: &str, body: Option<Value>) -> (StatusCode, Value) {
        let (status, bytes, _) = self.send(method, path, body).await;
        let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, json)
    }
}

#[tokio::test]
async fn map_detail_geometry_and_image() {
    let f = fixture().await;
    let map: wire::MapDetail = f.ok(Method::GET, &format!("/maps/{}", f.map), None).await;
    assert_eq!(map.name, "Goblin Hideout");
    assert!(map.grid_size_px > 0.0 && map.width_px > 0 && map.height_px > 0);
    assert_eq!(
        map.width_px,
        (map.grid_size_px * map.columns).round() as i32
    );

    let geo: wire::MapGeometry = f
        .ok(Method::GET, &format!("/maps/{}/geometry", f.map), None)
        .await;
    assert!(!geo.walls.is_empty(), "the fixture map has walls");
    assert_eq!(geo.grid_size_px, map.grid_size_px);
    assert_eq!((geo.columns, geo.rows), (map.columns, map.rows));

    let (status, bytes, ctype) = f
        .send(Method::GET, &format!("/maps/{}/image", f.map), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ctype, "image/jpeg");
    assert_eq!(&bytes[..2], &[0xFF, 0xD8], "a JPEG");

    let (status, body) = f.status(Method::GET, "/maps/nope", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["entity"], "Map");
}

#[tokio::test]
async fn tokens_create_move_hide_vision_delete() {
    let f = fixture().await;
    let base = format!("/maps/{}/tokens", f.map);
    let before: Vec<wire::Token> = f.ok(Method::GET, &base, None).await;
    assert!(!before.is_empty(), "the fixture places tokens");

    // A PC token needs a label.
    let (status, body) = f
        .status(Method::POST, &base, Some(json!({"grid_x": 1, "grid_y": 1})))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation");

    let (status, body) = f
        .status(
            Method::POST,
            &base,
            Some(json!({"grid_x": 2, "grid_y": 3, "label": "Robin"})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let t: wire::Token = serde_json::from_value(body).unwrap();
    assert_eq!(
        (t.name.as_str(), t.token_type.as_str(), t.grid_x, t.grid_y),
        ("Robin", "pc", 2, 3)
    );
    assert!(t.visible_to_players);
    assert_eq!(
        t.x,
        (2.0 + 0.5) * {
            let m: wire::MapDetail = f.ok(Method::GET, &format!("/maps/{}", f.map), None).await;
            m.grid_size_px
        }
    );

    let moved: wire::Token = f
        .ok(
            Method::PATCH,
            &format!("/tokens/{}", t.id),
            Some(
                json!({"grid_x": 5, "hidden": true, "vision_dark_ft": 60, "vision_bright_ft": 30}),
            ),
        )
        .await;
    assert_eq!((moved.grid_x, moved.grid_y), (5, 3), "absent grid_y stays");
    assert!(!moved.visible_to_players);
    assert_eq!(
        (moved.vision_dark_ft, moved.vision_bright_ft),
        (60, Some(30))
    );
    let cleared: wire::Token = f
        .ok(
            Method::PATCH,
            &format!("/tokens/{}", t.id),
            Some(json!({"vision_bright_ft": null})),
        )
        .await;
    assert_eq!(cleared.vision_bright_ft, None, "null clears");

    let after: Vec<wire::Token> = f.ok(Method::GET, &base, None).await;
    assert_eq!(after.len(), before.len() + 1);

    let (status, _, _) = f
        .send(Method::DELETE, &format!("/tokens/{}", t.id), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = f
        .status(
            Method::PATCH,
            &format!("/tokens/{}", t.id),
            Some(json!({"grid_x": 1})),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // A body that is not JSON for the type: 400 with the envelope.
    let (status, body) = f
        .status(Method::POST, &base, Some(json!({"grid_x": "two"})))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "bad_request");
}

#[tokio::test]
async fn token_images() {
    let f = fixture().await;
    let tokens: Vec<wire::Token> = f
        .ok(Method::GET, &format!("/maps/{}/tokens", f.map), None)
        .await;
    // A token of a catalog monster, and that monster's source and name.
    let map: wire::MapDetail = f.ok(Method::GET, &format!("/maps/{}", f.map), None).await;
    let module = map.module_id.clone().expect("a module map");
    let monsters: Vec<wire::ModuleMonsterSummary> = f
        .ok(Method::GET, &format!("/modules/{module}/monsters"), None)
        .await;
    let (monster, entry) = tokens
        .iter()
        .find_map(|t| {
            let m = monsters
                .iter()
                .find(|m| Some(&m.id) == t.monster_id.as_ref() && m.monster_source.is_some())?;
            Some((t, m))
        })
        .expect("a token of a catalog monster");
    let source = entry.monster_source.clone().unwrap();
    let name = entry.monster_name.clone().unwrap();
    let path = format!("/tokens/{}/image", monster.id);
    // No token art in the fixture data dir.
    let (status, _) = f.status(Method::GET, &path, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let art = f
        .dir
        .path()
        .join("assets/catalog/bestiary/tokens")
        .join(&source);
    std::fs::create_dir_all(&art).unwrap();
    let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    std::fs::write(art.join(format!("{name}.png")), png).unwrap();
    let (status, bytes, ctype) = f.send(Method::GET, &path, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ctype, "image/png");
    assert_eq!(bytes, png);

    let (status, _) = f.status(Method::GET, "/tokens/nope/image", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn fog_on_reveal_and_reset() {
    let f = fixture().await;
    let base = format!("/maps/{}/fog", f.map);
    let fog: wire::Fog = f
        .ok(Method::PUT, &base, Some(json!({"enabled": true})))
        .await;
    assert!(fog.enabled);
    assert!(fog.revealed.is_empty());

    let fog: wire::Fog = f
        .ok(
            Method::POST,
            &format!("{base}/reveal"),
            Some(json!({"shape": "rect", "x": 10, "y": 20, "width": 100, "height": 50})),
        )
        .await;
    assert_eq!(fog.revealed.len(), 1);
    assert_eq!(fog.revealed[0].width, 100.0);
    let fog: wire::Fog = f
        .ok(
            Method::POST,
            &format!("{base}/reveal"),
            Some(json!({"shape": "circle", "center_x": 200, "center_y": 200, "radius": 50})),
        )
        .await;
    assert_eq!(fog.revealed.len(), 2);
    assert_eq!(fog.revealed[1].x, 150.0, "a circle is kept as its box");
    let map: wire::MapDetail = f.ok(Method::GET, &format!("/maps/{}", f.map), None).await;
    let fog: wire::Fog = f
        .ok(
            Method::POST,
            &format!("{base}/reveal"),
            Some(json!({"shape": "all"})),
        )
        .await;
    assert_eq!(fog.revealed[2].width, f64::from(map.width_px));

    let (status, _, _) = f
        .send(
            Method::DELETE,
            &format!("{base}/revealed/{}", fog.revealed[0].id),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let fog: wire::Fog = f.ok(Method::GET, &base, None).await;
    assert_eq!(fog.revealed.len(), 2);
    let (status, _) = f
        .status(Method::DELETE, &format!("{base}/revealed/nope"), None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let fog: wire::Fog = f
        .ok(Method::DELETE, &format!("{base}/revealed"), None)
        .await;
    assert!(fog.revealed.is_empty());
    let fog: wire::Fog = f
        .ok(Method::PUT, &base, Some(json!({"enabled": false})))
        .await;
    assert!(!fog.enabled);
}

#[tokio::test]
async fn lights_presets_patch_delete() {
    let f = fixture().await;
    let base = format!("/maps/{}/lights", f.map);
    let before: Vec<wire::Light> = f.ok(Method::GET, &base, None).await;

    let torch: wire::Light = f
        .ok(
            Method::POST,
            &base,
            Some(json!({"grid_x": 4, "grid_y": 4, "preset": "torch"})),
        )
        .await;
    assert_eq!((torch.bright_radius_ft, torch.dim_radius_ft), (20, 40));
    assert!(torch.active);
    let (status, _) = f
        .status(
            Method::POST,
            &base,
            Some(json!({"grid_x": 1, "grid_y": 1, "preset": "sun"})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let custom: wire::Light = f
        .ok(
            Method::POST,
            &base,
            Some(json!({"grid_x": 1, "grid_y": 2, "name": "Brazier", "bright_radius_ft": 10, "dim_radius_ft": 20, "color": "#ff8800"})),
        )
        .await;
    assert_eq!(custom.name.as_deref(), Some("Brazier"));

    let patched: wire::Light = f
        .ok(
            Method::PATCH,
            &format!("/lights/{}", torch.id),
            Some(json!({"grid_y": 9, "active": false, "dim_radius_ft": 50})),
        )
        .await;
    assert_eq!((patched.grid_x, patched.grid_y), (4, 9));
    assert!(!patched.active);
    assert_eq!(patched.dim_radius_ft, 50);

    let (status, _, _) = f
        .send(Method::DELETE, &format!("/lights/{}", custom.id), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let after: Vec<wire::Light> = f.ok(Method::GET, &base, None).await;
    assert_eq!(after.len(), before.len() + 1);
    let (status, _, _) = f.send(Method::DELETE, &base, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let none: Vec<wire::Light> = f.ok(Method::GET, &base, None).await;
    assert!(none.is_empty());
}

#[tokio::test]
async fn traps_and_pois() {
    let f = fixture().await;
    let trap: wire::Trap = f
        .ok(
            Method::POST,
            &format!("/maps/{}/traps", f.map),
            Some(json!({"name": "Pit", "grid_x": 3, "grid_y": 3, "dc": 12, "effect_description": "2d6 falling"})),
        )
        .await;
    assert!(!trap.visible && !trap.triggered);
    let trap: wire::Trap = f
        .ok(
            Method::PATCH,
            &format!("/traps/{}", trap.id),
            Some(json!({"visible": true, "triggered": true, "grid_x": 6, "name": "Spiked pit"})),
        )
        .await;
    assert!(trap.visible && trap.triggered);
    assert_eq!(
        (trap.grid_x, trap.grid_y, trap.name.as_str()),
        (6, 3, "Spiked pit")
    );
    let again: wire::Trap = f
        .ok(
            Method::PATCH,
            &format!("/traps/{}", trap.id),
            Some(json!({"visible": true, "triggered": false})),
        )
        .await;
    assert!(again.visible, "visible true stays true");
    assert!(!again.triggered, "re-armed");
    let got: wire::Trap = f
        .ok(Method::GET, &format!("/traps/{}", trap.id), None)
        .await;
    assert_eq!(got, again);

    let poi: wire::Poi = f
        .ok(
            Method::POST,
            &format!("/maps/{}/pois", f.map),
            Some(json!({"name": "Altar", "grid_x": 2, "grid_y": 2, "icon": "star"})),
        )
        .await;
    let poi: wire::Poi = f
        .ok(
            Method::PATCH,
            &format!("/pois/{}", poi.id),
            Some(json!({"visible": true, "grid_y": 7})),
        )
        .await;
    assert!(poi.visible);
    assert_eq!((poi.grid_x, poi.grid_y, poi.icon.as_str()), (2, 7, "star"));

    for path in [format!("/traps/{}", trap.id), format!("/pois/{}", poi.id)] {
        let (status, _, _) = f.send(Method::DELETE, &path, None).await;
        assert_eq!(status, StatusCode::NO_CONTENT, "{path}");
        let (status, _) = f.status(Method::GET, &path, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn the_player_view_holds_only_what_players_see() {
    let f = fixture().await;
    let tokens = format!("/maps/{}/tokens", f.map);
    let shown: wire::Token = f
        .ok(
            Method::POST,
            &tokens,
            Some(json!({"grid_x": 1, "grid_y": 1, "label": "Shown"})),
        )
        .await;
    let hidden: wire::Token = f
        .ok(
            Method::POST,
            &tokens,
            Some(json!({"grid_x": 2, "grid_y": 1, "label": "Lurker", "hidden": true})),
        )
        .await;
    let lights = format!("/maps/{}/lights", f.map);
    let lit: wire::Light = f
        .ok(
            Method::POST,
            &lights,
            Some(json!({"grid_x": 1, "grid_y": 1, "preset": "torch"})),
        )
        .await;
    let unlit: wire::Light = f
        .ok(
            Method::POST,
            &lights,
            Some(json!({"grid_x": 2, "grid_y": 2, "preset": "lantern"})),
        )
        .await;
    let _: wire::Light = f
        .ok(
            Method::PATCH,
            &format!("/lights/{}", unlit.id),
            Some(json!({"active": false})),
        )
        .await;
    let seen_trap: wire::Trap = f
        .ok(
            Method::POST,
            &format!("/maps/{}/traps", f.map),
            Some(json!({"name": "Tripwire", "grid_x": 4, "grid_y": 4, "visible": true, "effect_description": "SECRET-EFFECT", "dc": 15})),
        )
        .await;
    let hidden_trap: wire::Trap = f
        .ok(
            Method::POST,
            &format!("/maps/{}/traps", f.map),
            Some(json!({"name": "Hidden pit", "grid_x": 5, "grid_y": 5})),
        )
        .await;

    let (status, bytes, _) = f
        .send(Method::GET, &format!("/maps/{}/player-view", f.map), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    let text = String::from_utf8(bytes).unwrap();
    let view: wire::PlayerView = serde_json::from_str(&text).unwrap();

    let ids: Vec<&str> = view.tokens.iter().map(|t| t.id.as_str()).collect();
    assert!(ids.contains(&shown.id.as_str()));
    assert!(!ids.contains(&hidden.id.as_str()), "hidden token");
    let all: Vec<wire::Token> = f.ok(Method::GET, &tokens, None).await;
    assert_eq!(
        view.tokens.len(),
        all.iter().filter(|t| t.visible_to_players).count()
    );

    let light_ids: Vec<&str> = view.lights.iter().map(|l| l.id.as_str()).collect();
    assert!(light_ids.contains(&lit.id.as_str()));
    assert!(!light_ids.contains(&unlit.id.as_str()), "unlit light");

    let marker_ids: Vec<&str> = view.markers.iter().map(|m| m.id.as_str()).collect();
    assert!(marker_ids.contains(&seen_trap.id.as_str()));
    assert!(
        !marker_ids.contains(&hidden_trap.id.as_str()),
        "hidden trap"
    );
    assert!(!text.contains("SECRET-EFFECT"), "no trap details");
    assert!(!text.contains("\"dc\""), "no trap DC");
    assert!(!text.contains("monster_id"), "no monster links");
    assert_eq!(view.map.id, f.map);
}
