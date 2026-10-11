//! Auth (MIMIR-T-0704, MIMIR-T-0716, ADR MIMIR-A-0010).
//!
//! [`authenticate`] runs on every API route and the socket. It names the
//! [`Caller`]:
//! - the DM: the DM bearer token, compared in constant time; or no token
//!   at all in open mode (no token configured; a warning is logged at
//!   startup);
//! - a player: the link token of a player character (stored hashed).
//!
//! Anything else is 401. [`dm_only`] then wraps the DM routes: a player
//! gets 403. Player routes check the player's scope in the handler.

use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use mimir_core::services::PlayerLinkService;
use subtle::ConstantTimeEq;

use crate::error::ApiError;
use crate::state::AppState;

pub use mimir_wire::Role;

/// Who made the request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Caller {
    Dm,
    Player(Player),
}

/// A player: the character of their link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub character_id: String,
    pub campaign_id: String,
    pub name: String,
}

impl Caller {
    pub fn role(&self) -> Role {
        match self {
            Caller::Dm => Role::Dm,
            Caller::Player(_) => Role::Player,
        }
    }
}

fn bearer(req: &Request) -> Option<String> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
}

/// Axum middleware: name the caller, or answer 401.
pub async fn authenticate(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let presented = bearer(&request);
    let dm_token = state.config.api_token.clone();
    let caller = match (&dm_token, presented) {
        (Some(dm), Some(t)) if token_matches(&t, dm) => Some(Caller::Dm),
        (None, None) => Some(Caller::Dm),
        (_, Some(t)) => {
            let found = state
                .with_db(move |conn| Ok(PlayerLinkService::new(conn).holder(&t)?))
                .await;
            match found {
                Ok(Some(h)) => Some(Caller::Player(Player {
                    character_id: h.character_id,
                    campaign_id: h.campaign_id,
                    name: h.name,
                })),
                // Open mode: any other token is the DM, as no token is.
                Ok(None) if dm_token.is_none() => Some(Caller::Dm),
                Ok(None) => None,
                Err(e) => return e.into_response(),
            }
        }
        (Some(_), None) => None,
    };
    match caller {
        Some(c) => {
            request.extensions_mut().insert(c);
            next.run(request).await
        }
        None => ApiError::unauthorized().into_response(),
    }
}

/// Axum middleware for DM routes (after [`authenticate`]): a player gets
/// 403.
pub async fn dm_only(
    Extension(caller): Extension<Caller>,
    request: Request,
    next: Next,
) -> Response {
    match caller {
        Caller::Dm => next.run(request).await,
        Caller::Player(_) => ApiError::forbidden().into_response(),
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
