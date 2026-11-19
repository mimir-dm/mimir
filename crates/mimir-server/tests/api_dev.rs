//! Dev tools (MIMIR-T-0734): with the fixture on, the DM seeds, reseeds
//! and clears the fixture campaign; on any other server the routes 404 and
//! the config says no dev tools.

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use serde_json::Value;
use tower::ServiceExt;

fn server(fixture: bool) -> (axum::Router, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir: dir.path().to_path_buf(),
        api_token: None,
        web_dist: None,
        seed_fixture: false,
    };
    // Prepare without the seed (a server built without the fixtures
    // feature refuses it); the routes look at the flag.
    mimir_server::prepare(&config).unwrap();
    let config = Config {
        seed_fixture: fixture,
        ..config
    };
    (mimir_server::router(AppState::new(config)), dir)
}

async fn call(r: &axum::Router, method: Method, path: &str) -> (StatusCode, Value) {
    let res = r
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn seed_reseed_clear_with_the_fixture() {
    let (r, _dir) = server(true);
    let (_, config) = call(&r, Method::GET, "/api/config").await;
    assert_eq!(config["dev_tools"], true);
    let (status, v) = call(&r, Method::GET, "/api/v1/dev/fixture").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["seeded"], false, "a new data folder");

    let (_, v) = call(&r, Method::POST, "/api/v1/dev/fixture/seed").await;
    assert_eq!(v["seeded"], true);
    let (_, list) = call(&r, Method::GET, "/api/v1/campaigns").await;
    assert!(list
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["name"] == mimir_core::seed::TEST_CAMPAIGN_NAME));

    let (_, v) = call(&r, Method::POST, "/api/v1/dev/fixture/clear").await;
    assert_eq!(v["seeded"], false);
    let (_, v) = call(&r, Method::POST, "/api/v1/dev/fixture/reseed").await;
    assert_eq!(v["seeded"], true);
    let (status, _) = call(&r, Method::POST, "/api/v1/dev/fixture/wipe").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn no_dev_tools_without_the_fixture() {
    let (r, _dir) = server(false);
    let (_, config) = call(&r, Method::GET, "/api/config").await;
    assert_eq!(config["dev_tools"], false);
    let (status, _) = call(&r, Method::GET, "/api/v1/dev/fixture").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(&r, Method::POST, "/api/v1/dev/fixture/seed").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
