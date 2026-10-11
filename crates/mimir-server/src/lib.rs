//! Mimir web server (COLLIERY-I-0612, ADR MIMIR-A-0009).
//!
//! An axum server over `mimir-core`: the REST API under `/api/v1` (DM
//! bearer auth, open mode without a token), the web app's own endpoints, and
//! the built Leptos app with SPA fallback.

pub mod api;
pub mod auth;
pub mod config;
pub mod error;
pub mod live;
pub mod state;
pub mod web;

use axum::extract::{Extension, State};
use axum::routing::get;
use axum::{Json, Router};
use mimir_wire::Session;

use crate::auth::{Caller, Role};
use crate::config::Config;
use crate::error::ApiError;
use crate::state::AppState;

/// Prepare the data directory and the database: create the directories,
/// run migrations, and seed the fixture when asked (MIMIR_SEED=fixture).
pub fn prepare(config: &Config) -> Result<(), String> {
    for dir in [
        config.database_path().parent().map(|p| p.to_path_buf()),
        Some(config.assets_dir()),
    ]
    .into_iter()
    .flatten()
    {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    let url = config.database_path().to_string_lossy().into_owned();
    let mut conn =
        mimir_core::db::init_database(&url).map_err(|e| format!("database init failed: {e}"))?;
    if config.seed_fixture {
        seed_fixture(&mut conn, config)?;
    }
    Ok(())
}

#[cfg(feature = "fixtures")]
fn seed_fixture(conn: &mut diesel::SqliteConnection, config: &Config) -> Result<(), String> {
    let report = mimir_core::seed::seed_ui_fixture(conn, &config.data_dir)
        .map_err(|e| format!("fixture seed failed: {e}"))?;
    tracing::info!(?report, "fixture seeded");
    Ok(())
}

#[cfg(not(feature = "fixtures"))]
fn seed_fixture(_conn: &mut diesel::SqliteConnection, _config: &Config) -> Result<(), String> {
    Err(
        "MIMIR_SEED=fixture needs a server built with --features fixtures (development only)"
            .into(),
    )
}

/// The whole router: open endpoints, the `/api/v1` API behind auth, and the
/// SPA fallback.
pub fn router(state: AppState) -> Router {
    let api = api::router(state.clone());
    // The live socket: the API's auth (DM or player), with the browser's
    // `?access_token=` copied into the header first.
    let ws = Router::new()
        .route("/ws", get(live::ws))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::authenticate,
        ))
        .route_layer(axum::middleware::from_fn(live::promote_query_token));

    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/config", get(web::api_config))
        .nest("/api/v1", api)
        .merge(ws)
        .fallback(web::spa_fallback)
        .with_state(state)
}

/// Liveness: the process answers.
async fn healthz() -> &'static str {
    "ok"
}

/// Readiness: the database opens and answers.
async fn readyz(State(state): State<AppState>) -> Result<&'static str, ApiError> {
    state
        .with_db(|conn| {
            use diesel::RunQueryDsl;
            diesel::sql_query("SELECT 1")
                .execute(conn)
                .map(|_| ())
                .map_err(|e| ApiError::unavailable(format!("database not ready: {e}")))
        })
        .await?;
    Ok("ready")
}

/// `GET /api/v1/session`: who the token belongs to (the login page checks a
/// token with it).
pub(crate) async fn session(Extension(caller): Extension<Caller>) -> Json<Session> {
    Json(match caller {
        Caller::Dm => Session {
            role: Role::Dm,
            character_id: None,
            character_name: None,
            campaign_id: None,
        },
        Caller::Player(p) => Session {
            role: Role::Player,
            character_id: Some(p.character_id),
            character_name: Some(p.name),
            campaign_id: Some(p.campaign_id),
        },
    })
}
