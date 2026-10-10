//! API errors (MIMIR-T-0707). The envelope and the codes are
//! `mimir_wire::ErrorBody` and `mimir_wire::ErrorCode`; this module adds the
//! axum response and the mapping from `mimir_core::ServiceError`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use mimir_core::services::ServiceError;
use mimir_wire::{ErrorBody, ErrorCode, ErrorDetail};

/// An API error: a stable code (which fixes the status) and a message.
#[derive(Debug)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
    pub entity: Option<String>,
    pub id: Option<String>,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            entity: None,
            id: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    pub fn unauthorized() -> Self {
        Self::new(ErrorCode::Unauthorized, "a valid bearer token is required")
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unavailable, message)
    }

    pub fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.code.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn body(&self) -> ErrorBody {
        ErrorBody {
            error: ErrorDetail {
                code: self.code,
                message: self.message.clone(),
                entity: self.entity.clone(),
                id: self.id.clone(),
            },
        }
    }
}

impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound { entity_type, id } => Self {
                code: ErrorCode::NotFound,
                message: format!("{entity_type} not found: {id}"),
                entity: Some(entity_type),
                id: Some(id),
            },
            ServiceError::Validation(message) => Self::new(ErrorCode::Validation, message),
            ServiceError::Database(DieselError::NotFound) => Self::not_found("not found"),
            ServiceError::Database(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                info,
            )) => Self::new(
                ErrorCode::Conflict,
                format!("already exists ({})", info.message()),
            ),
            // The detail of a database or file error goes to the log, not
            // to the client.
            ServiceError::Database(e) => {
                tracing::error!(error = %e, "database error");
                Self::internal("database error")
            }
            ServiceError::Io(e) => {
                tracing::error!(error = %e, "file error");
                Self::internal("file error")
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status(), Json(self.body())).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_errors_map_to_codes_and_statuses() {
        let nf: ApiError = ServiceError::not_found("Campaign", "c1").into();
        assert_eq!(
            (nf.code, nf.status()),
            (ErrorCode::NotFound, StatusCode::NOT_FOUND)
        );
        assert_eq!(nf.body().error.entity.as_deref(), Some("Campaign"));
        assert_eq!(nf.body().error.id.as_deref(), Some("c1"));

        let v: ApiError = ServiceError::validation("name is empty").into();
        assert_eq!(v.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(v.message, "name is empty");

        let dup: ApiError = ServiceError::Database(DieselError::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            Box::new("UNIQUE constraint failed: campaigns.name".to_string()),
        ))
        .into();
        assert_eq!(dup.status(), StatusCode::CONFLICT);

        let db: ApiError = ServiceError::Database(DieselError::RollbackTransaction).into();
        assert_eq!(
            (db.code, db.message.as_str()),
            (ErrorCode::Internal, "database error")
        );

        let io: ApiError = ServiceError::Io(std::io::Error::other("disk")).into();
        assert_eq!(io.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(!io.message.contains("disk"), "IO detail must not leak");

        let row: ApiError = ServiceError::Database(DieselError::NotFound).into();
        assert_eq!(row.status(), StatusCode::NOT_FOUND);
    }
}
