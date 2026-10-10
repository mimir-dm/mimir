//! DM bearer-token auth (MIMIR-T-0704, like Skadi's `bearer_auth`).
//!
//! The layer wraps the `/api/v1` router. With no token configured the server
//! is in open mode and the layer lets every request through (a warning is
//! logged at startup). Otherwise `Authorization: Bearer <token>` must match,
//! compared in constant time. Player link tokens join in MIMIR-T-0716.

use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use subtle::ConstantTimeEq;

use crate::error::ApiError;
use crate::state::AppState;

/// Who made the request.
pub use mimir_wire::Role;

/// Axum middleware: require the DM token unless in open mode.
pub async fn require_dm(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(expected) = state.config.api_token.as_deref() else {
        request.extensions_mut().insert(Role::Dm);
        return next.run(request).await;
    };
    let presented = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    if token_matches(presented, expected) {
        request.extensions_mut().insert(Role::Dm);
        next.run(request).await
    } else {
        ApiError::unauthorized().into_response()
    }
}

/// Constant-time comparison (length is not secret).
fn token_matches(presented: &str, expected: &str) -> bool {
    presented.len() == expected.len() && bool::from(presented.as_bytes().ct_eq(expected.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::token_matches;

    #[test]
    fn matches_only_the_exact_token() {
        assert!(token_matches("secret", "secret"));
        assert!(!token_matches("secreT", "secret"));
        assert!(!token_matches("secret!", "secret"));
        assert!(!token_matches("", "secret"));
    }
}
