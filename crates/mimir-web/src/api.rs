//! A thin client over the same-origin API of mimir-server. Each call sends
//! the stored DM token (if any) as a bearer token. Errors are Aurora's
//! `ApiError`, filled from the server's `mimir_wire::ErrorBody`, so pages
//! show them with `ErrorState`.

use aurora_leptos::tokens::ApiError;
use gloo_net::http::{Request, RequestBuilder};
use mimir_wire::{ErrorBody, Session, WebConfig};
use serde::de::DeserializeOwned;

use crate::auth;

/// `GET /api/config`: the server version and whether a token is needed.
pub async fn config() -> Result<WebConfig, ApiError> {
    get_json(Request::get("/api/config")).await
}

/// `GET /api/v1/session` with the stored token.
pub async fn session() -> Result<Session, ApiError> {
    get_json(with_token(
        Request::get("/api/v1/session"),
        auth::stored_token(),
    ))
    .await
}

/// `GET /api/v1/session` with a token that is not stored yet (sign-in).
pub async fn check_token(token: &str) -> Result<Session, ApiError> {
    get_json(with_token(
        Request::get("/api/v1/session"),
        Some(token.to_string()),
    ))
    .await
}

/// `GET` a path under `/api/v1` with the stored token.
pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let url = format!("/api/v1{path}");
    get_json(with_token(Request::get(&url), auth::stored_token())).await
}

fn with_token(req: RequestBuilder, token: Option<String>) -> RequestBuilder {
    match token {
        Some(t) => req.header("Authorization", &bearer(&t)),
        None => req,
    }
}

/// The `Authorization` header value for a token.
pub fn bearer(token: &str) -> String {
    format!("Bearer {}", token.trim())
}

async fn get_json<T: DeserializeOwned>(req: RequestBuilder) -> Result<T, ApiError> {
    let resp = req.send().await.map_err(|_| ApiError::Network)?;
    let status = resp.status();
    if resp.ok() {
        return resp
            .json::<T>()
            .await
            .map_err(|e| ApiError::Unknown(format!("unexpected response: {e}")));
    }
    let text = resp.text().await.unwrap_or_default();
    Err(decode_error(status, &text))
}

/// An error response as Aurora's `ApiError`: the code and the message of the
/// envelope when the body is one, else the status alone.
pub fn decode_error(status: u16, body: &str) -> ApiError {
    match serde_json::from_str::<ErrorBody>(body) {
        Ok(env) => ApiError::Http {
            status,
            message: env.error.message,
            code: serde_json::to_value(env.error.code)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string)),
        },
        Err(_) => ApiError::Http {
            status,
            message: if body.trim().is_empty() {
                format!("HTTP {status}")
            } else {
                body.trim().to_string()
            },
            code: None,
        },
    }
}

/// Is this a refused token (401)?
pub fn is_unauthorized(err: &ApiError) -> bool {
    matches!(err, ApiError::Http { status: 401, .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mimir_wire::ErrorCode;

    #[test]
    fn an_envelope_decodes_to_its_code_and_message() {
        let body =
            serde_json::to_string(&ErrorBody::new(ErrorCode::Validation, "name is empty")).unwrap();
        assert!(
            decode_error(422, &body)
                == ApiError::Http {
                    status: 422,
                    message: "name is empty".into(),
                    code: Some("validation".into()),
                }
        );
    }

    #[test]
    fn a_body_that_is_not_an_envelope_keeps_the_status() {
        assert!(
            decode_error(502, "")
                == ApiError::Http {
                    status: 502,
                    message: "HTTP 502".into(),
                    code: None
                }
        );
        assert!(
            decode_error(500, "boom\n")
                == ApiError::Http {
                    status: 500,
                    message: "boom".into(),
                    code: None
                }
        );
    }

    #[test]
    fn only_401_is_a_refused_token() {
        assert!(is_unauthorized(&decode_error(401, "")));
        assert!(!is_unauthorized(&decode_error(403, "")));
        assert!(!is_unauthorized(&ApiError::Network));
    }

    #[test]
    fn the_bearer_header_trims_the_token() {
        assert_eq!(bearer("  abc \n"), "Bearer abc");
    }
}
