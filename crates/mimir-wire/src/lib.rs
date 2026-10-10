//! Types on the wire between mimir-server and mimir-web (MIMIR-T-0707,
//! COLLIERY-I-0612). Both sides compile against these, so a change to the
//! API breaks the build, not the table.
//!
//! # Conventions
//!
//! - **JSON field names** are snake_case, as the Rust fields.
//! - **Ids** are strings (UUIDs as text, as stored).
//! - **Timestamps** are RFC 3339 strings, as stored.
//! - **Lists** return a JSON array; a list that can grow large returns
//!   [`Page`] and takes `?limit=&offset=`.
//! - **Errors** are [`ErrorBody`] with an [`ErrorCode`] and the HTTP status
//!   of that code ([`ErrorCode::status`]).
//! - **Routes** live under `/api/v1`; this crate has no route strings, the
//!   server and the client each name their paths.
//!
//! This crate must build for `wasm32-unknown-unknown` (checked in CI): serde
//! only. Conversions from `mimir-core` models live in the server.

pub mod campaign;
pub mod error;
pub mod map;
pub mod patch;

pub use campaign::*;
pub use error::{ErrorBody, ErrorCode, ErrorDetail};
pub use map::*;

use serde::{Deserialize, Serialize};

/// A page of a long list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    /// All matching items, not only this page.
    pub total: u64,
}

/// `GET /api/config`: what the app needs before login.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebConfig {
    /// Server version.
    pub version: String,
    pub auth: AuthMode,
}

/// How the API is protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    /// No token configured: every API call is allowed.
    Open,
    /// Send `Authorization: Bearer <token>`.
    Token,
}

/// `GET /api/v1/session`: who the token belongs to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub role: Role,
}

/// The caller's role (player links join in MIMIR-T-0716).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Dm,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_and_session_round_trip_in_snake_case() {
        let c = WebConfig {
            version: "1.0.0".into(),
            auth: AuthMode::Token,
        };
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, r#"{"version":"1.0.0","auth":"token"}"#);
        assert_eq!(serde_json::from_str::<WebConfig>(&json).unwrap(), c);
        let s = Session { role: Role::Dm };
        assert_eq!(serde_json::to_string(&s).unwrap(), r#"{"role":"dm"}"#);
    }

    #[test]
    fn a_page_carries_the_total() {
        let p = Page {
            items: vec![1, 2],
            total: 5,
        };
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, r#"{"items":[1,2],"total":5}"#);
        assert_eq!(serde_json::from_str::<Page<i32>>(&json).unwrap(), p);
    }
}
