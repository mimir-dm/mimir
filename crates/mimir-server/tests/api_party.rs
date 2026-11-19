//! Player notes and party sharing (MIMIR-T-0718): who writes and reads
//! notes, the DM's module view, sharing a sheet read-only, the party list.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use mimir_wire as wire;
use serde_json::{json, Value};
use tower::ServiceExt;

const DM: &str = "dm-token-for-party-tests";

struct F {
    router: axum::Router,
    _dir: tempfile::TempDir,
    campaign: String,
    module: String,
    a: String,
    b: String,
    a_token: String,
    b_token: String,
}

async fn send(
    router: &axum::Router,
    method: Method,
    path: &str,
    token: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method(method)
        .uri(format!("/api/v1{path}"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"));
    let body = match body {
        Some(v) => {
            req = req.header(header::CONTENT_TYPE, "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    let res = router
        .clone()
        .oneshot(req.body(body).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
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
    let router = mimir_server::router(AppState::new(config));
    let (_, c) = send(&router, Method::GET, "/campaigns", DM, None).await;
    let campaign = c[0]["id"].as_str().unwrap().to_string();
    let (_, m) = send(
        &router,
        Method::GET,
        &format!("/campaigns/{campaign}/modules"),
        DM,
        None,
    )
    .await;
    let module = m[0]["id"].as_str().unwrap().to_string();
    let (_, pcs) = send(
        &router,
        Method::GET,
        &format!("/campaigns/{campaign}/pcs"),
        DM,
        None,
    )
    .await;
    let a = pcs[0]["id"].as_str().unwrap().to_string();
    let b = pcs[1]["id"].as_str().unwrap().to_string();
    let (_, la) = send(
        &router,
        Method::POST,
        &format!("/characters/{a}/link"),
        DM,
        None,
    )
    .await;
    let (_, lb) = send(
        &router,
        Method::POST,
        &format!("/characters/{b}/link"),
        DM,
        None,
    )
    .await;
    F {
        router,
        _dir: dir,
        campaign,
        module,
        a,
        b,
        a_token: la["token"].as_str().unwrap().into(),
        b_token: lb["token"].as_str().unwrap().into(),
    }
}

#[tokio::test]
async fn players_write_their_notes_and_the_dm_reads_them() {
    let f = fixture().await;
    let r = &f.router;
    let path = format!("/characters/{}/notes", f.a);
    let (status, n) = send(
        r,
        Method::POST,
        &path,
        &f.a_token,
        Some(json!({"title": "Goblins", "content": "We met *goblins*.", "module_id": f.module})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let note: wire::PlayerNote = serde_json::from_value(n).unwrap();
    assert_eq!(note.module_id.as_deref(), Some(f.module.as_str()));

    // The DM reads but does not write; another player neither.
    let (status, list) = send(r, Method::GET, &path, DM, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
    for (method, token) in [
        (Method::POST, DM),
        (Method::POST, f.b_token.as_str()),
        (Method::GET, f.b_token.as_str()),
    ] {
        let (status, _) = send(r, method.clone(), &path, token, Some(json!({"title": "x"}))).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method}");
    }
    let note_path = format!("/notes/{}", note.id);
    for token in [DM, f.b_token.as_str()] {
        let (status, _) = send(
            r,
            Method::PATCH,
            &note_path,
            token,
            Some(json!({"title": "y"})),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        let (status, _) = send(r, Method::DELETE, &note_path, token, None).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    // The DM's module view names the character.
    let (status, about) = send(
        r,
        Method::GET,
        &format!("/modules/{}/notes", f.module),
        DM,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(about[0]["title"], "Goblins");
    assert!(!about[0]["character_name"].as_str().unwrap().is_empty());
    let (status, _) = send(
        r,
        Method::GET,
        &format!("/modules/{}/notes", f.module),
        &f.a_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "a DM route");

    // The player changes and deletes it.
    let (status, changed) = send(
        r,
        Method::PATCH,
        &note_path,
        &f.a_token,
        Some(json!({"title": "Goblin cave", "module_id": null})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(changed["title"], "Goblin cave");
    assert!(changed["module_id"].is_null());
    let (status, _) = send(
        r,
        Method::POST,
        &path,
        &f.a_token,
        Some(json!({"title": "  "})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = send(r, Method::DELETE, &note_path, &f.a_token, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn a_shared_sheet_is_read_only_for_the_party() {
    let f = fixture().await;
    let r = &f.router;
    let a_sheet = format!("/characters/{}", f.a);
    let (status, _) = send(r, Method::GET, &a_sheet, &f.b_token, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "not shared by default");

    let (status, s) = send(
        r,
        Method::PUT,
        &format!("/characters/{}/sharing", f.a),
        &f.a_token,
        Some(json!({"shared": true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(s["shared"], true);
    let (status, sheet) = send(r, Method::GET, &a_sheet, &f.b_token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(sheet["shared"], true);
    // Read-only: no change, no notes, no sharing for another's sheet.
    for (method, path, body) in [
        (Method::PATCH, a_sheet.clone(), json!({"bonds": "x"})),
        (
            Method::PUT,
            format!("/characters/{}/sharing", f.a),
            json!({"shared": false}),
        ),
        (
            Method::GET,
            format!("/characters/{}/notes", f.a),
            Value::Null,
        ),
        (
            Method::POST,
            format!("/characters/{}/inventory", f.a),
            json!({"item_name": "x", "item_source": "x"}),
        ),
    ] {
        let (status, _) = send(r, method.clone(), &path, &f.b_token, Some(body)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {path}");
    }

    // The party list.
    let (status, party) = send(
        r,
        Method::GET,
        &format!("/campaigns/{}/party", f.campaign),
        &f.b_token,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let members: Vec<wire::PartyMember> = serde_json::from_value(party).unwrap();
    assert!(members
        .iter()
        .any(|m| m.character_id == f.a && m.shared && !m.me));
    assert!(members
        .iter()
        .any(|m| m.character_id == f.b && m.me && !m.shared));
    let (status, _) = send(r, Method::GET, "/campaigns/other/party", &f.b_token, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // The DM may unshare; then it is closed again.
    let (status, _) = send(
        r,
        Method::PUT,
        &format!("/characters/{}/sharing", f.a),
        DM,
        Some(json!({"shared": false})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send(r, Method::GET, &a_sheet, &f.b_token, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn a_new_note_is_about_the_module_on_the_display() {
    let f = fixture().await;
    let r = &f.router;
    let path = format!("/characters/{}/notes", f.a);
    let add = |title: &'static str| {
        send(
            r,
            Method::POST,
            &path,
            &f.a_token,
            Some(json!({ "title": title, "content": "" })),
        )
    };
    // Nothing on the display: no module.
    let (status, n) = add("Before").await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(n["module_id"].is_null());

    // A map of the module on the display: the note is about that module.
    let (_, maps) = send(
        r,
        Method::GET,
        &format!("/modules/{}/maps", f.module),
        DM,
        None,
    )
    .await;
    let map = maps[0]["id"].as_str().unwrap();
    let (status, _) = send(
        r,
        Method::PUT,
        &format!("/campaigns/{}/display", f.campaign),
        DM,
        Some(json!({ "map_id": map, "blackout": false })),
    )
    .await;
    assert!(status.is_success());
    let (_, n) = add("During").await;
    assert_eq!(n["module_id"], f.module.as_str());
    let (_, module) = send(r, Method::GET, &format!("/modules/{}", f.module), DM, None).await;
    assert_eq!(n["module_name"], module["name"]);
}
