//! The API error envelope: `{"error": {"code": "...", "message": "..."}}`.

use serde::{Deserialize, Serialize};

/// The body of every error response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

/// What went wrong.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: ErrorCode,
    /// For a person; the app may show it.
    pub message: String,
    /// The kind of thing that was not found ("Campaign").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
    /// Its id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// A stable error code; each has one HTTP status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// 400: the request is malformed (bad JSON, bad query).
    BadRequest,
    /// 401: no token, or the token is wrong.
    Unauthorized,
    /// 403: the token is valid but its role may not do this.
    Forbidden,
    /// 404: no such thing (or no such endpoint).
    NotFound,
    /// 409: it clashes with what exists (a duplicate name).
    Conflict,
    /// 422: the input breaks a rule (a validation error).
    Validation,
    /// 503: the server is not ready (database).
    Unavailable,
    /// 500: anything else.
    Internal,
}

impl ErrorCode {
    /// The HTTP status of this code.
    pub fn status(self) -> u16 {
        match self {
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::Validation => 422,
            Self::Unavailable => 503,
            Self::Internal => 500,
        }
    }
}

impl ErrorBody {
    /// An error with a code and a message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            error: ErrorDetail {
                code,
                message: message.into(),
                entity: None,
                id: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_envelope_round_trips_and_omits_empty_fields() {
        let body = ErrorBody::new(ErrorCode::Validation, "name is empty");
        let json = serde_json::to_string(&body).unwrap();
        assert_eq!(
            json,
            r#"{"error":{"code":"validation","message":"name is empty"}}"#
        );
        assert_eq!(serde_json::from_str::<ErrorBody>(&json).unwrap(), body);

        let nf = ErrorBody {
            error: ErrorDetail {
                code: ErrorCode::NotFound,
                message: "Not found".into(),
                entity: Some("Campaign".into()),
                id: Some("c1".into()),
            },
        };
        let back: ErrorBody = serde_json::from_str(&serde_json::to_string(&nf).unwrap()).unwrap();
        assert_eq!(back, nf);
    }

    #[test]
    fn every_code_has_its_status() {
        use ErrorCode::*;
        let pairs = [
            (BadRequest, 400),
            (Unauthorized, 401),
            (Forbidden, 403),
            (NotFound, 404),
            (Conflict, 409),
            (Validation, 422),
            (Unavailable, 503),
            (Internal, 500),
        ];
        for (code, status) in pairs {
            assert_eq!(code.status(), status, "{code:?}");
        }
    }
}
