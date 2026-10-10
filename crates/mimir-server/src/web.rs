//! The web app's own endpoints and the SPA fallback (MIMIR-T-0704; the
//! pattern of kairos-server `web.rs` and brokkr-broker `api/assets.rs`).
//!
//! - `GET /api/config` (open): what the app needs before login.
//! - The fallback serves the built app with SPA semantics: a real asset path
//!   gets the asset, any other path (a client route like `/campaigns/x`) gets
//!   `index.html`. API surfaces are reserved and 404 with the error envelope
//!   instead. Assets come from `MIMIR_WEB_DIST`, else the binary
//!   (`embed-web` feature), else a placeholder page.

use std::borrow::Cow;

use axum::extract::State;
use axum::http::{header, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::error::ApiError;
use crate::state::AppState;

/// Paths that belong to API surfaces: never the SPA.
pub const RESERVED_PREFIXES: &[&str] = &["/api", "/ws", "/mcp", "/healthz", "/readyz"];

fn is_reserved(path: &str) -> bool {
    RESERVED_PREFIXES
        .iter()
        .any(|p| path == *p || path.starts_with(&format!("{p}/")))
}

/// `GET /api/config`
#[derive(Debug, Serialize)]
pub struct WebConfig {
    /// Server version.
    pub version: &'static str,
    /// `"open"`: no token needed; `"token"`: send the DM bearer token.
    pub auth: &'static str,
}

pub async fn api_config(State(state): State<AppState>) -> Json<WebConfig> {
    Json(WebConfig {
        version: env!("CARGO_PKG_VERSION"),
        auth: if state.config.open_mode() {
            "open"
        } else {
            "token"
        },
    })
}

/// The router fallback: reserved paths 404, anything else is the app.
pub async fn spa_fallback(State(state): State<AppState>, uri: Uri) -> Response {
    let path = uri.path();
    if is_reserved(path) {
        return ApiError::not_found(format!("no such endpoint: {path}")).into_response();
    }
    let rel = path.trim_start_matches('/');
    match load(&state, rel) {
        Some(bytes) => asset(rel, bytes),
        // A client-side route: the app shell.
        None => match load(&state, "index.html") {
            Some(bytes) => asset("index.html", bytes),
            None => placeholder(),
        },
    }
}

fn asset(path: &str, bytes: Cow<'static, [u8]>) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    // index.html must not be cached (it names the hashed bundles); hashed
    // assets can be.
    let cache = if path == "index.html" {
        "no-cache"
    } else {
        "public, max-age=31536000, immutable"
    };
    (
        [
            (header::CONTENT_TYPE, mime.as_ref().to_string()),
            (header::CACHE_CONTROL, cache.to_string()),
        ],
        bytes,
    )
        .into_response()
}

/// Load a file of the built app: the dev directory first, then the binary.
fn load(state: &AppState, rel: &str) -> Option<Cow<'static, [u8]>> {
    if rel.is_empty() || rel.split('/').any(|seg| seg == ".." || seg.is_empty()) {
        return None;
    }
    if let Some(dir) = &state.config.web_dist {
        return std::fs::read(dir.join(rel)).ok().map(Cow::Owned);
    }
    embedded(rel)
}

#[cfg(feature = "embed-web")]
fn embedded(rel: &str) -> Option<Cow<'static, [u8]>> {
    #[derive(rust_embed::RustEmbed)]
    #[folder = "../mimir-web/dist"]
    struct WebDist;
    WebDist::get(rel).map(|f| f.data)
}

#[cfg(not(feature = "embed-web"))]
fn embedded(_rel: &str) -> Option<Cow<'static, [u8]>> {
    None
}

fn placeholder() -> Response {
    (
        StatusCode::OK,
        Html(
            "<!doctype html><title>Mimir</title><h1>Mimir</h1>\
             <p>The web app is not built into this server. Build it with \
             <code>angreal web build</code> and set <code>MIMIR_WEB_DIST</code>, \
             or build the server with <code>--features embed-web</code>.</p>",
        ),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::is_reserved;

    #[test]
    fn reserves_api_surfaces_only() {
        for p in ["/api", "/api/v1/x", "/ws", "/mcp", "/healthz", "/readyz"] {
            assert!(is_reserved(p), "{p}");
        }
        for p in [
            "/",
            "/campaigns/abc",
            "/apiary",
            "/play/token",
            "/index.html",
        ] {
            assert!(!is_reserved(p), "{p}");
        }
    }
}
