//! mimir-server skeleton: health, config, auth and the SPA fallback
//! (MIMIR-T-0704), through the real router.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use tower::ServiceExt;

struct Server {
    router: axum::Router,
    _dir: tempfile::TempDir,
}

fn server(token: Option<&str>, web: Option<&[(&str, &str)]>) -> Server {
    let dir = tempfile::tempdir().unwrap();
    let web_dist = web.map(|files| {
        let d = dir.path().join("dist");
        std::fs::create_dir_all(&d).unwrap();
        for (name, body) in files {
            std::fs::write(d.join(name), body).unwrap();
        }
        d
    });
    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir: dir.path().join("app"),
        api_token: token.map(String::from),
        web_dist,
        seed_fixture: false,
    };
    mimir_server::prepare(&config).unwrap();
    Server {
        router: mimir_server::router(AppState::new(config)),
        _dir: dir,
    }
}

async fn get(s: &Server, path: &str, bearer: Option<&str>) -> (StatusCode, String, String) {
    let mut req = Request::get(path);
    if let Some(t) = bearer {
        req = req.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let res = s
        .router
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let ctype = res
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let body =
        String::from_utf8(res.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap();
    (status, ctype, body)
}

#[tokio::test]
async fn health_and_readiness_are_open_and_the_database_is_migrated() {
    let s = server(Some("secret"), None);
    assert_eq!(get(&s, "/healthz", None).await.0, StatusCode::OK);
    let (status, _, body) = get(&s, "/readyz", None).await;
    assert_eq!((status, body.as_str()), (StatusCode::OK, "ready"));
}

#[tokio::test]
async fn api_config_tells_the_app_the_auth_mode() {
    let (status, _, body) = get(&server(None, None), "/api/config", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(r#""auth":"open""#), "{body}");
    let (_, _, body) = get(&server(Some("secret"), None), "/api/config", None).await;
    assert!(body.contains(r#""auth":"token""#), "{body}");
    assert!(body.contains(env!("CARGO_PKG_VERSION")));
}

#[tokio::test]
async fn the_api_needs_the_token_unless_open() {
    let open = server(None, None);
    let (status, _, body) = get(&open, "/api/v1/session", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(r#""role":"dm""#));

    let locked = server(Some("secret"), None);
    for bearer in [None, Some("wrong"), Some("secre")] {
        let (status, ctype, body) = get(&locked, "/api/v1/session", bearer).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{bearer:?}");
        assert!(ctype.starts_with("application/json"));
        assert!(body.contains(r#""code":"unauthorized""#), "{body}");
    }
    assert_eq!(
        get(&locked, "/api/v1/session", Some("secret")).await.0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn the_fallback_serves_assets_and_the_shell_but_never_for_api_paths() {
    let s = server(
        Some("secret"),
        Some(&[
            ("index.html", "<html>shell</html>"),
            ("app.js", "console.log(1)"),
        ]),
    );
    let (status, ctype, body) = get(&s, "/app.js", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(ctype.contains("javascript"), "{ctype}");
    assert_eq!(body, "console.log(1)");

    for route in ["/", "/campaigns/abc/dashboard", "/play/sometoken"] {
        let (status, ctype, body) = get(&s, route, None).await;
        assert_eq!(status, StatusCode::OK, "{route}");
        assert!(ctype.starts_with("text/html"), "{route}: {ctype}");
        assert_eq!(body, "<html>shell</html>", "{route}");
    }

    for reserved in ["/api/v1/nope", "/api/nope", "/ws", "/mcp/x"] {
        let (status, ctype, body) = get(&s, reserved, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{reserved}");
        assert!(ctype.starts_with("application/json"), "{reserved}");
        assert!(body.contains(r#""code":"not_found""#), "{reserved}");
    }
}

#[tokio::test]
async fn no_path_escapes_the_dist_directory() {
    let s = server(None, Some(&[("index.html", "shell")]));
    // A traversal never reads outside dist; it falls back to the shell.
    let (status, _, body) = get(&s, "/../app/data/mimir.db", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "shell");
}

#[tokio::test]
async fn without_a_built_app_a_placeholder_says_how_to_build_it() {
    let (status, ctype, body) = get(&server(None, None), "/", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(ctype.starts_with("text/html"));
    assert!(body.contains("angreal web build"), "{body}");
}
