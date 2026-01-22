//! Tauri Command Handlers
//!
//! This module contains all Tauri commands that bridge the Vue.js frontend
//! with the mimir-core service layer.

pub mod asset;
pub mod campaign;
pub mod catalog;
pub mod character;
pub mod document;
pub mod map;
pub mod module;
pub mod source;

use serde::Serialize;

/// API Response wrapper for frontend compatibility.
///
/// The frontend expects responses in this format with success/error handling.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response with data.
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response.
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Helper to convert service results to API responses.
pub fn to_api_response<T: Serialize, E: std::fmt::Display>(
    result: Result<T, E>,
) -> ApiResponse<T> {
    match result {
        Ok(data) => ApiResponse::ok(data),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}
