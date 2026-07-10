//! Dev-only invoke-over-HTTP bridge for the Playwright UI harness.
//!
//! Runs the full Mimir backend headlessly (Tauri MockRuntime, no window) and
//! exposes the invoke pipeline over HTTP so a plain browser (Playwright,
//! Chrome) can drive the real frontend against real data:
//!
//! ```text
//! POST /invoke/{command}   body: JSON args (camelCase, same as the frontend)
//! GET  /health             liveness + resolved DB path
//! ```
//!
//! Requests go through `tauri::test::get_ipc_response`, i.e. the exact same
//! `generate_handler!` dispatch as the production app — zero per-command glue.
//!
//! Safety rails (see MIMIR-T-0659/0660):
//! - only built with `--features ui-harness`; never part of release builds
//! - binds 127.0.0.1 only
//! - requires `MIMIR_BRIDGE_APP_DIR` pointing at a scratch app dir and REFUSES
//!   to start if the resolved database lives under the production app dir
//!
//! Usage:
//! ```text
//! MIMIR_BRIDGE_APP_DIR=/tmp/mimir-ui-session \
//!   cargo run -p mimir --no-default-features --features ui-harness --bin ui-bridge
//! ```

use std::path::PathBuf;
use std::sync::mpsc;

use axum::extract::{Path, State};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use mimir_core::db::init_database;
use mimir_lib::{AppPaths, AppState};
use mimir_print::PrintState;
use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
use tauri::webview::InvokeRequest;
use tauri::Manager;
use tower_http::cors::CorsLayer;

/// A single invoke forwarded from HTTP to the dispatch thread.
struct BridgeRequest {
    cmd: String,
    args: serde_json::Value,
    reply: tokio::sync::oneshot::Sender<Result<InvokeResponseBody, serde_json::Value>>,
}

#[derive(Clone)]
struct BridgeState {
    tx: mpsc::Sender<BridgeRequest>,
    db_path: String,
}

/// The production application-support directory. The bridge must never touch
/// anything under it — sessions run against disposable scratch copies.
fn production_app_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    return std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join("Library/Application Support/com.mimir.app"));
    #[cfg(target_os = "windows")]
    return std::env::var("APPDATA")
        .ok()
        .map(|h| PathBuf::from(h).join("com.mimir.app"));
    #[cfg(target_os = "linux")]
    return std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".local/share/com.mimir.app"));
}

fn build_paths(app_dir: PathBuf) -> AppPaths {
    let config_dir = app_dir.join("config");
    let data_dir = app_dir.join("data");
    let logs_dir = app_dir.join("logs");
    let assets_dir = app_dir.join("assets");
    let database_path = data_dir.join("mimir.db");
    for dir in [&app_dir, &config_dir, &data_dir, &logs_dir, &assets_dir] {
        std::fs::create_dir_all(dir).expect("Failed to create bridge app directory");
    }
    AppPaths {
        app_dir,
        config_dir,
        data_dir,
        logs_dir,
        assets_dir,
        database_path,
        is_dev: true,
    }
}

fn main() {
    let app_dir = std::env::var("MIMIR_BRIDGE_APP_DIR").unwrap_or_else(|_| {
        eprintln!("ui-bridge: MIMIR_BRIDGE_APP_DIR must point at a scratch app dir");
        eprintln!("ui-bridge: (containing data/mimir.db — a COPY, never the live database)");
        std::process::exit(1);
    });
    let app_dir = PathBuf::from(app_dir);
    let paths = build_paths(app_dir);

    // Hard refusal: never run against anything under the production app dir.
    if let Some(prod) = production_app_dir() {
        let resolved = paths
            .database_path
            .canonicalize()
            .unwrap_or_else(|_| paths.database_path.clone());
        let prod = prod.canonicalize().unwrap_or(prod);
        if resolved.starts_with(&prod) {
            eprintln!(
                "ui-bridge: REFUSING to start: {} is inside the production app dir {}",
                resolved.display(),
                prod.display()
            );
            eprintln!("ui-bridge: point MIMIR_BRIDGE_APP_DIR at a scratch copy instead");
            std::process::exit(1);
        }
    }

    let db_url = paths.database_url();
    init_database(&db_url).expect("Failed to initialize bridge database");
    eprintln!("ui-bridge: database {}", db_url);

    // The MockRuntime app lives on its own thread; HTTP handlers forward
    // invokes through this channel and commands run serially.
    let (tx, rx) = mpsc::channel::<BridgeRequest>();
    let dispatch_paths = paths.clone();
    std::thread::spawn(move || dispatch_loop(dispatch_paths, rx));

    let port: u16 = std::env::var("MIMIR_BRIDGE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4175);

    let state = BridgeState { tx, db_path: db_url };

    let cors = CorsLayer::new()
        .allow_origin([
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("http://127.0.0.1:5173"),
        ])
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/health", get(health))
        .route("/invoke/{cmd}", post(invoke))
        .layer(cors)
        .with_state(state);

    let rt = tokio::runtime::Runtime::new().expect("Failed to start tokio runtime");
    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .expect("Failed to bind bridge port");
        eprintln!("ui-bridge: listening on http://127.0.0.1:{}", port);
        axum::serve(listener, app).await.expect("Bridge server error");
    });
}

/// Owns the headless Tauri app and pushes each request through the real
/// invoke pipeline via `tauri::test::get_ipc_response`.
fn dispatch_loop(paths: AppPaths, rx: mpsc::Receiver<BridgeRequest>) {
    let app = tauri::test::mock_builder()
        .invoke_handler(mimir_lib::ipc::invoke_handler())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("Failed to build mock Tauri app");

    app.manage(AppState::new(paths.clone()));
    app.manage(PrintState::new(
        paths.app_dir.join("templates"),
        paths.assets_dir.clone(),
    ));

    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("Failed to build mock webview");

    eprintln!("ui-bridge: dispatch ready");
    while let Ok(req) = rx.recv() {
        let request = InvokeRequest {
            cmd: req.cmd,
            callback: CallbackFn(0),
            error: CallbackFn(1),
            // Must match the webview's local origin (tauri://localhost on
            // macOS/Linux) or the ACL treats the invoke as remote and blocks it.
            url: if cfg!(windows) {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            }
            .parse()
            .unwrap(),
            body: InvokeBody::Json(req.args),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        };
        let result = tauri::test::get_ipc_response(&webview, request);
        let _ = req.reply.send(result);
    }
}

async fn health(State(state): State<BridgeState>) -> Response {
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        format!(r#"{{"status":"ok","database":{}}}"#, serde_json::json!(state.db_path)),
    )
        .into_response()
}

async fn invoke(
    State(state): State<BridgeState>,
    Path(cmd): Path<String>,
    body: Option<axum::Json<serde_json::Value>>,
) -> Response {
    let args = body.map(|axum::Json(v)| v).unwrap_or_else(|| serde_json::json!({}));
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    if state
        .tx
        .send(BridgeRequest { cmd, args, reply: reply_tx })
        .is_err()
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, "dispatch thread gone").into_response();
    }
    match reply_rx.await {
        Ok(Ok(InvokeResponseBody::Json(json))) => (
            StatusCode::OK,
            [("content-type", "application/json")],
            json,
        )
            .into_response(),
        Ok(Ok(InvokeResponseBody::Raw(bytes))) => (
            StatusCode::OK,
            [("content-type", "application/octet-stream")],
            bytes,
        )
            .into_response(),
        // Command returned an Err — forward it so the browser shim can reject.
        Ok(Err(error)) => (
            StatusCode::BAD_REQUEST,
            [("content-type", "application/json")],
            error.to_string(),
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "dispatch dropped reply").into_response(),
    }
}
