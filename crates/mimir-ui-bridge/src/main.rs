//! Dev-only invoke-over-HTTP bridge for the Playwright UI harness.
//!
//! Runs the full Mimir backend headlessly (Tauri MockRuntime, no window) and
//! exposes the invoke pipeline over HTTP so a plain browser (Playwright,
//! Chrome) can drive the real frontend against the real backend:
//!
//! ```text
//! POST /invoke/{command}   body: JSON args (camelCase, same as the frontend)
//! GET  /file?path=<abs>    a file under the scratch app dir (map images);
//!                          the browser shim's convertFileSrc points here
//! GET  /health             liveness + resolved DB path
//! ```
//!
//! Requests go through `tauri::test::get_ipc_response`, i.e. the exact same
//! `generate_handler!` dispatch as the production app — zero per-command glue.
//!
//! Data: development machines hold no real campaign data. With
//! `MIMIR_BRIDGE_SEED=fixture` the bridge seeds the UI fixture (SRD catalog +
//! "The Lost Mine of Phandelver" dev campaign, see `mimir_core::seed`) into the
//! scratch DB at startup. `scripts/ui-session.sh` always does this.
//!
//! Safety rails (see MIMIR-T-0659/0660):
//! - lives in its own dev-only crate, outside the app crate, so the Tauri
//!   bundler can never ship it
//! - binds 127.0.0.1 only
//! - requires `MIMIR_BRIDGE_APP_DIR` pointing at a scratch app dir and REFUSES
//!   to start if the resolved database lives under the production app dir
//!
//! Usage:
//! ```text
//! MIMIR_BRIDGE_APP_DIR=$(mktemp -d) MIMIR_BRIDGE_SEED=fixture cargo run -p mimir-ui-bridge
//! ```

use std::path::PathBuf;
use std::sync::mpsc;

use axum::extract::{Path, State};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use mimir_core::db::init_database;
use mimir_core::seed::{seed_ui_fixture, FIXTURE_CAMPAIGN_NAME};
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
    /// Canonical scratch app dir: the only tree `GET /file` serves from.
    files_root: PathBuf,
}

/// The production application-support directory. The bridge must never touch
/// anything under it — sessions run against a disposable scratch DB.
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
        eprintln!("ui-bridge: (an empty temp dir; set MIMIR_BRIDGE_SEED=fixture to seed it)");
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
            eprintln!("ui-bridge: point MIMIR_BRIDGE_APP_DIR at a scratch dir instead");
            std::process::exit(1);
        }
    }

    let db_url = paths.database_url();
    let mut db = init_database(&db_url).expect("Failed to initialize bridge database");
    eprintln!("ui-bridge: database {}", db_url);

    // MIMIR_BRIDGE_SEED=fixture: seed the UI fixture (SRD catalog + the dev
    // campaign, map assets under the scratch app dir). Dev machines hold no
    // real campaign data, so harness sessions start from this.
    if std::env::var("MIMIR_BRIDGE_SEED").as_deref() == Ok("fixture") {
        match seed_ui_fixture(&mut db, &paths.app_dir) {
            Ok(report) => eprintln!(
                "ui-bridge: fixture ready: campaign '{}' ({}), catalog {}",
                FIXTURE_CAMPAIGN_NAME,
                if report.campaign_created {
                    "seeded"
                } else {
                    "already present"
                },
                match report.catalog {
                    Some(c) => format!(
                        "seeded ({} spells, {} monsters, {} items)",
                        c.spells, c.monsters, c.items
                    ),
                    None => "already present".to_string(),
                }
            ),
            Err(e) => {
                eprintln!("ui-bridge: failed to seed the fixture: {}", e);
                std::process::exit(1);
            }
        }
    }
    drop(db);

    // The MockRuntime app lives on its own thread; HTTP handlers forward
    // invokes through this channel and commands run serially.
    let (tx, rx) = mpsc::channel::<BridgeRequest>();
    let dispatch_paths = paths.clone();
    std::thread::spawn(move || dispatch_loop(dispatch_paths, rx));

    let port: u16 = std::env::var("MIMIR_BRIDGE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4175);

    let files_root = paths
        .app_dir
        .canonicalize()
        .expect("Failed to resolve bridge app dir");
    let state = BridgeState {
        tx,
        db_path: db_url,
        files_root,
    };

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
        .route("/file", get(file))
        .layer(cors)
        .with_state(state);

    let rt = tokio::runtime::Runtime::new().expect("Failed to start tokio runtime");
    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .expect("Failed to bind bridge port");
        eprintln!("ui-bridge: listening on http://127.0.0.1:{}", port);
        axum::serve(listener, app)
            .await
            .expect("Bridge server error");
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
        format!(
            r#"{{"status":"ok","database":{}}}"#,
            serde_json::json!(state.db_path)
        ),
    )
        .into_response()
}

#[derive(serde::Deserialize)]
struct FileQuery {
    path: String,
}

/// Serve a file the backend handed out as a path (e.g. `serve_map_image`).
/// In the app the asset protocol does this; in a browser the shim's
/// `convertFileSrc` points here. Only files inside the scratch app dir.
async fn file(
    State(state): State<BridgeState>,
    axum::extract::Query(q): axum::extract::Query<FileQuery>,
) -> Response {
    let Some(path) = resolve_served_file(&state.files_root, &q.path) else {
        return (StatusCode::NOT_FOUND, "not found under the bridge app dir").into_response();
    };
    match std::fs::read(&path) {
        Ok(bytes) => (
            StatusCode::OK,
            [("content-type", content_type_for(&path))],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "not readable").into_response(),
    }
}

/// The canonical path of `requested` if it is an existing file inside `root`
/// (a canonical directory). `..` and symlinks are resolved before the check.
fn resolve_served_file(root: &std::path::Path, requested: &str) -> Option<PathBuf> {
    let path = PathBuf::from(requested).canonicalize().ok()?;
    (path.starts_with(root) && path.is_file()).then_some(path)
}

fn content_type_for(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("json") | Some("dd2vtt") | Some("uvtt") => "application/json",
        _ => "application/octet-stream",
    }
}

async fn invoke(
    State(state): State<BridgeState>,
    Path(cmd): Path<String>,
    body: Option<axum::Json<serde_json::Value>>,
) -> Response {
    let args = body
        .map(|axum::Json(v)| v)
        .unwrap_or_else(|| serde_json::json!({}));
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    if state
        .tx
        .send(BridgeRequest {
            cmd,
            args,
            reply: reply_tx,
        })
        .is_err()
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, "dispatch thread gone").into_response();
    }
    match reply_rx.await {
        Ok(Ok(InvokeResponseBody::Json(json))) => {
            (StatusCode::OK, [("content-type", "application/json")], json).into_response()
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(root.join("assets/map.png"), b"png").unwrap();
        (dir, root)
    }

    #[test]
    fn serves_files_inside_the_app_dir() {
        let (_dir, root) = scratch();
        let file = root.join("assets/map.png");
        assert_eq!(
            resolve_served_file(&root, file.to_str().unwrap()),
            Some(file.clone())
        );
        assert_eq!(content_type_for(&file), "image/png");
    }

    #[test]
    fn refuses_paths_outside_the_app_dir() {
        let (_dir, root) = scratch();
        let outside = tempfile::NamedTempFile::new().unwrap();
        assert_eq!(
            resolve_served_file(&root, outside.path().to_str().unwrap()),
            None
        );

        let traversal = format!("{}/assets/../../{}", root.display(), "etc/hosts");
        assert_eq!(resolve_served_file(&root, &traversal), None);
        assert_eq!(resolve_served_file(&root, "/etc/hosts"), None);
    }

    #[test]
    fn refuses_directories_and_missing_files() {
        let (_dir, root) = scratch();
        assert_eq!(
            resolve_served_file(&root, root.join("assets").to_str().unwrap()),
            None
        );
        assert_eq!(
            resolve_served_file(&root, root.join("assets/nope.png").to_str().unwrap()),
            None
        );
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlinks_that_leave_the_app_dir() {
        let (_dir, root) = scratch();
        let outside = tempfile::NamedTempFile::new().unwrap();
        let link = root.join("assets/escape.png");
        std::os::unix::fs::symlink(outside.path(), &link).unwrap();
        assert_eq!(resolve_served_file(&root, link.to_str().unwrap()), None);
    }
}
